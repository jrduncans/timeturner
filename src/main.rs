use structopt::StructOpt;
use chrono::prelude::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "timeturner", about = "Manipulate date-time strings")]
struct Opt {
    input: String
}

fn main() {
    let opt = Opt::from_args();
    if let Ok(epoch_millis) = opt.input.parse::<i64>() {
        let date = Local.timestamp_millis(epoch_millis);
        println!("{}", date.to_rfc3339());
    }
}
