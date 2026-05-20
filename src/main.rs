use std::process;

use clap::Parser;
use timeturner::DurationUnit;
use timeturner::EpochUnit;
use timeturner::OutputFormat;
use timeturner::OutputMode;
use timeturner::TimeZoneSpec;

#[derive(Debug, Parser)]
#[command(name = "timeturner", about = "Manipulate date-time strings", version)]
struct Opt {
    #[arg(long, help = "Output in JSON for Alfred Workflow integration")]
    alfred: bool,

    #[arg(short, long)]
    duration_unit: Option<DurationUnit>,

    #[arg(
        short = 'u',
        long,
        help = "Force epoch input to be interpreted in the given unit (seconds, millis/ms, micros/us, nanos/ns)"
    )]
    epoch_unit: Option<EpochUnit>,

    #[arg(
        long,
        allow_hyphen_values = true,
        value_parser = timeturner::parse_timezone_spec,
        help = "Timezone to assume for inputs lacking explicit zone info (IANA name or fixed offset, e.g. America/New_York, -05:00)"
    )]
    input_timezone: Option<TimeZoneSpec>,

    #[arg(
        long,
        allow_hyphen_values = true,
        value_parser = timeturner::parse_timezone_spec,
        help = "Timezone used for the zoned RFC3339 output (defaults to system local; IANA name or fixed offset)"
    )]
    output_timezone: Option<TimeZoneSpec>,

    #[arg(
        short = 'o',
        long,
        value_delimiter = ',',
        help = "Comma-separated list of outputs to produce (default: utc,zoned,millis,duration)"
    )]
    outputs: Option<Vec<OutputFormat>>,

    input: Option<String>,
}

fn main() {
    let opt: Opt = Parser::parse();

    if let Err(err) = timeturner::run(
        opt.input.as_deref(),
        &output_mode(&opt),
        opt.duration_unit,
        opt.epoch_unit,
        opt.input_timezone,
        opt.output_timezone,
        opt.outputs.as_deref(),
    ) {
        eprintln!("{err}");
        process::exit(1);
    }
}

fn output_mode(opt: &Opt) -> OutputMode {
    if opt.alfred {
        OutputMode::Alfred
    } else {
        OutputMode::ValuePerLine
    }
}
