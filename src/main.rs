use std::process;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "timeturner", about = "Manipulate date-time strings")]
struct Opt {
    input: Option<String>,
}

fn main() {
    let opt = Opt::from_args();

    if let Err(err) = timeturner::run(opt.input) {
        eprintln!("{}", err);
        process::exit(1);
    }
}
