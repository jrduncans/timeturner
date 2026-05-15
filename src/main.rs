use std::process;

use clap::Parser;
use timeturner::DurationUnit;
use timeturner::EpochUnit;
use timeturner::OutputMode;

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

    input: Option<String>,
}

fn main() {
    let opt: Opt = Parser::parse();

    if let Err(err) = timeturner::run(
        opt.input.as_deref(),
        &output_mode(&opt),
        opt.duration_unit,
        opt.epoch_unit,
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
