use chrono::prelude::*;
use dateparser::parse_with;

const CUSTOM_UNZONED_FORMATS: [&str; 3] = ["%F %T,%3f", "%d %b %Y %H:%M:%S,%3f", "%T%.3f UTC %F"];

fn parse_custom_unzoned_format(input: &str) -> Option<DateTime<Utc>> {
    CUSTOM_UNZONED_FORMATS
        .iter()
        .find_map(|s| parse_from_format_unzoned(input, s))
}

fn parse_from_format_unzoned(input: &str, format: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(input, format)
        .map(|d| d.and_utc())
        .ok()
}

fn parse_with_dateparser(input: &str) -> Option<DateTime<Utc>> {
    let midnight = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    parse_with(input, &Utc, midnight).ok()
}

pub fn parse_input(input: Option<&String>) -> Result<DateTime<Utc>, &'static str> {
    input.as_ref().filter(|i| !i.trim().is_empty()).map_or_else(
        || Ok(Utc::now()),
        |i| {
            parse_with_dateparser(i)
                .or_else(|| parse_custom_unzoned_format(i))
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
        let result = parse_input(None).unwrap();
        assert!(
            result.timestamp_millis() >= now.timestamp_millis(),
            "Provided time {result} was not after the start of the test {now}"
        );

        assert!(
            result.timestamp_millis() < now.timestamp_millis() + 1000,
            "Provided time {result} was more than one second after the start of the test {now}"
        );
    }

    #[test]
    fn empty_input() {
        let now = Utc::now();
        let result = parse_input(Some(&String::from(" "))).unwrap();
        assert!(
            result.timestamp_millis() >= now.timestamp_millis(),
            "Provided time {result} was not after the start of the test {now}"
        );

        assert!(
            result.timestamp_millis() < now.timestamp_millis() + 1000,
            "Provided time {result} was more than one second after the start of the test {now}"
        );
    }

    #[test]
    fn epoch_millis_input() {
        let result = parse_input(Some(&String::from("1572213799747"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1572213799747).unwrap());
    }

    #[test]
    fn rfc3339_input() {
        let result = parse_input(Some(&String::from("2019-10-27T15:03:19.747-07:00"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1572213799747).unwrap());
    }

    #[test]
    fn rfc3339_input_no_partial_seconds() {
        let result = parse_input(Some(&String::from("2019-10-27T15:03:19-07:00"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1572213799000).unwrap());
    }

    #[test]
    fn rfc3339_input_zulu() {
        let result = parse_input(Some(&String::from("2019-10-27T22:03:19.747Z"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1572213799747).unwrap());
    }

    #[test]
    fn rfc3339_input_space_instead_of_t() {
        let result = parse_input(Some(&String::from("2019-10-27 15:03:19.747-07:00"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1572213799747).unwrap());
    }

    #[test]
    fn custom_unzoned_rfc3339_like_with_space_and_comma() {
        let result = parse_input(Some(&String::from("2020-12-17 00:00:34,247"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1608163234247).unwrap());
    }

    #[test]
    fn date_spelled_short_month_time_with_dot_input() {
        let result = parse_input(Some(&String::from("03 Feb 2020 01:03:10.534"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1580691790534).unwrap());
    }

    #[test]
    fn date_spelled_short_month_time_with_comma_input() {
        let result = parse_input(Some(&String::from("03 Feb 2020 01:03:10,534"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1580691790534).unwrap());
    }

    #[test]
    fn year_space_date_space_utc() {
        let result = parse_input(Some(&String::from("2019-11-22 09:03:44.00 UTC"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1574413424000).unwrap());
    }

    #[test]
    fn time_space_utc_space_date() {
        let result = parse_input(Some(&String::from("04:10:39 UTC 2020-02-17"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1581912639000).unwrap());
    }

    #[test]
    fn test_casssandra_zoned_no_millis() {
        let result = parse_input(Some(&String::from("2015-03-07 00:59:56+0100"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1425686396000).unwrap());
    }

    #[test]
    fn test_casssandra_zoned_millis() {
        let result = parse_input(Some(&String::from("2015-03-07 00:59:56.001+0100"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1425686396001).unwrap());
    }

    #[test]
    fn test_mysql_datetime() {
        let result = parse_input(Some(&String::from("2021-01-20 18:13:37.842000"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1611166417842).unwrap());
    }

    #[test]
    fn english_input() {
        let result = parse_input(Some(&String::from("May 23, 2020 12:00"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1590235200000).unwrap());
    }

    #[test]
    fn invalid_input() {
        let result = parse_input(Some(&String::from("not a date"))).err();
        assert_eq!(result, Some("Input format not recognized"));
    }
}
