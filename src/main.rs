#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use stgv::{cli, exec};
use structopt::StructOpt;

fn main() {
    let opt = cli::CLI::from_args();

    if let Err(err) = opt.validate() {
        eprintln!("{}", err);
        std::process::exit(1);
    }

    if let Err(err) = exec::run(opt) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}