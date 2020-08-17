use chrono::prelude::*;
use chrono_english::{parse_date_string, Dialect};

#[derive(PartialEq, Debug)]
pub enum DateTimeFormat {
    Missing,
    EpochMillis,
    Rfc3339,
    CustomUnzoned,
    CustomZoned,
    English,
}

impl DateTimeFormat {
    fn parse(&self, input: &str) -> Option<ParsedInput> {
        match self {
            Self::Missing => None,
            Self::EpochMillis => parse_from_epoch_millis(input),
            Self::Rfc3339 => parse_from_rfc3339(input),
            Self::CustomUnzoned => parse_custom_unzoned_format(input),
            Self::CustomZoned => parse_custom_zoned_format(input),
            Self::English => parse_from_english(input),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct ParsedInput {
    pub input_format: DateTimeFormat,
    pub input_zone: Option<FixedOffset>,
    pub value: DateTime<Utc>,
}

fn parse_from_epoch_millis(input: &str) -> Option<ParsedInput> {
    input
        .parse::<i64>()
        .ok()
        .and_then(|e| Utc.timestamp_millis_opt(e).single())
        .map(|d| ParsedInput {
            input_format: DateTimeFormat::EpochMillis,
            input_zone: None,
            value: d,
        })
}

fn parse_from_rfc3339(input: &str) -> Option<ParsedInput> {
    DateTime::parse_from_rfc3339(&input.replace(" ", "T"))
        .ok()
        .map(|d| ParsedInput {
            input_format: DateTimeFormat::Rfc3339,
            input_zone: Some(d.timezone()),
            value: d.with_timezone(&Utc),
        })
}

const CUSTOM_UNZONED_FORMATS: [&str; 4] = [
    "%d %b %Y %H:%M:%S%.3f",
    "%d %b %Y %H:%M:%S,%3f",
    "%F %T%.3f UTC",
    "%T%.3f UTC %F",
];

fn parse_custom_unzoned_format(input: &str) -> Option<ParsedInput> {
    CUSTOM_UNZONED_FORMATS
        .iter()
        .find_map(|s| parse_from_format_unzoned(input, s))
}

fn parse_from_format_unzoned(input: &str, format: &str) -> Option<ParsedInput> {
    Utc.datetime_from_str(input, format)
        .ok()
        .map(|d| ParsedInput {
            input_format: DateTimeFormat::CustomUnzoned,
            input_zone: None,
            value: d.with_timezone(&Utc),
        })
}

const CUSTOM_ZONED_FORMATS: [&str; 0] = [];

fn parse_custom_zoned_format(input: &str) -> Option<ParsedInput> {
    CUSTOM_ZONED_FORMATS
        .iter()
        .find_map(|s| parse_from_format_zoned(input, s))
}

fn parse_from_format_zoned(input: &str, format: &str) -> Option<ParsedInput> {
    DateTime::parse_from_str(input, format)
        .ok()
        .map(|d| ParsedInput {
            input_format: DateTimeFormat::CustomZoned,
            input_zone: Some(d.timezone()),
            value: d.with_timezone(&Utc),
        })
}

fn parse_from_english(input: &str) -> Option<ParsedInput> {
    parse_date_string(input, Local::now(), Dialect::Us)
        .ok()
        .map(|d| ParsedInput {
            input_format: DateTimeFormat::English,
            input_zone: None,
            value: d.with_timezone(&Utc),
        })
}

pub fn parse_input(input: &Option<String>) -> Result<ParsedInput, &'static str> {
    input.as_ref().filter(|i| !i.trim().is_empty()).map_or_else(
        || {
            Ok(ParsedInput {
                input_format: DateTimeFormat::Missing,
                input_zone: None,
                value: Utc::now(),
            })
        },
        |i| {
            DateTimeFormat::EpochMillis
                .parse(i)
                .or_else(|| DateTimeFormat::Rfc3339.parse(i))
                .or_else(|| DateTimeFormat::CustomZoned.parse(i))
                .or_else(|| DateTimeFormat::CustomUnzoned.parse(i))
                .or_else(|| DateTimeFormat::English.parse(i))
                .ok_or("Input format not recognized")
        },
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]

    use super::*;

    #[test]
    fn missing_input() {
        let now = Utc::now();
        let result = parse_input(&None).unwrap();
        assert_eq!(result.input_format, crate::parsing::DateTimeFormat::Missing);
        assert_eq!(result.input_zone, None);
        assert!(
            result.value.timestamp_millis() >= now.timestamp_millis(),
            "Provided time {} was not after the start of the test {}",
            result.value,
            now
        );

        assert!(
            result.value.timestamp_millis() < now.timestamp_millis() + 1000,
            "Provided time {} was more than one second after the start of the test {}",
            result.value,
            now
        );
    }

    #[test]
    fn empty_input() {
        let now = Utc::now();
        let result = parse_input(&Some(String::from(" "))).unwrap();
        assert_eq!(result.input_format, crate::parsing::DateTimeFormat::Missing);
        assert_eq!(result.input_zone, None);
        assert!(
            result.value.timestamp_millis() >= now.timestamp_millis(),
            "Provided time {} was not after the start of the test {}",
            result.value,
            now
        );

        assert!(
            result.value.timestamp_millis() < now.timestamp_millis() + 1000,
            "Provided time {} was more than one second after the start of the test {}",
            result.value,
            now
        );
    }

    #[test]
    fn epoch_millis_input() {
        let result = parse_input(&Some(String::from("1572213799747"))).unwrap();
        assert_eq!(result.input_format, DateTimeFormat::EpochMillis);
        assert_eq!(result.input_zone, None);
        assert_eq!(result.value, Utc.timestamp_millis(1572213799747));
    }

    #[test]
    fn rfc3339_input() {
        let result = parse_input(&Some(String::from("2019-10-27T15:03:19.747-07:00"))).unwrap();
        assert_eq!(result.input_format, DateTimeFormat::Rfc3339);
        assert_eq!(result.input_zone, Some(FixedOffset::west(25200)));
        assert_eq!(result.value, Utc.timestamp_millis(1572213799747));
    }

    #[test]
    fn rfc3339_input_no_partial_seconds() {
        let result = parse_input(&Some(String::from("2019-10-27T15:03:19-07:00"))).unwrap();
        assert_eq!(result.input_format, DateTimeFormat::Rfc3339);
        assert_eq!(result.input_zone, Some(FixedOffset::west(25200)));
        assert_eq!(result.value, Utc.timestamp_millis(1572213799000));
    }

    #[test]
    fn rfc3339_input_zulu() {
        let result = parse_input(&Some(String::from("2019-10-27T22:03:19.747Z"))).unwrap();
        assert_eq!(result.input_format, DateTimeFormat::Rfc3339);
        assert_eq!(result.input_zone, Some(FixedOffset::west(0)));
        assert_eq!(result.value, Utc.timestamp_millis(1572213799747));
    }

    #[test]
    fn rfc3339_input_space_instead_of_t() {
        let result = parse_input(&Some(String::from("2019-10-27 15:03:19.747-07:00"))).unwrap();
        assert_eq!(result.input_format, DateTimeFormat::Rfc3339);
        assert_eq!(result.input_zone, Some(FixedOffset::west(25200)));
        assert_eq!(result.value, Utc.timestamp_millis(1572213799747));
    }

    #[test]
    fn date_spelled_short_month_time_with_dot_input() {
        let result = parse_input(&Some(String::from("03 Feb 2020 01:03:10.534"))).unwrap();
        assert_eq!(result.input_format, DateTimeFormat::CustomUnzoned);
        assert_eq!(result.input_zone, None);
        assert_eq!(result.value, Utc.timestamp_millis(1580691790534));
    }

    #[test]
    fn date_spelled_short_month_time_with_comma_input() {
        let result = parse_input(&Some(String::from("03 Feb 2020 01:03:10,534"))).unwrap();
        assert_eq!(result.input_format, DateTimeFormat::CustomUnzoned);
        assert_eq!(result.input_zone, None);
        assert_eq!(result.value, Utc.timestamp_millis(1580691790534));
    }

    #[test]
    fn year_space_date_space_utc() {
        let result = parse_input(&Some(String::from("2019-11-22 09:03:44.00 UTC"))).unwrap();
        assert_eq!(result.input_format, DateTimeFormat::CustomUnzoned);
        assert_eq!(result.input_zone, None);
        assert_eq!(result.value, Utc.timestamp_millis(1574413424000));
    }

    #[test]
    fn time_space_utc_space_date() {
        let result = parse_input(&Some(String::from("04:10:39 UTC 2020-02-17"))).unwrap();
        assert_eq!(result.input_format, DateTimeFormat::CustomUnzoned);
        assert_eq!(result.input_zone, None);
        assert_eq!(result.value, Utc.timestamp_millis(1581912639000));
    }

    #[test]
    fn english_input() {
        let result = parse_input(&Some(String::from("May 23, 2020 12:00"))).unwrap();
        assert_eq!(result.input_format, DateTimeFormat::English);
        assert_eq!(result.input_zone, None);
        assert_eq!(result.value, Utc.timestamp_millis(1590260400000));
    }

    #[test]
    fn invalid_input() {
        let result = parse_input(&Some(String::from("not a date"))).err();
        assert_eq!(result, Some("Input format not recognized"));
    }
}
