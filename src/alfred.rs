use super::ConversionResult;
use super::converting::ConversionFormat;
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
                ConversionFormat::Rfc3339Utc => String::from("rfc3339_utc"),
                ConversionFormat::Rfc3339Local => String::from("rfc3339_local"),
                ConversionFormat::EpochMillis => String::from("epoch_millis"),
                ConversionFormat::DurationSince => String::from("duration_since"),
                ConversionFormat::DurationSinceUnits(duration_unit) => {
                    format!(
                        "duration_since_{}",
                        format!("{duration_unit:?}").to_lowercase()
                    )
                }
            },
            title: conversion_result.converted_text.clone(),
            subtitle: match &conversion_result.format {
                ConversionFormat::Rfc3339Utc => String::from("RFC3339 - UTC"),
                ConversionFormat::Rfc3339Local => String::from("RFC3339 - Local"),
                ConversionFormat::EpochMillis => String::from("Epoch Millis"),
                ConversionFormat::DurationSince => String::from("Duration"),
                ConversionFormat::DurationSinceUnits(duration_unit) => {
                    format!("Duration {duration_unit:?}")
                }
            },
            arg: conversion_result.converted_text.clone(),
        })
        .collect();

    serde_json::to_string(&Alfred { items }).unwrap()
}
