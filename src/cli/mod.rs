pub mod convert;
pub mod validate;

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use crate::ForgeError;

#[derive(Parser)]
#[command(
    name = "forge",
    about = "FORGE — Framework for OSCAL Risk & Governance Execution",
    long_about = "FORGE — Framework for OSCAL Risk & Governance Execution\n\n\
        Converts security policy documents (Markdown) into machine-readable OSCAL\n\
        (Open Security Controls Assessment Language) JSON artifacts.\n\n\
        Pipeline: Ingest → Parse → Atomize → Map → Serialize → Validate\n\n\
        Supported output strategies:\n\
         catalog     OSCAL Catalog (groups, controls, statements)\n\
         component   OSCAL Component Definition (implemented requirements)",
    version,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output showing pipeline stage information
    #[arg(short, long, global = true, conflicts_with = "quiet")]
    pub verbose: bool,

    /// Suppress all non-essential output (only OSCAL artifact on stdout)
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Convert a policy document to OSCAL format
    Convert {
        /// Path to the input Markdown policy document (.md)
        input: PathBuf,

        /// Conversion strategy: 'catalog' for OSCAL Catalog, 'component' for Component Definition
        #[arg(long)]
        strategy: Strategy,

        /// Output format (currently only 'json' is supported)
        #[arg(long, default_value = "json")]
        format: OutputFormat,

        /// Write output to a file instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,

        /// Maximum input file size in MB (default: 10)
        #[arg(long, default_value = "10")]
        max_size: u64,

        /// Source profile/baseline reference for component strategy (e.g., path to OSCAL profile JSON)
        #[arg(long)]
        source_profile: Option<String>,
    },

    /// Validate an OSCAL artifact against JSON schemas
    Validate {
        /// Path to the OSCAL JSON artifact to validate
        input: PathBuf,

        /// Override auto-detected OSCAL model type (catalog or component-definition)
        #[arg(long, value_enum)]
        schema_type: Option<SchemaType>,

        /// Output format for validation results (text or json)
        #[arg(long, value_enum, default_value = "text")]
        format: ValidateOutputFormat,
    },
}

/// Output format for `forge validate` results (WI-20).
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum ValidateOutputFormat {
    /// Human-readable text output (default)
    Text,
    /// Machine-parseable JSON output (PRD S-1)
    Json,
}

/// CLI enum for --schema-type override.
#[derive(ValueEnum, Clone, Debug)]
pub enum SchemaType {
    Catalog,
    #[value(name = "component-definition")]
    ComponentDefinition,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Strategy {
    Catalog,
    Component,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Json,
    Xml,
    Yaml,
}

/// Execute the CLI command, dispatching to the appropriate subcommand handler.
///
/// # Errors
///
/// Returns `ForgeError` if the subcommand handler fails.
pub fn execute(cli: &Cli) -> Result<(), ForgeError> {
    match &cli.command {
        Commands::Convert { input, strategy, format, output, max_size, source_profile } => {
            convert::execute(
                input,
                strategy,
                format,
                output.as_deref(),
                *max_size,
                source_profile.as_deref(),
            )
        }
        Commands::Validate { input, schema_type, format } => {
            validate::execute(input, schema_type.as_ref(), format)
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn parse_convert_subcommand() {
        let cli = Cli::try_parse_from([
            "forge",
            "convert",
            "test.md",
            "--strategy",
            "catalog",
            "--format",
            "json",
        ])
        .unwrap();
        if let Commands::Convert { input, .. } = cli.command {
            assert_eq!(input, PathBuf::from("test.md"));
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn parse_validate_subcommand() {
        let cli = Cli::try_parse_from(["forge", "validate", "artifact.json"]).unwrap();
        if let Commands::Validate { input, schema_type, format } = cli.command {
            assert_eq!(input, PathBuf::from("artifact.json"));
            assert!(schema_type.is_none());
            assert_eq!(format, ValidateOutputFormat::Text);
        } else {
            panic!("Expected Validate command");
        }
    }

    #[test]
    fn parse_validate_with_schema_type_catalog() {
        let cli =
            Cli::try_parse_from(["forge", "validate", "artifact.json", "--schema-type", "catalog"])
                .unwrap();
        if let Commands::Validate { schema_type, .. } = cli.command {
            assert!(matches!(schema_type, Some(SchemaType::Catalog)));
        } else {
            panic!("Expected Validate command");
        }
    }

    #[test]
    fn parse_validate_with_schema_type_component_definition() {
        let cli = Cli::try_parse_from([
            "forge",
            "validate",
            "artifact.json",
            "--schema-type",
            "component-definition",
        ])
        .unwrap();
        if let Commands::Validate { schema_type, .. } = cli.command {
            assert!(matches!(schema_type, Some(SchemaType::ComponentDefinition)));
        } else {
            panic!("Expected Validate command");
        }
    }

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
            assert!(matches!(strategy, Strategy::Catalog));
            assert!(matches!(format, OutputFormat::Json));
            assert_eq!(output, Some(PathBuf::from("out.json")));
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn parse_convert_missing_strategy_fails() {
        let result = Cli::try_parse_from(["forge", "convert", "test.md", "--format", "json"]);
        assert!(result.is_err(), "Should fail when --strategy is omitted");
    }

    #[test]
    fn parse_convert_missing_format_defaults_to_json() {
        // T001 (EC-1): --format omitted → defaults to Json
        let cli = Cli::try_parse_from(["forge", "convert", "test.md", "--strategy", "catalog"])
            .expect("Should succeed when --format is omitted (defaults to json)");
        if let Commands::Convert { format, .. } = cli.command {
            assert!(matches!(format, OutputFormat::Json), "Default format should be Json");
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
