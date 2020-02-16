use super::parsing::DateTimeFormat;
use crate::parsing::ParsedInput;
use chrono::prelude::*;

#[derive(PartialEq, Debug)]
pub enum ConversionFormat {
    Rfc3339Utc,
    Rfc3339Local,
    EpochMillis,
}

#[derive(PartialEq, Debug)]
pub struct ConversionResult {
    pub converted_text: String,
    pub format: ConversionFormat,
}

pub fn convert(parsed_input: &ParsedInput) -> Vec<ConversionResult> {
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

    results
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]

    use super::*;

    #[test]
    fn missing_input() {
        let date = Utc.timestamp_millis(1572213799747);
        let result = convert(&ParsedInput {
            input_format: DateTimeFormat::Missing,
            input_zone: None,
            value: date,
        });

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
                }
            ]
        );
    }

    #[test]
    fn epoch_millis_input() {
        let date = Utc.timestamp_millis(1572213799747);
        let result = convert(&ParsedInput {
            input_format: DateTimeFormat::EpochMillis,
            input_zone: None,
            value: date,
        });

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
                }
            ]
        );
    }

    #[test]
    fn rfc3339_utc() {
        let date = Utc.timestamp_millis(1572213799747);
        let result = convert(&ParsedInput {
            input_format: DateTimeFormat::Rfc3339,
            input_zone: Some(FixedOffset::west(0)),
            value: date,
        });

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
                }
            ]
        );
    }

    #[test]
    fn rfc3339_offset() {
        let date = Utc.timestamp_millis(1572213799747);
        let result = convert(&ParsedInput {
            input_format: DateTimeFormat::Rfc3339,
            input_zone: Some(date.with_timezone(&Local).offset().fix()),
            value: date,
        });

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
                }
            ]
        );
    }
}
