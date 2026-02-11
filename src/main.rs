use std::process;

use clap::Parser;
use forge::cli::{self, Cli};

fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli::execute(&cli) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
