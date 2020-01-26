mod parsing;

use chrono::prelude::*;
use parsing::DateTimeFormat;

/// Takes an optional input and prints conversions to different date-time formats.
/// If an input string is not given, then `now` is used.
/// If the input format cannot be handled, a string suitable for display to the user
/// is given as the error result.
pub fn run(input: Option<String>) -> Result<(), &'static str> {
    let parsed_input = crate::parsing::parse_input(input)?;

    if parsed_input.input_zone != Some(FixedOffset::west(0)) {
        println!("{}", parsed_input.value.to_rfc3339_opts(SecondsFormat::Millis, true));
    }

    if parsed_input.input_zone != Some(parsed_input.value.with_timezone(&Local).offset().fix()) {
        println!("{}", parsed_input.value.with_timezone(&Local).to_rfc3339_opts(SecondsFormat::Millis, true));
    }

    if parsed_input.input_format != DateTimeFormat::EpochMillis {
        println!("{}", parsed_input.value.timestamp_millis());
    }

    Ok(())
}
