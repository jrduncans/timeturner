use super::OutputFormat;
use super::converting::ConversionResult;
use serde::Serialize;

#[derive(Serialize)]
struct Item {
    uid: String,
    title: String,
    subtitle: String,
    arg: String,
}

#[derive(Serialize)]
struct Alfred {
    items: Vec<Item>,
}

pub fn output_json(conversion_results: &[ConversionResult]) -> String {
    let items: Vec<_> = conversion_results
        .iter()
        .map(|conversion_result| Item {
            uid: match &conversion_result.format {
                OutputFormat::Utc => String::from("utc"),
                OutputFormat::Zoned => String::from("zoned"),
                OutputFormat::Seconds => String::from("seconds"),
                OutputFormat::Millis => String::from("millis"),
                OutputFormat::Nanos => String::from("nanos"),
                OutputFormat::Duration => String::from("duration"),
                OutputFormat::DurationSinceUnits(duration_unit) => {
                    format!("duration_since_{duration_unit:?}").to_lowercase()
                }
            },
            title: conversion_result.converted_text.clone(),
            subtitle: match &conversion_result.format {
                OutputFormat::Utc => String::from("RFC3339 - UTC"),
                OutputFormat::Zoned => String::from("RFC3339 - Zoned"),
                OutputFormat::Seconds => String::from("Epoch Seconds"),
                OutputFormat::Millis => String::from("Epoch Millis"),
                OutputFormat::Nanos => String::from("Epoch Nanoseconds"),
                OutputFormat::Duration => String::from("Duration"),
                OutputFormat::DurationSinceUnits(duration_unit) => {
                    format!("Duration {duration_unit:?}")
                }
            },
            arg: conversion_result.converted_text.clone(),
        })
        .collect();

    serde_json::to_string(&Alfred { items }).unwrap()
}
