use crate::converting::ConversionResult;
use chrono::prelude::*;
use chrono_tz::Tz;
use clap::ValueEnum;

mod alfred;
mod converting;
mod parsing;

pub enum OutputMode {
    ValuePerLine,
    Alfred,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, ValueEnum)]
pub enum DurationUnit {
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
    Fortnights,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, ValueEnum)]
pub enum EpochUnit {
    #[value(alias = "s")]
    Seconds,
    #[value(alias = "ms")]
    Millis,
    #[value(alias = "us")]
    Micros,
    #[value(alias = "ns")]
    Nanos,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Utc,
    Zoned,
    Seconds,
    Millis,
    Nanos,
    Duration,
    #[value(skip)]
    DurationSinceUnits(DurationUnit),
}

pub const DEFAULT_OUTPUTS: &[OutputFormat] = &[
    OutputFormat::Utc,
    OutputFormat::Zoned,
    OutputFormat::Millis,
    OutputFormat::Duration,
];

#[derive(Debug, Clone, Copy)]
pub enum TimeZoneSpec {
    Named(Tz),
    Fixed(FixedOffset),
}

impl TimeZoneSpec {
    #[must_use]
    pub fn naive_to_utc(self, naive: NaiveDateTime) -> Option<DateTime<Utc>> {
        match self {
            TimeZoneSpec::Named(tz) => tz.from_local_datetime(&naive).single().map(|d| d.to_utc()),
            TimeZoneSpec::Fixed(off) => {
                off.from_local_datetime(&naive).single().map(|d| d.to_utc())
            }
        }
    }

    #[must_use]
    pub fn format_rfc3339_millis(self, dt: &DateTime<Utc>) -> String {
        match self {
            TimeZoneSpec::Named(tz) => dt
                .with_timezone(&tz)
                .to_rfc3339_opts(SecondsFormat::Millis, true),
            TimeZoneSpec::Fixed(off) => dt
                .with_timezone(&off)
                .to_rfc3339_opts(SecondsFormat::Millis, true),
        }
    }
}

/// Parses an IANA timezone name (e.g. `"America/New_York"`) or a fixed UTC offset
/// (e.g. `"+05:30"`, `"-08:00"`, `"+0530"`, `"-0800"`, `"Z"`, `"UTC"`) into a `TimeZoneSpec`.
///
/// # Errors
///
/// Returns an error string if the input is not a recognized IANA name or fixed offset.
///
/// # Panics
///
/// Never panics.
pub fn parse_timezone_spec(s: &str) -> Result<TimeZoneSpec, String> {
    if s.eq_ignore_ascii_case("Z") || s.eq_ignore_ascii_case("UTC") {
        return Ok(TimeZoneSpec::Fixed(FixedOffset::east_opt(0).unwrap()));
    }
    if let Some(offset) = try_parse_fixed_offset(s) {
        return Ok(TimeZoneSpec::Fixed(offset));
    }
    s.parse::<Tz>()
        .map(TimeZoneSpec::Named)
        .map_err(|_| format!("Unknown timezone: {s}"))
}

fn try_parse_fixed_offset(s: &str) -> Option<FixedOffset> {
    let (sign, rest) = match s.chars().next()? {
        '+' => (1i32, &s[1..]),
        '-' => (-1i32, &s[1..]),
        _ => return None,
    };
    let (hours, minutes) = if rest.contains(':') {
        let mut parts = rest.splitn(2, ':');
        let h: i32 = parts.next()?.parse().ok()?;
        let m: i32 = parts.next()?.parse().ok()?;
        (h, m)
    } else if rest.len() == 4 {
        let h: i32 = rest[..2].parse().ok()?;
        let m: i32 = rest[2..].parse().ok()?;
        (h, m)
    } else {
        return None;
    };
    let total_secs = sign * (hours * 3600 + minutes * 60);
    FixedOffset::east_opt(total_secs)
}

/// Takes an optional input and prints conversions to different date-time formats.
/// If an input string is not given, then `now` is used.
/// If the input format cannot be handled, a string suitable for display to the user
/// is given as the error result.
///
/// # Errors
///
/// Will return an error string if `input` cannot be parsed to a date.
pub fn run(
    input: Option<&str>,
    output_mode: &OutputMode,
    extra_duration_unit: Option<DurationUnit>,
    epoch_unit: Option<EpochUnit>,
    input_timezone: Option<TimeZoneSpec>,
    output_timezone: Option<TimeZoneSpec>,
    outputs: Option<&[OutputFormat]>,
) -> Result<(), &'static str> {
    let outputs = outputs.unwrap_or(DEFAULT_OUTPUTS);
    let parsed_input = parsing::parse_input(input, epoch_unit, input_timezone)?;
    let conversion_results = converting::convert(
        &parsed_input,
        &Utc::now(),
        outputs,
        output_timezone,
        extra_duration_unit,
    );

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_timezone_spec_iana() {
        assert!(matches!(
            parse_timezone_spec("America/New_York"),
            Ok(TimeZoneSpec::Named(_))
        ));
    }

    #[test]
    fn parse_timezone_spec_z() {
        let Ok(TimeZoneSpec::Fixed(off)) = parse_timezone_spec("Z") else {
            panic!("expected Fixed");
        };
        assert_eq!(off.utc_minus_local(), 0);
    }

    #[test]
    fn parse_timezone_spec_utc_lowercase() {
        assert!(matches!(
            parse_timezone_spec("utc"),
            Ok(TimeZoneSpec::Fixed(_))
        ));
    }

    #[test]
    fn parse_timezone_spec_positive_colon() {
        let Ok(TimeZoneSpec::Fixed(off)) = parse_timezone_spec("+05:30") else {
            panic!("expected Fixed");
        };
        assert_eq!(off.utc_minus_local(), -(5 * 3600 + 30 * 60));
    }

    #[test]
    fn parse_timezone_spec_negative_colon() {
        let Ok(TimeZoneSpec::Fixed(off)) = parse_timezone_spec("-08:00") else {
            panic!("expected Fixed");
        };
        assert_eq!(off.utc_minus_local(), 8 * 3600);
    }

    #[test]
    fn parse_timezone_spec_no_colon() {
        let Ok(TimeZoneSpec::Fixed(off)) = parse_timezone_spec("+0530") else {
            panic!("expected Fixed");
        };
        assert_eq!(off.utc_minus_local(), -(5 * 3600 + 30 * 60));
    }

    #[test]
    fn parse_timezone_spec_negative_no_colon() {
        let Ok(TimeZoneSpec::Fixed(off)) = parse_timezone_spec("-0800") else {
            panic!("expected Fixed");
        };
        assert_eq!(off.utc_minus_local(), 8 * 3600);
    }

    #[test]
    fn parse_timezone_spec_invalid() {
        assert!(parse_timezone_spec("Nope/Nowhere").is_err());
    }
}
