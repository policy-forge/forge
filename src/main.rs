use std::process::ExitCode;

use clap::Parser;
use forge::cli::{self, Cli};
use forge::error::ForgeError;
use forge::exit_code;
use tracing_subscriber::EnvFilter;

fn main() -> ExitCode {
    let cli = Cli::parse();

    let default_filter = if cli.verbose {
        "debug"
    } else if cli.quiet {
        "error"
    } else {
        "warn"
    };
    // A valid RUST_LOG filter overrides the flag-derived default for per-module control.
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));
    tracing_subscriber::fmt().with_env_filter(filter).with_writer(std::io::stderr).init();

    match cli::execute(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(
            ForgeError::DiffHasChanges
            | ForgeError::DriftDetected
            | ForgeError::MigrationHasChanges
            | ForgeError::MappingReviewRequired
            | ForgeError::LifecycleActionRequired
            | ForgeError::ApplicabilityReviewRequired
            | ForgeError::FrameworkReviewRequired
            | ForgeError::AssessmentResultsReviewRequired,
        ) => ExitCode::from(1u8),
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::from(exit_code(&e))
        }
    }
}
