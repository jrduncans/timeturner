mod converting;
mod parsing;

/// Takes an optional input and prints conversions to different date-time formats.
/// If an input string is not given, then `now` is used.
/// If the input format cannot be handled, a string suitable for display to the user
/// is given as the error result.
pub fn run(input: Option<String>) -> Result<(), &'static str> {
    let parsed_input = crate::parsing::parse_input(input)?;
    let conversion_results = crate::converting::convert(parsed_input);

    for conversion_result in &conversion_results {
        println!("{}", conversion_result.converted_text);
    }

    Ok(())
}
