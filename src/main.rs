use std::process::ExitCode;

use clap::Parser;
use forge::cli::{self, Cli};
use forge::error::ForgeError;
use forge::exit_code;

fn main() -> ExitCode {
    let cli = Cli::parse();

    let filter = if cli.verbose {
        "debug"
    } else if cli.quiet {
        "error"
    } else {
        "warn"
    };
    tracing_subscriber::fmt().with_env_filter(filter).with_writer(std::io::stderr).init();

    match cli::execute(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(
            ForgeError::DiffHasChanges
            | ForgeError::DriftDetected
            | ForgeError::MigrationHasChanges
            | ForgeError::MappingReviewRequired,
        ) => ExitCode::from(1u8),
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::from(exit_code(&e))
        }
    }
}
