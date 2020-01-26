use chrono::prelude::*;

#[derive(PartialEq, Debug)]
pub enum DateTimeFormat {
    Missing,
    EpochMillis,
    Rfc3339,
}

impl DateTimeFormat {
    fn parse(&self, input: &str) -> Option<ParsedInput> {
        match self {
            DateTimeFormat::EpochMillis => parse_from_epoch_millis(input),
            DateTimeFormat::Rfc3339 => parse_from_rfc3339(input),
            DateTimeFormat::Missing => None,
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

pub fn parse_input(input: Option<String>) -> Result<ParsedInput, &'static str> {
    input
        .map(|i| {
            DateTimeFormat::EpochMillis
                .parse(&i)
                .or_else(|| DateTimeFormat::Rfc3339.parse(&i))
                .ok_or("Input format not recognized")
        })
        .unwrap_or_else(|| {
            Ok(ParsedInput {
                input_format: DateTimeFormat::Missing,
                input_zone: None,
                value: Utc::now(),
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_input() {
        let now = Utc::now();
        let result = parse_input(None).unwrap();
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
        let result = parse_input(Some(String::from("1572213799747"))).unwrap();
        assert_eq!(result.input_format, DateTimeFormat::EpochMillis);
        assert_eq!(result.input_zone, None);
        assert_eq!(result.value, Utc.timestamp_millis(1572213799747));
    }

    #[test]
    fn rfc3339_input() {
        let result = parse_input(Some(String::from("2019-10-27T15:03:19.747-07:00"))).unwrap();
        assert_eq!(result.input_format, DateTimeFormat::Rfc3339);
        assert_eq!(result.input_zone, Some(FixedOffset::west(25200)));
        assert_eq!(result.value, Utc.timestamp_millis(1572213799747));
    }

    #[test]
    fn rfc3339_input_no_partial_seconds() {
        let result = parse_input(Some(String::from("2019-10-27T15:03:19-07:00"))).unwrap();
        assert_eq!(result.input_format, DateTimeFormat::Rfc3339);
        assert_eq!(result.input_zone, Some(FixedOffset::west(25200)));
        assert_eq!(result.value, Utc.timestamp_millis(1572213799000));
    }

    #[test]
    fn rfc3339_input_zulu() {
        let result = parse_input(Some(String::from("2019-10-27T22:03:19.747Z"))).unwrap();
        assert_eq!(result.input_format, DateTimeFormat::Rfc3339);
        assert_eq!(result.input_zone, Some(FixedOffset::west(0)));
        assert_eq!(result.value, Utc.timestamp_millis(1572213799747));
    }

    #[test]
    fn rfc3339_input_space_instead_of_t() {
        let result = parse_input(Some(String::from("2019-10-27 15:03:19.747-07:00"))).unwrap();
        assert_eq!(result.input_format, DateTimeFormat::Rfc3339);
        assert_eq!(result.input_zone, Some(FixedOffset::west(25200)));
        assert_eq!(result.value, Utc.timestamp_millis(1572213799747));
    }

    #[test]
    fn invalid_input() {
        let result = parse_input(Some(String::from("not a date"))).err();
        assert_eq!(result, Some("Input format not recognized"));
    }
}
