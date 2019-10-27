use chrono::prelude::*;

#[derive(PartialEq)]
enum DateTimeFormat {
    Missing,
    EpochMillis,
    Rfc3339,
}

struct ParsedInput {
    input_format: DateTimeFormat,
    value: DateTime<Utc>,
}

impl DateTimeFormat {
    fn parse(&self, input: &str) -> Option<ParsedInput> {
        match self {
            DateTimeFormat::EpochMillis => parse_from_epoch_millis(input),
            DateTimeFormat::Rfc3339 => parse_from_rfc3339(input),
            DateTimeFormat::Missing => None,
        }
    }
}

fn parse_from_epoch_millis(input: &str) -> Option<ParsedInput> {
    input
        .parse::<i64>()
        .ok()
        .and_then(|e| Utc.timestamp_millis_opt(e).single())
        .map(|d| ParsedInput {
            input_format: DateTimeFormat::EpochMillis,
            value: d,
        })
}

fn parse_from_rfc3339(input: &str) -> Option<ParsedInput> {
    DateTime::parse_from_rfc3339(input)
        .ok()
        .map(|d| ParsedInput {
            input_format: DateTimeFormat::Rfc3339,
            value: d.with_timezone(&Utc),
        })
}

fn parse_input(input: Option<String>) -> Result<ParsedInput, &'static str> {
    input
        .map(|i| {
            DateTimeFormat::EpochMillis
                .parse(&i)
                .or(DateTimeFormat::Rfc3339.parse(&i))
                .ok_or("Input format not recognized")
        })
        .unwrap_or_else(|| {
            Ok(ParsedInput {
                input_format: DateTimeFormat::Missing,
                value: Utc::now(),
            })
        })
}

pub fn run(input: Option<String>) -> Result<(), &'static str> {
    let parsed_input = parse_input(input)?;

    if parsed_input.input_format != DateTimeFormat::Rfc3339 {
        println!("{}", parsed_input.value.to_rfc3339());
        println!("{}", parsed_input.value.with_timezone(&Local).to_rfc3339());
    }

    if parsed_input.input_format != DateTimeFormat::EpochMillis {
        println!("{}", parsed_input.value.timestamp_millis());
    }

    Ok(())
}
