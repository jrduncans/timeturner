use chrono::prelude::*;
use speedate::DateTime as SpeedDateTime;

// Formats speedate doesn't handle; all are interpreted as UTC
const CUSTOM_UTC_FORMATS: [&str; 5] = [
    "%d %b %Y %H:%M:%S%.f", // 03 Feb 2020 01:03:10.534
    "%F %T%.f UTC",         // 2019-11-22 09:03:44.00 UTC
    "%T UTC %F",            // 04:10:39 UTC 2020-02-17
    "%B %d, %Y %H:%M",      // May 23, 2020 12:00
    "%a %b %e %T UTC %Y",   // Sun Oct 27 22:03:19 UTC 2019 (Go UnixDate)
];

// Formats with an embedded timezone offset
const CUSTOM_ZONED_FORMATS: [&str; 2] = [
    "%d/%b/%Y:%T %z",       // 27/Oct/2019:22:03:19 +0000 (nginx access log)
    "%a %b %d %Y %T GMT%z", // Sun Oct 27 2019 22:03:19 GMT-0700 (JS Date.toString(), suffix stripped)
];

fn parse_custom_utc_format(input: &str) -> Option<DateTime<Utc>> {
    CUSTOM_UTC_FORMATS.iter().find_map(|s| {
        NaiveDateTime::parse_from_str(input, s)
            .ok()
            .map(|d| d.and_utc())
    })
}

fn parse_custom_zoned_format(input: &str) -> Option<DateTime<Utc>> {
    CUSTOM_ZONED_FORMATS
        .iter()
        .find_map(|s| DateTime::parse_from_str(input, s).ok().map(|d| d.to_utc()))
}

// Strips " (Timezone Name)" suffix produced by JS Date.toString()
fn strip_js_tz_name(input: &str) -> Option<String> {
    if input.ends_with(')') {
        input.rfind(" (").map(|pos| input[..pos].to_string())
    } else {
        None
    }
}

fn speedate_to_chrono(dt: SpeedDateTime) -> Option<DateTime<Utc>> {
    let naive = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(
            dt.date.year.into(),
            dt.date.month.into(),
            dt.date.day.into(),
        )?,
        NaiveTime::from_hms_micro_opt(
            dt.time.hour.into(),
            dt.time.minute.into(),
            dt.time.second.into(),
            dt.time.microsecond,
        )?,
    );
    Some(match dt.time.tz_offset {
        Some(offset_secs) => FixedOffset::east_opt(offset_secs)?
            .from_local_datetime(&naive)
            .single()?
            .to_utc(),
        None => naive.and_utc(),
    })
}

fn replace_comma_decimal(input: &str) -> Option<String> {
    let chars: Vec<char> = input.chars().collect();
    let mut changed = false;
    let result: String = chars
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            if c == ','
                && i > 0
                && chars[i - 1].is_ascii_digit()
                && i + 1 < chars.len()
                && chars[i + 1].is_ascii_digit()
            {
                changed = true;
                '.'
            } else {
                c
            }
        })
        .collect();
    if changed { Some(result) } else { None }
}

fn parse_with_dateparser(input: &str) -> Option<DateTime<Utc>> {
    dateparser::parse_with_timezone(input, &Utc).ok()
}

fn parse_with_speedate(input: &str) -> Option<DateTime<Utc>> {
    SpeedDateTime::parse_str(input)
        .ok()
        .and_then(speedate_to_chrono)
}

pub fn parse_input(input: Option<&str>) -> Result<DateTime<Utc>, &'static str> {
    input.filter(|i| !i.trim().is_empty()).map_or_else(
        || Ok(Utc::now()),
        |i| {
            parse_with_speedate(i)
                .or_else(|| parse_custom_utc_format(i))
                .or_else(|| {
                    replace_comma_decimal(i).and_then(|normalized| {
                        parse_with_speedate(&normalized)
                            .or_else(|| parse_custom_utc_format(&normalized))
                    })
                })
                .or_else(|| parse_custom_zoned_format(i))
                .or_else(|| strip_js_tz_name(i).and_then(|s| parse_custom_zoned_format(&s)))
                .or_else(|| parse_with_dateparser(i))
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
    fn rfc3339_input_lowercase_t() {
        let result = parse_input(Some(&String::from("2019-10-27t15:03:19.747-07:00"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1572213799747).unwrap());
    }

    #[test]
    fn rfc3339_no_offset_assumed_utc() {
        let result = parse_input(Some(&String::from("2019-10-27T22:03:19.747"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1572213799747).unwrap());
    }

    #[test]
    fn rfc3339_no_offset_no_millis_assumed_utc() {
        let result = parse_input(Some(&String::from("2019-10-27T22:03:19"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1572213799000).unwrap());
    }

    #[test]
    fn rfc3339_lowercase_t_no_offset_assumed_utc() {
        let result = parse_input(Some(&String::from("2019-10-27t22:03:19.747"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1572213799747).unwrap());
    }

    #[test]
    fn rfc3339_space_separator_no_offset_assumed_utc() {
        let result = parse_input(Some(&String::from("2019-10-27 22:03:19.747"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1572213799747).unwrap());
    }

    #[test]
    fn custom_unzoned_rfc3339_like_with_space_and_comma() {
        let result = parse_input(Some(&String::from("2020-12-17 00:00:34,247"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1608163234247).unwrap());
    }

    #[test]
    fn rfc3339_input_comma_decimal_zulu() {
        let result = parse_input(Some(&String::from("2019-10-27T22:03:19,747Z"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1572213799747).unwrap());
    }

    #[test]
    fn rfc3339_input_comma_decimal_with_offset() {
        let result = parse_input(Some(&String::from("2019-10-27T15:03:19,747-07:00"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1572213799747).unwrap());
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

    // nginx/Apache combined access log: 27/Oct/2019:22:03:19 +0000
    #[test]
    fn nginx_access_log_format() {
        let result = parse_input(Some(&String::from("27/Oct/2019:22:03:19 +0000"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1572213799000).unwrap());
    }

    #[test]
    fn nginx_access_log_format_nonzero_offset() {
        let result = parse_input(Some(&String::from("27/Oct/2019:15:03:19 -0700"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1572213799000).unwrap());
    }

    // HTTP date (RFC 7231): always GMT
    #[test]
    fn http_date_rfc7231() {
        let result = parse_input(Some(&String::from("Sun, 27 Oct 2019 22:03:19 GMT"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1572213799000).unwrap());
    }

    // RFC 2822 with numeric offset
    #[test]
    fn rfc2822_numeric_utc_offset() {
        let result = parse_input(Some(&String::from("Sun, 27 Oct 2019 22:03:19 +0000"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1572213799000).unwrap());
    }

    #[test]
    fn rfc2822_nonzero_offset() {
        let result = parse_input(Some(&String::from("Sun, 27 Oct 2019 15:03:19 -0700"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1572213799000).unwrap());
    }

    // Go UnixDate / output of Unix `date` command: Sun Oct 27 22:03:19 UTC 2019
    #[test]
    fn go_unix_date_format() {
        let result = parse_input(Some(&String::from("Sun Oct 27 22:03:19 UTC 2019"))).unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1572213799000).unwrap());
    }

    // JavaScript Date.toString(): Sun Oct 27 2019 22:03:19 GMT+0000 (Coordinated Universal Time)
    #[test]
    fn javascript_date_tostring_utc() {
        let result = parse_input(Some(&String::from(
            "Sun Oct 27 2019 22:03:19 GMT+0000 (Coordinated Universal Time)",
        )))
        .unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1572213799000).unwrap());
    }

    #[test]
    fn javascript_date_tostring_nonzero_offset() {
        let result = parse_input(Some(&String::from(
            "Sun Oct 27 2019 15:03:19 GMT-0700 (Pacific Daylight Time)",
        )))
        .unwrap();
        assert_eq!(result, Utc.timestamp_millis_opt(1572213799000).unwrap());
    }
}
