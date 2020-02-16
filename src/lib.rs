use crate::converting::ConversionResult;

mod alfred;
mod converting;
mod parsing;

pub enum OutputMode {
    ValuePerLine,
    Alfred,
}

/// Takes an optional input and prints conversions to different date-time formats.
/// If an input string is not given, then `now` is used.
/// If the input format cannot be handled, a string suitable for display to the user
/// is given as the error result.
///
/// # Errors
///
/// Will return an error string if `input` cannot be parsed to a date.
pub fn run(input: &Option<String>, output_mode: &OutputMode) -> Result<(), &'static str> {
    let parsed_input = crate::parsing::parse_input(input)?;
    let conversion_results = crate::converting::convert(&parsed_input);

    match output_mode {
        OutputMode::ValuePerLine => output_value_per_line(&conversion_results),
        OutputMode::Alfred => println!("{}", crate::alfred::output_json(&conversion_results)),
    }

    Ok(())
}

fn output_value_per_line(conversion_results: &[ConversionResult]) {
    for conversion_result in conversion_results {
        println!("{}", conversion_result.converted_text);
    }
}
