pub mod convert;
pub mod validate;

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use crate::ForgeError;

#[derive(Parser)]
#[command(
    name = "forge",
    about = "FORGE — Framework for OSCAL Risk & Governance Execution",
    version,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true, conflicts_with = "quiet")]
    pub verbose: bool,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Convert a policy document to OSCAL format
    Convert {
        /// Path to the input policy document
        input: PathBuf,

        /// Conversion strategy
        #[arg(long)]
        strategy: Option<Strategy>,

        /// Output format
        #[arg(long, default_value = "json")]
        format: OutputFormat,

        /// Output file path
        #[arg(long)]
        output: Option<PathBuf>,

        /// Maximum file size in MB
        #[arg(long, default_value = "10")]
        max_size: u64,
    },

    /// Validate an OSCAL artifact against schemas
    Validate {
        /// Path to the OSCAL artifact to validate
        input: PathBuf,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Strategy {
    Catalog,
    Component,
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum OutputFormat {
    #[default]
    Json,
    Xml,
    Yaml,
}

/// Dispatches the top-level CLI to the selected subcommand handler.
///
/// For the current `Cli` command, forwards parsed arguments to the corresponding
/// subcommand implementation (`convert` or `validate`) and returns its result.
///
/// # Examples
///
/// ```
/// use clap::Parser;
/// let cli = Cli::parse_from(&["forge", "validate", "artifact.json"]);
/// execute(&cli).unwrap();
/// ```
///
/// # Errors
///
/// Returns `ForgeError` if the selected subcommand handler fails.
pub fn execute(cli: &Cli) -> Result<(), ForgeError> {
    match &cli.command {
        Commands::Convert { input, strategy, format, output, max_size } => {
            convert::execute(input, strategy.as_ref(), format, output.as_deref(), *max_size)
        }
        Commands::Validate { input } => validate::execute(input),
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn parse_convert_subcommand() {
        let cli = Cli::try_parse_from(["forge", "convert", "test.md"]).unwrap();
        if let Commands::Convert { input, .. } = cli.command {
            assert_eq!(input, PathBuf::from("test.md"));
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn parse_validate_subcommand() {
        let cli = Cli::try_parse_from(["forge", "validate", "artifact.json"]).unwrap();
        if let Commands::Validate { input } = cli.command {
            assert_eq!(input, PathBuf::from("artifact.json"));
        } else {
            panic!("Expected Validate command");
        }
    }

    /// Ensures the CLI correctly parses the `convert` subcommand when all options are provided.
    ///
    /// # Examples
    ///
    /// ```
    /// let cli = Cli::try_parse_from([
    ///     "forge",
    ///     "convert",
    ///     "test.md",
    ///     "--strategy",
    ///     "catalog",
    ///     "--format",
    ///     "json",
    ///     "--output",
    ///     "out.json",
    /// ]).unwrap();
    ///
    /// if let Commands::Convert { input, strategy, format, output, .. } = cli.command {
    ///     assert_eq!(input, PathBuf::from("test.md"));
    ///     assert!(matches!(strategy, Some(Strategy::Catalog)));
    ///     assert!(matches!(format, OutputFormat::Json));
    ///     assert_eq!(output, Some(PathBuf::from("out.json")));
    /// } else {
    ///     panic!("Expected Convert command");
    /// }
    /// ```
    #[test]
    fn parse_convert_with_all_options() {
        let cli = Cli::try_parse_from([
            "forge",
            "convert",
            "test.md",
            "--strategy",
            "catalog",
            "--format",
            "json",
            "--output",
            "out.json",
        ])
        .unwrap();

        if let Commands::Convert { input, strategy, format, output, .. } = cli.command {
            assert_eq!(input, PathBuf::from("test.md"));
            assert!(matches!(strategy, Some(Strategy::Catalog)));
            assert!(matches!(format, OutputFormat::Json));
            assert_eq!(output, Some(PathBuf::from("out.json")));
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn verbose_and_quiet_conflict() {
        let result = Cli::try_parse_from(["forge", "-v", "-q", "convert", "test.md"]);
        assert!(result.is_err(), "Expected error when both --verbose and --quiet are provided");
    }
}