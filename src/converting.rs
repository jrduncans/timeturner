use crate::DurationUnit;
use chrono::prelude::*;
use humantime::format_duration;
use std::convert::TryInto;
use std::time::Duration;

#[derive(PartialEq, Eq, Debug)]
pub enum ConversionFormat {
    Rfc3339Utc,
    Rfc3339Local,
    EpochMillis,
    DurationSince,
    DurationSinceUnits(DurationUnit),
}

#[derive(PartialEq, Eq, Debug)]
pub struct ConversionResult {
    pub converted_text: String,
    pub format: ConversionFormat,
}

pub fn convert(
    parsed_input: &DateTime<Utc>,
    now: &DateTime<Utc>,
    extra_duration_unit: Option<DurationUnit>,
) -> Vec<ConversionResult> {
    let mut results = vec![
        ConversionResult {
            converted_text: parsed_input.to_rfc3339_opts(SecondsFormat::Millis, true),
            format: ConversionFormat::Rfc3339Utc,
        },
        ConversionResult {
            converted_text: parsed_input
                .with_timezone(&Local)
                .to_rfc3339_opts(SecondsFormat::Millis, true),
            format: ConversionFormat::Rfc3339Local,
        },
        ConversionResult {
            converted_text: parsed_input.timestamp_millis().to_string(),
            format: ConversionFormat::EpochMillis,
        },
        ConversionResult {
            converted_text: human_duration_since(parsed_input, now),
            format: ConversionFormat::DurationSince,
        },
    ];

    if let Some(duration_unit) = extra_duration_unit {
        results.push(ConversionResult {
            converted_text: unit_duration_since(parsed_input, now, duration_unit),
            format: ConversionFormat::DurationSinceUnits(duration_unit),
        });
    }

    results
}

pub fn human_duration_since(input: &DateTime<Utc>, now: &DateTime<Utc>) -> String {
    let difference_millis = now.timestamp_millis() - input.timestamp_millis();

    let in_future = difference_millis.is_negative();
    let difference_millis = difference_millis.abs();

    let duration = Duration::from_millis(difference_millis.try_into().unwrap());
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
    let difference_millis = difference_millis.abs();

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

#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn rounded_division(value: i64, units: &str, divide_by: f64) -> String {
    format!("{:.1} {units}", value as f64 / divide_by)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]

    use super::*;

    #[test]
    fn missing_input() {
        let now = Utc.timestamp_millis_opt(1572303922748).unwrap();
        let date = Utc.timestamp_millis_opt(1572213799747).unwrap();
        let result = convert(&date, &now, None);

        assert_eq!(
            result,
            vec![
                ConversionResult {
                    converted_text: String::from("2019-10-27T22:03:19.747Z"),
                    format: ConversionFormat::Rfc3339Utc
                },
                ConversionResult {
                    converted_text: date
                        .with_timezone(&Local)
                        .to_rfc3339_opts(SecondsFormat::Millis, true),
                    format: ConversionFormat::Rfc3339Local
                },
                ConversionResult {
                    converted_text: String::from("1572213799747"),
                    format: ConversionFormat::EpochMillis
                },
                ConversionResult {
                    converted_text: String::from("1day 1h 2m 3s 1ms ago"),
                    format: ConversionFormat::DurationSince
                }
            ]
        );
    }

    #[test]
    fn epoch_millis_input() {
        let now = Utc.timestamp_millis_opt(1572123676746).unwrap();
        let date = Utc.timestamp_millis_opt(1572213799747).unwrap();
        let result = convert(&date, &now, None);

        assert_eq!(
            result,
            vec![
                ConversionResult {
                    converted_text: String::from("2019-10-27T22:03:19.747Z"),
                    format: ConversionFormat::Rfc3339Utc
                },
                ConversionResult {
                    converted_text: date
                        .with_timezone(&Local)
                        .to_rfc3339_opts(SecondsFormat::Millis, true),
                    format: ConversionFormat::Rfc3339Local
                },
                ConversionResult {
                    converted_text: String::from("1572213799747"),
                    format: ConversionFormat::EpochMillis
                },
                ConversionResult {
                    converted_text: String::from("in 1day 1h 2m 3s 1ms"),
                    format: ConversionFormat::DurationSince
                }
            ]
        );
    }

    #[test]
    fn rfc3339_utc() {
        let now = Utc.timestamp_millis_opt(1572213929748).unwrap();
        let date = Utc.timestamp_millis_opt(1572213799747).unwrap();
        let result = convert(&date, &now, None);

        assert_eq!(
            result,
            vec![
                ConversionResult {
                    converted_text: date
                        .with_timezone(&Utc)
                        .to_rfc3339_opts(SecondsFormat::Millis, true),
                    format: ConversionFormat::Rfc3339Utc
                },
                ConversionResult {
                    converted_text: date
                        .with_timezone(&Local)
                        .to_rfc3339_opts(SecondsFormat::Millis, true),
                    format: ConversionFormat::Rfc3339Local
                },
                ConversionResult {
                    converted_text: String::from("1572213799747"),
                    format: ConversionFormat::EpochMillis
                },
                ConversionResult {
                    converted_text: String::from("2m 10s 1ms ago"),
                    format: ConversionFormat::DurationSince
                }
            ]
        );
    }

    #[test]
    fn rfc3339_offset() {
        let now = Utc.timestamp_millis_opt(1572213799749).unwrap();
        let date = Utc.timestamp_millis_opt(1572213799747).unwrap();
        let result = convert(&date, &now, None);

        assert_eq!(
            result,
            vec![
                ConversionResult {
                    converted_text: String::from("2019-10-27T22:03:19.747Z"),
                    format: ConversionFormat::Rfc3339Utc
                },
                ConversionResult {
                    converted_text: date
                        .with_timezone(&Local)
                        .to_rfc3339_opts(SecondsFormat::Millis, true),
                    format: ConversionFormat::Rfc3339Local
                },
                ConversionResult {
                    converted_text: String::from("1572213799747"),
                    format: ConversionFormat::EpochMillis
                },
                ConversionResult {
                    converted_text: String::from("2ms ago"),
                    format: ConversionFormat::DurationSince
                }
            ]
        );
    }

    #[test]
    fn duration_millis() {
        let now = Utc.timestamp_millis_opt(1572123676746).unwrap();
        let date = Utc.timestamp_millis_opt(1572213799747).unwrap();
        let result = convert(&date, &now, Some(DurationUnit::Milliseconds));

        assert_eq!(
            result.last().unwrap(),
            &ConversionResult {
                converted_text: String::from("in 90123001 ms"),
                format: ConversionFormat::DurationSinceUnits(DurationUnit::Milliseconds)
            }
        );
    }

    #[test]
    fn duration_seconds() {
        let now = Utc.timestamp_millis_opt(1572123676746).unwrap();
        let date = Utc.timestamp_millis_opt(1572213799747).unwrap();
        let result = convert(&date, &now, Some(DurationUnit::Seconds));

        assert_eq!(
            result.last().unwrap(),
            &ConversionResult {
                converted_text: String::from("in 90123.0 s"),
                format: ConversionFormat::DurationSinceUnits(DurationUnit::Seconds)
            }
        );
    }

    #[test]
    fn duration_minutes() {
        let now = Utc.timestamp_millis_opt(1572123676746).unwrap();
        let date = Utc.timestamp_millis_opt(1572213799747).unwrap();
        let result = convert(&date, &now, Some(DurationUnit::Minutes));

        assert_eq!(
            result.last().unwrap(),
            &ConversionResult {
                converted_text: String::from("in 1502.1 m"),
                format: ConversionFormat::DurationSinceUnits(DurationUnit::Minutes)
            }
        );
    }

    #[test]
    fn duration_hours() {
        let now = Utc.timestamp_millis_opt(1572123676746).unwrap();
        let date = Utc.timestamp_millis_opt(1572213799747).unwrap();
        let result = convert(&date, &now, Some(DurationUnit::Hours));

        assert_eq!(
            result.last().unwrap(),
            &ConversionResult {
                converted_text: String::from("in 25.0 h"),
                format: ConversionFormat::DurationSinceUnits(DurationUnit::Hours)
            }
        );
    }

    #[test]
    fn duration_days() {
        let now = Utc.timestamp_millis_opt(1572123676746).unwrap();
        let date = Utc.timestamp_millis_opt(1572213799747).unwrap();
        let result = convert(&date, &now, Some(DurationUnit::Days));

        assert_eq!(
            result.last().unwrap(),
            &ConversionResult {
                converted_text: String::from("in 1.0 days"),
                format: ConversionFormat::DurationSinceUnits(DurationUnit::Days)
            }
        );
    }

    #[test]
    fn duration_weeks() {
        let now = Utc.timestamp_millis_opt(1572123676746).unwrap();
        let date = Utc.timestamp_millis_opt(1572213799747).unwrap();
        let result = convert(&date, &now, Some(DurationUnit::Weeks));

        assert_eq!(
            result.last().unwrap(),
            &ConversionResult {
                converted_text: String::from("in 0.1 weeks"),
                format: ConversionFormat::DurationSinceUnits(DurationUnit::Weeks)
            }
        );
    }

    #[test]
    fn duration_fortnights() {
        let now = Utc.timestamp_millis_opt(1572123676746).unwrap();
        let date = Utc.timestamp_millis_opt(1572213799747).unwrap();
        let result = convert(&date, &now, Some(DurationUnit::Fortnights));

        assert_eq!(
            result.last().unwrap(),
            &ConversionResult {
                converted_text: String::from("in 0.1 fortnights"),
                format: ConversionFormat::DurationSinceUnits(DurationUnit::Fortnights)
            }
        );
    }
}
