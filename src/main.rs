use std::process;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "timeturner", about = "Manipulate date-time strings")]
struct Opt {
    input: Option<String>,
}

fn main() {
    let opt = Opt::from_args();

    timeturner::run(opt.input).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });
}
