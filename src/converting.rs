use crate::{DurationUnit, OutputFormat, TimeZoneSpec};
use chrono::prelude::*;
use humantime::format_duration;
use std::time::Duration;

#[derive(PartialEq, Eq, Debug)]
pub struct ConversionResult {
    pub converted_text: String,
    pub format: OutputFormat,
}

pub fn convert(
    parsed_input: &DateTime<Utc>,
    now: &DateTime<Utc>,
    outputs: &[OutputFormat],
    display_tz: Option<TimeZoneSpec>,
    extra_duration_unit: Option<DurationUnit>,
) -> Vec<ConversionResult> {
    let mut results: Vec<ConversionResult> = outputs
        .iter()
        .map(|fmt| {
            let text = match fmt {
                OutputFormat::Utc => parsed_input.to_rfc3339_opts(SecondsFormat::Millis, true),
                OutputFormat::Zoned => match display_tz {
                    Some(tz) => tz.format_rfc3339_millis(parsed_input),
                    None => parsed_input
                        .with_timezone(&Local)
                        .to_rfc3339_opts(SecondsFormat::Millis, true),
                },
                OutputFormat::Seconds => parsed_input.timestamp().to_string(),
                OutputFormat::Millis => parsed_input.timestamp_millis().to_string(),
                OutputFormat::Nanos => parsed_input
                    .timestamp_nanos_opt()
                    .map_or_else(|| String::from("out of range"), |n| n.to_string()),
                OutputFormat::Duration => human_duration_since(parsed_input, now),
                OutputFormat::DurationSinceUnits(_) => unreachable!(),
            };
            ConversionResult {
                converted_text: text,
                format: *fmt,
            }
        })
        .collect();

    if let Some(duration_unit) = extra_duration_unit {
        results.push(ConversionResult {
            converted_text: unit_duration_since(parsed_input, now, duration_unit),
            format: OutputFormat::DurationSinceUnits(duration_unit),
        });
    }

    results
}

pub fn human_duration_since(input: &DateTime<Utc>, now: &DateTime<Utc>) -> String {
    let difference_millis = now.timestamp_millis() - input.timestamp_millis();

    let in_future = difference_millis.is_negative();

    let duration = Duration::from_millis(difference_millis.unsigned_abs());
    let duration_format = format_duration(duration);

    if in_future {
        format!("in {duration_format}")
    } else {
        format!("{duration_format} ago")
    }
}

pub fn unit_duration_since(
    input: &DateTime<Utc>,
    now: &DateTime<Utc>,
    duration_unit: DurationUnit,
) -> String {
    let difference_millis = now.timestamp_millis() - input.timestamp_millis();

    let in_future = difference_millis.is_negative();
    let difference_millis = difference_millis.unsigned_abs();

    let duration_format = match duration_unit {
        DurationUnit::Milliseconds => format!("{difference_millis} ms"),
        DurationUnit::Seconds => rounded_division(difference_millis, "s", 1000.0),
        DurationUnit::Minutes => rounded_division(difference_millis, "m", 60.0 * 1000.0),
        DurationUnit::Hours => rounded_division(difference_millis, "h", 60.0 * 60.0 * 1000.0),
        DurationUnit::Days => {
            rounded_division(difference_millis, "days", 24.0 * 60.0 * 60.0 * 1000.0)
        }
        DurationUnit::Weeks => rounded_division(
            difference_millis,
            "weeks",
            7.0 * 24.0 * 60.0 * 60.0 * 1000.0,
        ),
        DurationUnit::Fortnights => rounded_division(
            difference_millis,
            "fortnights",
            14.0 * 24.0 * 60.0 * 60.0 * 1000.0,
        ),
    };

    if in_future {
        format!("in {duration_format}")
    } else {
        format!("{duration_format} ago")
    }
}

#[allow(clippy::cast_precision_loss)]
fn rounded_division(value: u64, units: &str, divide_by: f64) -> String {
    format!("{:.1} {units}", value as f64 / divide_by)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]

    use super::*;
    use crate::DEFAULT_OUTPUTS;

    fn datetime_from_millis(millis: i64) -> DateTime<Utc> {
        Utc.timestamp_millis_opt(millis)
            .single()
            .expect("valid millis")
    }

    #[test]
    fn missing_input() {
        let now = datetime_from_millis(1572303922748);
        let date = datetime_from_millis(1572213799747);
        let result = convert(&date, &now, DEFAULT_OUTPUTS, None, None);

        assert_eq!(
            result,
            vec![
                ConversionResult {
                    converted_text: String::from("2019-10-27T22:03:19.747Z"),
                    format: OutputFormat::Utc
                },
                ConversionResult {
                    converted_text: date
                        .with_timezone(&Local)
                        .to_rfc3339_opts(SecondsFormat::Millis, true),
                    format: OutputFormat::Zoned
                },
                ConversionResult {
                    converted_text: String::from("1572213799747"),
                    format: OutputFormat::Millis
                },
                ConversionResult {
                    converted_text: String::from("1day 1h 2m 3s 1ms ago"),
                    format: OutputFormat::Duration
                }
            ]
        );
    }

    #[test]
    fn epoch_millis_input() {
        let now = datetime_from_millis(1572123676746);
        let date = datetime_from_millis(1572213799747);
        let result = convert(&date, &now, DEFAULT_OUTPUTS, None, None);

        assert_eq!(
            result,
            vec![
                ConversionResult {
                    converted_text: String::from("2019-10-27T22:03:19.747Z"),
                    format: OutputFormat::Utc
                },
                ConversionResult {
                    converted_text: date
                        .with_timezone(&Local)
                        .to_rfc3339_opts(SecondsFormat::Millis, true),
                    format: OutputFormat::Zoned
                },
                ConversionResult {
                    converted_text: String::from("1572213799747"),
                    format: OutputFormat::Millis
                },
                ConversionResult {
                    converted_text: String::from("in 1day 1h 2m 3s 1ms"),
                    format: OutputFormat::Duration
                }
            ]
        );
    }

    #[test]
    fn rfc3339_utc() {
        let now = datetime_from_millis(1572213929748);
        let date = datetime_from_millis(1572213799747);
        let result = convert(&date, &now, DEFAULT_OUTPUTS, None, None);

        assert_eq!(
            result,
            vec![
                ConversionResult {
                    converted_text: date
                        .with_timezone(&Utc)
                        .to_rfc3339_opts(SecondsFormat::Millis, true),
                    format: OutputFormat::Utc
                },
                ConversionResult {
                    converted_text: date
                        .with_timezone(&Local)
                        .to_rfc3339_opts(SecondsFormat::Millis, true),
                    format: OutputFormat::Zoned
                },
                ConversionResult {
                    converted_text: String::from("1572213799747"),
                    format: OutputFormat::Millis
                },
                ConversionResult {
                    converted_text: String::from("2m 10s 1ms ago"),
                    format: OutputFormat::Duration
                }
            ]
        );
    }

    #[test]
    fn rfc3339_offset() {
        let now = datetime_from_millis(1572213799749);
        let date = datetime_from_millis(1572213799747);
        let result = convert(&date, &now, DEFAULT_OUTPUTS, None, None);

        assert_eq!(
            result,
            vec![
                ConversionResult {
                    converted_text: String::from("2019-10-27T22:03:19.747Z"),
                    format: OutputFormat::Utc
                },
                ConversionResult {
                    converted_text: date
                        .with_timezone(&Local)
                        .to_rfc3339_opts(SecondsFormat::Millis, true),
                    format: OutputFormat::Zoned
                },
                ConversionResult {
                    converted_text: String::from("1572213799747"),
                    format: OutputFormat::Millis
                },
                ConversionResult {
                    converted_text: String::from("2ms ago"),
                    format: OutputFormat::Duration
                }
            ]
        );
    }

    #[test]
    fn duration_millis() {
        let now = datetime_from_millis(1572123676746);
        let date = datetime_from_millis(1572213799747);
        let result = convert(
            &date,
            &now,
            DEFAULT_OUTPUTS,
            None,
            Some(DurationUnit::Milliseconds),
        );

        assert_eq!(
            result.last().unwrap(),
            &ConversionResult {
                converted_text: String::from("in 90123001 ms"),
                format: OutputFormat::DurationSinceUnits(DurationUnit::Milliseconds)
            }
        );
    }

    #[test]
    fn duration_seconds() {
        let now = datetime_from_millis(1572123676746);
        let date = datetime_from_millis(1572213799747);
        let result = convert(
            &date,
            &now,
            DEFAULT_OUTPUTS,
            None,
            Some(DurationUnit::Seconds),
        );

        assert_eq!(
            result.last().unwrap(),
            &ConversionResult {
                converted_text: String::from("in 90123.0 s"),
                format: OutputFormat::DurationSinceUnits(DurationUnit::Seconds)
            }
        );
    }

    #[test]
    fn duration_minutes() {
        let now = datetime_from_millis(1572123676746);
        let date = datetime_from_millis(1572213799747);
        let result = convert(
            &date,
            &now,
            DEFAULT_OUTPUTS,
            None,
            Some(DurationUnit::Minutes),
        );

        assert_eq!(
            result.last().unwrap(),
            &ConversionResult {
                converted_text: String::from("in 1502.1 m"),
                format: OutputFormat::DurationSinceUnits(DurationUnit::Minutes)
            }
        );
    }

    #[test]
    fn duration_hours() {
        let now = datetime_from_millis(1572123676746);
        let date = datetime_from_millis(1572213799747);
        let result = convert(
            &date,
            &now,
            DEFAULT_OUTPUTS,
            None,
            Some(DurationUnit::Hours),
        );

        assert_eq!(
            result.last().unwrap(),
            &ConversionResult {
                converted_text: String::from("in 25.0 h"),
                format: OutputFormat::DurationSinceUnits(DurationUnit::Hours)
            }
        );
    }

    #[test]
    fn duration_days() {
        let now = datetime_from_millis(1572123676746);
        let date = datetime_from_millis(1572213799747);
        let result = convert(&date, &now, DEFAULT_OUTPUTS, None, Some(DurationUnit::Days));

        assert_eq!(
            result.last().unwrap(),
            &ConversionResult {
                converted_text: String::from("in 1.0 days"),
                format: OutputFormat::DurationSinceUnits(DurationUnit::Days)
            }
        );
    }

    #[test]
    fn duration_weeks() {
        let now = datetime_from_millis(1572123676746);
        let date = datetime_from_millis(1572213799747);
        let result = convert(
            &date,
            &now,
            DEFAULT_OUTPUTS,
            None,
            Some(DurationUnit::Weeks),
        );

        assert_eq!(
            result.last().unwrap(),
            &ConversionResult {
                converted_text: String::from("in 0.1 weeks"),
                format: OutputFormat::DurationSinceUnits(DurationUnit::Weeks)
            }
        );
    }

    #[test]
    fn duration_fortnights() {
        let now = datetime_from_millis(1572123676746);
        let date = datetime_from_millis(1572213799747);
        let result = convert(
            &date,
            &now,
            DEFAULT_OUTPUTS,
            None,
            Some(DurationUnit::Fortnights),
        );

        assert_eq!(
            result.last().unwrap(),
            &ConversionResult {
                converted_text: String::from("in 0.1 fortnights"),
                format: OutputFormat::DurationSinceUnits(DurationUnit::Fortnights)
            }
        );
    }

    #[test]
    fn select_outputs_subset() {
        let now = datetime_from_millis(1572303922748);
        let date = datetime_from_millis(1572213799747);
        let result = convert(
            &date,
            &now,
            &[OutputFormat::Seconds, OutputFormat::Millis],
            None,
            None,
        );

        assert_eq!(
            result,
            vec![
                ConversionResult {
                    converted_text: String::from("1572213799"),
                    format: OutputFormat::Seconds
                },
                ConversionResult {
                    converted_text: String::from("1572213799747"),
                    format: OutputFormat::Millis
                },
            ]
        );
    }

    #[test]
    fn select_outputs_with_duration_unit_appends() {
        let now = datetime_from_millis(1572123676746);
        let date = datetime_from_millis(1572213799747);
        let result = convert(
            &date,
            &now,
            &[OutputFormat::Utc],
            None,
            Some(DurationUnit::Days),
        );

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].format, OutputFormat::Utc);
        assert_eq!(
            result[1],
            ConversionResult {
                converted_text: String::from("in 1.0 days"),
                format: OutputFormat::DurationSinceUnits(DurationUnit::Days)
            }
        );
    }

    #[test]
    fn epoch_seconds_output() {
        let now = datetime_from_millis(1572303922748);
        let date = datetime_from_millis(1572213799747);
        let result = convert(&date, &now, &[OutputFormat::Seconds], None, None);

        assert_eq!(result[0].converted_text, "1572213799");
    }

    #[test]
    fn epoch_nanoseconds_output() {
        let now = datetime_from_millis(1572303922748);
        let date = datetime_from_millis(1572213799747);
        let result = convert(&date, &now, &[OutputFormat::Nanos], None, None);

        assert_eq!(result[0].converted_text, "1572213799747000000");
    }

    #[test]
    fn epoch_nanoseconds_overflow() {
        let now = datetime_from_millis(1572303922748);
        // Year 2300 is beyond the i64 nanos range (~2262)
        let far_future = Utc.with_ymd_and_hms(2300, 1, 1, 0, 0, 0).unwrap();
        let result = convert(&far_future, &now, &[OutputFormat::Nanos], None, None);

        assert_eq!(result[0].converted_text, "out of range");
    }

    #[test]
    fn rfc3339_zoned_with_named_tz() {
        let now = datetime_from_millis(1572303922748);
        let date = datetime_from_millis(1572213799747);
        let tz = crate::parse_timezone_spec("Asia/Tokyo").unwrap();
        let result = convert(&date, &now, &[OutputFormat::Zoned], Some(tz), None);

        assert!(
            result[0].converted_text.ends_with("+09:00"),
            "got: {}",
            result[0].converted_text
        );
    }

    #[test]
    fn rfc3339_zoned_with_fixed_offset() {
        let now = datetime_from_millis(1572303922748);
        let date = datetime_from_millis(1572213799747);
        let tz = crate::parse_timezone_spec("-05:00").unwrap();
        let result = convert(&date, &now, &[OutputFormat::Zoned], Some(tz), None);

        assert!(
            result[0].converted_text.ends_with("-05:00"),
            "got: {}",
            result[0].converted_text
        );
    }

    #[test]
    fn rfc3339_zoned_default_uses_local() {
        let now = datetime_from_millis(1572303922748);
        let date = datetime_from_millis(1572213799747);
        let result = convert(&date, &now, &[OutputFormat::Zoned], None, None);

        let expected = date
            .with_timezone(&Local)
            .to_rfc3339_opts(SecondsFormat::Millis, true);
        assert_eq!(result[0].converted_text, expected);
    }
}
