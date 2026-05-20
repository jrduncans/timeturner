use crate::{EpochUnit, TimeZoneSpec};
use chrono::prelude::*;
use speedate::DateTime as SpeedDateTime;

// Formats speedate doesn't handle; all are interpreted as UTC when no input timezone is given
const CUSTOM_UNZONED_FORMATS: [&str; 5] = [
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

fn parse_custom_unzoned_format(
    input: &str,
    input_timezone: Option<TimeZoneSpec>,
) -> Option<DateTime<Utc>> {
    CUSTOM_UNZONED_FORMATS.iter().find_map(|s| {
        NaiveDateTime::parse_from_str(input, s).ok().and_then(|d| {
            Some(match input_timezone {
                Some(tz) => tz.naive_to_utc(d)?,
                None => d.and_utc(),
            })
        })
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

fn speedate_to_chrono(
    dt: SpeedDateTime,
    input_timezone: Option<TimeZoneSpec>,
) -> Option<DateTime<Utc>> {
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
        None => match input_timezone {
            Some(tz) => tz.naive_to_utc(naive)?,
            None => naive.and_utc(),
        },
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

// Parses a signed integer string as an epoch value in the given unit.
// Returns None if the input is not a valid integer.
fn parse_epoch_with_unit(input: &str, unit: EpochUnit) -> Option<DateTime<Utc>> {
    let value: i64 = input.parse().ok()?;
    epoch_value_to_datetime(value, unit)
}

// Parses pure-(signed-)integer epoch strings, inferring the unit from the value's magnitude:
//   abs(value) < 1e11  → seconds    (covers 1970 → ~year 5138)
//   abs(value) < 1e14  → millis     (covers 1973 → ~year 5138)
//   abs(value) < 1e17  → micros     (covers 1973 → ~year 5138)
//   abs(value) >= 1e17 → nanos      (covers 1973 → 2262, capped by i64::MAX)
// Supports a leading minus sign for pre-1970 values.
fn parse_epoch_auto(input: &str) -> Option<DateTime<Utc>> {
    let value: i64 = input.parse().ok()?;
    let unit = match value.unsigned_abs() {
        v if v < 100_000_000_000 => EpochUnit::Seconds,
        v if v < 100_000_000_000_000 => EpochUnit::Millis,
        v if v < 100_000_000_000_000_000 => EpochUnit::Micros,
        _ => EpochUnit::Nanos,
    };
    epoch_value_to_datetime(value, unit)
}

fn epoch_value_to_datetime(value: i64, unit: EpochUnit) -> Option<DateTime<Utc>> {
    match unit {
        EpochUnit::Seconds => Utc.timestamp_opt(value, 0).single(),
        EpochUnit::Millis => Utc.timestamp_millis_opt(value).single(),
        EpochUnit::Micros => Utc.timestamp_micros(value).single(),
        EpochUnit::Nanos => Some(DateTime::from_timestamp_nanos(value)),
    }
}

fn parse_with_dateparser(
    input: &str,
    input_timezone: Option<TimeZoneSpec>,
) -> Option<DateTime<Utc>> {
    match input_timezone {
        Some(TimeZoneSpec::Named(tz)) => dateparser::parse_with_timezone(input, &tz).ok(),
        Some(TimeZoneSpec::Fixed(off)) => dateparser::parse_with_timezone(input, &off).ok(),
        None => dateparser::parse_with_timezone(input, &Utc).ok(),
    }
}

fn parse_with_speedate(input: &str, input_timezone: Option<TimeZoneSpec>) -> Option<DateTime<Utc>> {
    SpeedDateTime::parse_str(input)
        .ok()
        .and_then(|dt| speedate_to_chrono(dt, input_timezone))
}

pub fn parse_input(
    input: Option<&str>,
    epoch_unit: Option<EpochUnit>,
    input_timezone: Option<TimeZoneSpec>,
) -> Result<DateTime<Utc>, &'static str> {
    input.map(str::trim).filter(|i| !i.is_empty()).map_or_else(
        || Ok(Utc::now()),
        |i| {
            if let Some(unit) = epoch_unit {
                return parse_epoch_with_unit(i, unit)
                    .ok_or("--epoch-unit requires a numeric epoch input");
            }
            parse_epoch_auto(i)
                .or_else(|| parse_with_speedate(i, input_timezone))
                .or_else(|| parse_custom_unzoned_format(i, input_timezone))
                .or_else(|| {
                    replace_comma_decimal(i).and_then(|normalized| {
                        parse_with_speedate(&normalized, input_timezone)
                            .or_else(|| parse_custom_unzoned_format(&normalized, input_timezone))
                    })
                })
                .or_else(|| parse_custom_zoned_format(i))
                .or_else(|| strip_js_tz_name(i).and_then(|s| parse_custom_zoned_format(&s)))
                .or_else(|| parse_with_dateparser(i, input_timezone))
                .ok_or("Input format not recognized")
        },
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]

    use super::*;

    fn expected_from_millis(millis: i64) -> Result<DateTime<Utc>, &'static str> {
        Utc.timestamp_millis_opt(millis)
            .single()
            .ok_or("invalid millis")
    }

    #[test]
    fn missing_input() {
        let now = Utc::now();
        let result = parse_input(None, None, None).unwrap();
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
        let result = parse_input(Some(&String::from(" ")), None, None).unwrap();
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
        assert_eq!(
            parse_input(Some("1572213799747"), None, None),
            expected_from_millis(1572213799747),
        );
    }

    #[test]
    fn epoch_micros_input() {
        assert_eq!(
            parse_input(Some("1572213799747000"), None, None),
            expected_from_millis(1572213799747),
        );
    }

    #[test]
    fn epoch_nanos_input() {
        assert_eq!(
            parse_input(Some("1572213799747000000"), None, None),
            expected_from_millis(1572213799747),
        );
    }

    // Boundary: smallest 16-digit microsecond value → 2001-09-09T01:46:40 UTC
    #[test]
    fn epoch_micros_min_16_digit() {
        assert_eq!(
            parse_input(Some("1000000000000000"), None, None),
            expected_from_millis(1000000000000),
        );
    }

    // Boundary: smallest 19-digit nanosecond value → 2001-09-09T01:46:40 UTC
    #[test]
    fn epoch_nanos_min_19_digit() {
        assert_eq!(
            parse_input(Some("1000000000000000000"), None, None),
            expected_from_millis(1000000000000),
        );
    }

    // Pre-2001: 15-digit microsecond timestamp (2000-01-01T00:00:00 UTC)
    // Previously rejected by the digit-count heuristic; now handled by range-based detection.
    #[test]
    fn epoch_micros_pre_2001() {
        assert_eq!(
            parse_input(Some("946684800000000"), None, None),
            expected_from_millis(946684800000),
        );
    }

    // Pre-2001: 18-digit nanosecond timestamp (2000-01-01T00:00:00 UTC)
    #[test]
    fn epoch_nanos_pre_2001() {
        assert_eq!(
            parse_input(Some("946684800000000000"), None, None),
            expected_from_millis(946684800000),
        );
    }

    // 10-digit seconds timestamp now explicitly handled by parse_epoch_auto
    #[test]
    fn epoch_seconds_auto() {
        assert_eq!(
            parse_input(Some("1572213799"), None, None),
            expected_from_millis(1572213799000),
        );
    }

    // Pre-epoch negative seconds
    #[test]
    fn epoch_negative_seconds() {
        assert_eq!(
            parse_input(Some("-1"), None, None),
            expected_from_millis(-1000)
        );
    }

    // Forced unit: 14-digit input interpreted as microseconds
    #[test]
    fn epoch_forced_micros_14_digit() {
        assert_eq!(
            parse_input(Some("10000000000000"), Some(EpochUnit::Micros), None),
            expected_from_millis(10000000000),
        );
    }

    // Forced unit: seconds for a very short value
    #[test]
    fn epoch_forced_seconds_short() {
        assert_eq!(
            parse_input(Some("60"), Some(EpochUnit::Seconds), None),
            expected_from_millis(60000),
        );
    }

    // Forced unit rejects non-numeric input
    #[test]
    fn epoch_forced_rejects_non_numeric() {
        assert_eq!(
            parse_input(Some("2020-01-01"), Some(EpochUnit::Millis), None),
            Err("--epoch-unit requires a numeric epoch input"),
        );
    }

    #[test]
    fn rfc3339_input() {
        assert_eq!(
            parse_input(Some("2019-10-27T15:03:19.747-07:00"), None, None),
            expected_from_millis(1572213799747),
        );
    }

    #[test]
    fn rfc3339_input_no_partial_seconds() {
        assert_eq!(
            parse_input(Some("2019-10-27T15:03:19-07:00"), None, None),
            expected_from_millis(1572213799000),
        );
    }

    #[test]
    fn rfc3339_input_zulu() {
        assert_eq!(
            parse_input(Some("2019-10-27T22:03:19.747Z"), None, None),
            expected_from_millis(1572213799747),
        );
    }

    #[test]
    fn rfc3339_input_space_instead_of_t() {
        assert_eq!(
            parse_input(Some("2019-10-27 15:03:19.747-07:00"), None, None),
            expected_from_millis(1572213799747),
        );
    }

    #[test]
    fn rfc3339_input_lowercase_t() {
        assert_eq!(
            parse_input(Some("2019-10-27t15:03:19.747-07:00"), None, None),
            expected_from_millis(1572213799747),
        );
    }

    #[test]
    fn rfc3339_no_offset_assumed_utc() {
        assert_eq!(
            parse_input(Some("2019-10-27T22:03:19.747"), None, None),
            expected_from_millis(1572213799747),
        );
    }

    #[test]
    fn rfc3339_no_offset_no_millis_assumed_utc() {
        assert_eq!(
            parse_input(Some("2019-10-27T22:03:19"), None, None),
            expected_from_millis(1572213799000),
        );
    }

    #[test]
    fn rfc3339_lowercase_t_no_offset_assumed_utc() {
        assert_eq!(
            parse_input(Some("2019-10-27t22:03:19.747"), None, None),
            expected_from_millis(1572213799747),
        );
    }

    #[test]
    fn rfc3339_space_separator_no_offset_assumed_utc() {
        assert_eq!(
            parse_input(Some("2019-10-27 22:03:19.747"), None, None),
            expected_from_millis(1572213799747),
        );
    }

    #[test]
    fn custom_unzoned_rfc3339_like_with_space_and_comma() {
        assert_eq!(
            parse_input(Some("2020-12-17 00:00:34,247"), None, None),
            expected_from_millis(1608163234247),
        );
    }

    #[test]
    fn rfc3339_input_comma_decimal_zulu() {
        assert_eq!(
            parse_input(Some("2019-10-27T22:03:19,747Z"), None, None),
            expected_from_millis(1572213799747),
        );
    }

    #[test]
    fn rfc3339_input_comma_decimal_with_offset() {
        assert_eq!(
            parse_input(Some("2019-10-27T15:03:19,747-07:00"), None, None),
            expected_from_millis(1572213799747),
        );
    }

    #[test]
    fn date_spelled_short_month_time_with_dot_input() {
        assert_eq!(
            parse_input(Some("03 Feb 2020 01:03:10.534"), None, None),
            expected_from_millis(1580691790534),
        );
    }

    #[test]
    fn date_spelled_short_month_time_with_comma_input() {
        assert_eq!(
            parse_input(Some("03 Feb 2020 01:03:10,534"), None, None),
            expected_from_millis(1580691790534),
        );
    }

    #[test]
    fn year_space_date_space_utc() {
        assert_eq!(
            parse_input(Some("2019-11-22 09:03:44.00 UTC"), None, None),
            expected_from_millis(1574413424000),
        );
    }

    #[test]
    fn time_space_utc_space_date() {
        assert_eq!(
            parse_input(Some("04:10:39 UTC 2020-02-17"), None, None),
            expected_from_millis(1581912639000),
        );
    }

    #[test]
    fn test_casssandra_zoned_no_millis() {
        assert_eq!(
            parse_input(Some("2015-03-07 00:59:56+0100"), None, None),
            expected_from_millis(1425686396000),
        );
    }

    #[test]
    fn test_casssandra_zoned_millis() {
        assert_eq!(
            parse_input(Some("2015-03-07 00:59:56.001+0100"), None, None),
            expected_from_millis(1425686396001),
        );
    }

    #[test]
    fn test_mysql_datetime() {
        assert_eq!(
            parse_input(Some("2021-01-20 18:13:37.842000"), None, None),
            expected_from_millis(1611166417842),
        );
    }

    #[test]
    fn english_input() {
        assert_eq!(
            parse_input(Some("May 23, 2020 12:00"), None, None),
            expected_from_millis(1590235200000),
        );
    }

    #[test]
    fn invalid_input() {
        assert_eq!(
            parse_input(Some("not a date"), None, None),
            Err("Input format not recognized"),
        );
    }

    // nginx/Apache combined access log: 27/Oct/2019:22:03:19 +0000
    #[test]
    fn nginx_access_log_format() {
        assert_eq!(
            parse_input(Some("27/Oct/2019:22:03:19 +0000"), None, None),
            expected_from_millis(1572213799000),
        );
    }

    #[test]
    fn nginx_access_log_format_nonzero_offset() {
        assert_eq!(
            parse_input(Some("27/Oct/2019:15:03:19 -0700"), None, None),
            expected_from_millis(1572213799000),
        );
    }

    // HTTP date (RFC 7231): always GMT
    #[test]
    fn http_date_rfc7231() {
        assert_eq!(
            parse_input(Some("Sun, 27 Oct 2019 22:03:19 GMT"), None, None),
            expected_from_millis(1572213799000),
        );
    }

    // RFC 2822 with numeric offset
    #[test]
    fn rfc2822_numeric_utc_offset() {
        assert_eq!(
            parse_input(Some("Sun, 27 Oct 2019 22:03:19 +0000"), None, None),
            expected_from_millis(1572213799000),
        );
    }

    #[test]
    fn rfc2822_nonzero_offset() {
        assert_eq!(
            parse_input(Some("Sun, 27 Oct 2019 15:03:19 -0700"), None, None),
            expected_from_millis(1572213799000),
        );
    }

    // Go UnixDate / output of Unix `date` command: Sun Oct 27 22:03:19 UTC 2019
    #[test]
    fn go_unix_date_format() {
        assert_eq!(
            parse_input(Some("Sun Oct 27 22:03:19 UTC 2019"), None, None),
            expected_from_millis(1572213799000),
        );
    }

    // JavaScript Date.toString(): Sun Oct 27 2019 22:03:19 GMT+0000 (Coordinated Universal Time)
    #[test]
    fn javascript_date_tostring_utc() {
        assert_eq!(
            parse_input(
                Some("Sun Oct 27 2019 22:03:19 GMT+0000 (Coordinated Universal Time)"),
                None,
                None,
            ),
            expected_from_millis(1572213799000),
        );
    }

    #[test]
    fn javascript_date_tostring_nonzero_offset() {
        assert_eq!(
            parse_input(
                Some("Sun Oct 27 2019 15:03:19 GMT-0700 (Pacific Daylight Time)"),
                None,
                None,
            ),
            expected_from_millis(1572213799000),
        );
    }

    // 2019-10-27T22:03:19 UTC is 2019-10-27T15:03:19 in America/Los_Angeles (PDT = UTC-7)
    #[test]
    fn naive_input_with_input_tz_named() {
        let tz = crate::parse_timezone_spec("America/Los_Angeles").unwrap();
        assert_eq!(
            parse_input(Some("2019-10-27T15:03:19"), None, Some(tz)),
            expected_from_millis(1572213799000),
        );
    }

    #[test]
    fn naive_input_with_input_tz_fixed_offset() {
        let tz = crate::parse_timezone_spec("-07:00").unwrap();
        assert_eq!(
            parse_input(Some("2019-10-27T15:03:19"), None, Some(tz)),
            expected_from_millis(1572213799000),
        );
    }

    #[test]
    fn custom_unzoned_format_with_input_tz() {
        // "03 Feb 2020 01:03:10.534" is normally treated as UTC → 1580691790534
        // With +09:00, it's 9 hours earlier in UTC → 1580691790534 - 9*3600*1000 = 1580659390534
        let tz = crate::parse_timezone_spec("+09:00").unwrap();
        assert_eq!(
            parse_input(Some("03 Feb 2020 01:03:10.534"), None, Some(tz)),
            expected_from_millis(1580691790534 - 9 * 3600 * 1000),
        );
    }

    #[test]
    fn dateparser_path_with_input_tz() {
        // "May 23, 2020 12:00" + America/New_York (EDT = UTC-4) → 2020-05-23T16:00:00Z
        let tz = crate::parse_timezone_spec("America/New_York").unwrap();
        let result = parse_input(Some("May 23, 2020 12:00"), None, Some(tz)).unwrap();
        assert_eq!(result.timestamp_millis(), 1590249600000);
    }

    #[test]
    fn input_tz_does_not_affect_zoned_input() {
        // Input already carries its own offset — override must be ignored
        let tz = crate::parse_timezone_spec("Asia/Tokyo").unwrap();
        assert_eq!(
            parse_input(Some("2019-10-27T15:03:19.747-07:00"), None, Some(tz)),
            expected_from_millis(1572213799747),
        );
    }

    #[test]
    fn input_tz_does_not_affect_epoch_input() {
        let tz = crate::parse_timezone_spec("Asia/Tokyo").unwrap();
        assert_eq!(
            parse_input(Some("1572213799747"), None, Some(tz)),
            expected_from_millis(1572213799747),
        );
    }
}
