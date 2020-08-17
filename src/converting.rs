use super::parsing::DateTimeFormat;
use crate::parsing::ParsedInput;
use chrono::prelude::*;
use humantime::format_duration;
use std::convert::TryInto;
use std::time::Duration;

#[derive(PartialEq, Debug)]
pub enum ConversionFormat {
    Rfc3339Utc,
    Rfc3339Local,
    EpochMillis,
    DurationSince,
}

#[derive(PartialEq, Debug)]
pub struct ConversionResult {
    pub converted_text: String,
    pub format: ConversionFormat,
}

pub fn convert(parsed_input: &ParsedInput, now: DateTime<Utc>) -> Vec<ConversionResult> {
    let mut results = Vec::new();

    if parsed_input.input_zone != Some(FixedOffset::west(0)) {
        results.push(ConversionResult {
            converted_text: parsed_input
                .value
                .to_rfc3339_opts(SecondsFormat::Millis, true),
            format: ConversionFormat::Rfc3339Utc,
        });
    }

    if parsed_input.input_zone != Some(parsed_input.value.with_timezone(&Local).offset().fix()) {
        results.push(ConversionResult {
            converted_text: parsed_input
                .value
                .with_timezone(&Local)
                .to_rfc3339_opts(SecondsFormat::Millis, true),
            format: ConversionFormat::Rfc3339Local,
        });
    }

    if parsed_input.input_format != DateTimeFormat::EpochMillis {
        results.push(ConversionResult {
            converted_text: parsed_input.value.timestamp_millis().to_string(),
            format: ConversionFormat::EpochMillis,
        });
    }

    results.push(ConversionResult {
        converted_text: duration_since(parsed_input.value, now),
        format: ConversionFormat::DurationSince,
    });

    results
}

pub fn duration_since(input: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let difference_millis = now.timestamp_millis() - input.timestamp_millis();

    let in_future = difference_millis.is_negative();
    let difference_millis = difference_millis.abs();

    let duration = Duration::from_millis(difference_millis.try_into().unwrap());
    let duration_format = format_duration(duration);

    if in_future {
        format!("in {}", duration_format)
    } else {
        format!("{} ago", duration_format)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]

    use super::*;

    #[test]
    fn missing_input() {
        let now = Utc.timestamp_millis(1572303922748);
        let date = Utc.timestamp_millis(1572213799747);
        let result = convert(
            &ParsedInput {
                input_format: DateTimeFormat::Missing,
                input_zone: None,
                value: date,
            },
            now,
        );

        // Should include all output formats
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
        let now = Utc.timestamp_millis(1572123676746);
        let date = Utc.timestamp_millis(1572213799747);
        let result = convert(
            &ParsedInput {
                input_format: DateTimeFormat::EpochMillis,
                input_zone: None,
                value: date,
            },
            now,
        );

        // Should skip epoch-millis output format
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
                    converted_text: String::from("in 1day 1h 2m 3s 1ms"),
                    format: ConversionFormat::DurationSince
                }
            ]
        );
    }

    #[test]
    fn rfc3339_utc() {
        let now = Utc.timestamp_millis(1572213929748);
        let date = Utc.timestamp_millis(1572213799747);
        let result = convert(
            &ParsedInput {
                input_format: DateTimeFormat::Rfc3339,
                input_zone: Some(FixedOffset::west(0)),
                value: date,
            },
            now,
        );

        // Should skip RFC3339 in UTC
        assert_eq!(
            result,
            vec![
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
        let now = Utc.timestamp_millis(1572213799749);
        let date = Utc.timestamp_millis(1572213799747);
        let result = convert(
            &ParsedInput {
                input_format: DateTimeFormat::Rfc3339,
                input_zone: Some(date.with_timezone(&Local).offset().fix()),
                value: date,
            },
            now,
        );

        // Should skip RFC3339 in Local
        assert_eq!(
            result,
            vec![
                ConversionResult {
                    converted_text: String::from("2019-10-27T22:03:19.747Z"),
                    format: ConversionFormat::Rfc3339Utc
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
}
