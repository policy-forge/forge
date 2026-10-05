use std::process::ExitCode;

use clap::Parser;
use forge::cli::{self, Cli};
use forge::error::ForgeError;
use forge::exit_code;
use tracing_subscriber::EnvFilter;

/// Fixed Windows CLI stack reservation; domain input and work limits remain unchanged.
#[cfg(windows)]
const WINDOWS_CLI_STACK_BYTES: usize = 8 * 1024 * 1024;

/// Start CLI parsing and execution on a bounded Windows worker before building the command tree.
#[cfg(windows)]
fn main() -> ExitCode {
    let worker = match std::thread::Builder::new()
        .name("forge-cli".to_string())
        .stack_size(WINDOWS_CLI_STACK_BYTES)
        .spawn(run_cli)
    {
        Ok(worker) => worker,
        Err(_) => {
            eprintln!("Error: CLI startup failed");
            return ExitCode::FAILURE;
        }
    };
    match worker.join() {
        Ok(code) => code,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

/// Keep CLI parsing and execution on the original main thread outside Windows.
#[cfg(not(windows))]
fn main() -> ExitCode {
    run_cli()
}

/// Dispatch the selected CLI and preserve valid action exits without printing an error.
#[inline(never)]
fn run_cli() -> ExitCode {
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
    if matches!(
        &cli.command,
        cli::Commands::Workspace { .. } | cli::Commands::Mcp { .. } | cli::Commands::Review { .. }
    ) {
        // Workspace, MCP and private review data must never enter process logs, even
        // with RUST_LOG=trace or --verbose. Bootstrap uses explicit safe output.
        tracing::subscriber::set_global_default(tracing::subscriber::NoSubscriber::default())
            .expect("private workflow installs the process subscriber once");
    } else {
        tracing_subscriber::fmt().with_env_filter(filter).with_writer(std::io::stderr).init();
    }

    match cli::execute(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(
            ForgeError::DiffHasChanges
            | ForgeError::DriftDetected
            | ForgeError::MigrationHasChanges
            | ForgeError::MappingReviewRequired
            | ForgeError::LinkageActionRequired
            | ForgeError::LifecycleActionRequired
            | ForgeError::ApplicabilityReviewRequired
            | ForgeError::FrameworkReviewRequired
            | ForgeError::AuthoringActionRequired
            | ForgeError::AssessmentResultsReviewRequired
            | ForgeError::PoamActionRequired,
        ) => ExitCode::from(1u8),
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::from(exit_code(&e))
        }
    }
}
