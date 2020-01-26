use super::parsing::DateTimeFormat;
use crate::parsing::ParsedInput;
use chrono::prelude::*;

pub enum ConversionFormat {
    Rfc3339Utc,
    Rfc3339Local,
    EpochMillis,
}

pub struct ConversionResult {
    pub converted_text: String,
    pub format: ConversionFormat,
}

pub fn convert(parsed_input: ParsedInput) -> Vec<ConversionResult> {
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
            converted_text: format!("{}", parsed_input.value.timestamp_millis()),
            format: ConversionFormat::EpochMillis,
        });
    }

    results
}
