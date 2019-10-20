use chrono::prelude::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "timeturner", about = "Manipulate date-time strings")]
struct Opt {
    input: Option<String>,
}

fn main() {
    let opt = Opt::from_args();
    let date_time = opt
        .input
        .map(|i| {
            i.parse::<i64>()
                .map(|e| Utc.timestamp_millis_opt(e).unwrap())
                .unwrap()
        })
        .unwrap_or_else(|| Utc::now());
    println!("{}", date_time.to_rfc3339());
    println!("{}", date_time.with_timezone(&Local).to_rfc3339());
}
