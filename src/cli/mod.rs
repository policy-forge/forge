pub mod convert;
pub mod diff;
pub mod export;
pub mod profile;
pub mod resolve;
pub mod trace;
pub mod validate;

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

pub use crate::types::{OutputFormat, Strategy};

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
        /// Path(s) to the input Markdown policy document(s) (.md)
        #[arg(num_args = 1..)]
        input: Vec<PathBuf>,

        /// Conversion strategy: 'catalog' for OSCAL Catalog, 'component' for Component Definition
        #[arg(long)]
        strategy: Strategy,

        /// Output format: json, xml, or yaml
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

        /// Number of parallel jobs for batch conversion (0 = auto, 1 = sequential)
        #[arg(long, default_value = "0", value_parser = clap::value_parser!(u16).range(0..=256))]
        jobs: u16,

        /// Optional baseline policy used to compare stable IDs and emit substantive-change warnings.
        ///
        /// When provided, FORGE computes stable IDs for both documents and emits a warning if
        /// matching requirement locations have different stable IDs (indicating non-whitespace
        /// substantive text changes).
        #[arg(long)]
        stable_id_baseline: Option<PathBuf>,

        /// Print a conversion summary dashboard to stderr after conversion
        #[arg(long)]
        summary: bool,
    },

    /// Export an OSCAL artifact to a different format
    Export {
        /// Path to the input OSCAL artifact (JSON, XML, or YAML)
        input: PathBuf,

        /// Target output format
        #[arg(long, value_enum)]
        format: OutputFormat,

        /// Write output to a file instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,
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

    /// Resolve an OSCAL Profile into a flat Catalog baseline via oscal-cli
    Resolve {
        /// Path to the OSCAL Profile JSON file
        input: Option<PathBuf>,

        /// Output file path (default: <input-stem>-resolved.json)
        #[arg(long)]
        output: Option<PathBuf>,

        /// Report oscal-cli detection status and exit
        #[arg(long)]
        check: bool,

        /// Timeout for oscal-cli execution in seconds (default: 60)
        #[arg(long, default_value = "60")]
        timeout: u64,

        /// Explicit path to oscal-cli binary (overrides PATH search)
        #[arg(long)]
        oscal_cli_path: Option<PathBuf>,
    },

    /// Show traceability between OSCAL elements and source policy locations.
    ///
    /// The report inherits the sensitivity classification of the source policy.
    /// Treat the output with the same access controls as the source document.
    Trace {
        /// Path to the OSCAL artifact (JSON)
        artifact: PathBuf,

        /// Path to the source policy file
        #[arg(long)]
        source: PathBuf,

        /// Write output to a file instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Compare two OSCAL artifacts and show differences
    Diff {
        /// Path to the old OSCAL artifact (JSON)
        old_artifact: PathBuf,

        /// Path to the new OSCAL artifact (JSON)
        new_artifact: PathBuf,
    },

    /// Generate an OSCAL Profile by selecting controls from a source Catalog
    Profile {
        /// Path to the source Catalog file (OSCAL Catalog JSON)
        #[arg(long)]
        catalog: PathBuf,

        /// Comma-separated control IDs to include (mutually exclusive with --exclude)
        #[arg(long, conflicts_with = "exclude")]
        include: Option<String>,

        /// Comma-separated control IDs to exclude (mutually exclusive with --include)
        #[arg(long, conflicts_with = "include")]
        exclude: Option<String>,

        /// Output format: json, xml, or yaml
        #[arg(long, default_value = "json")]
        format: OutputFormat,

        /// Write output to a file instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,

        /// Set a parameter value override in the Profile's modify section (WI-31).
        /// Takes exactly two arguments: `<PARAM_ID>` `<VALUE>`. Repeatable.
        #[arg(long = "set-param", num_args = 2, action = clap::ArgAction::Append, value_names = ["PARAM_ID", "VALUE"])]
        set_params: Vec<String>,
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

/// Execute the CLI command, dispatching to the appropriate subcommand handler.
///
/// # Errors
///
/// Returns `ForgeError` if the subcommand handler fails.
pub fn execute(cli: &Cli) -> Result<(), ForgeError> {
    match &cli.command {
        Commands::Convert {
            input,
            strategy,
            format,
            output,
            max_size,
            source_profile,
            jobs,
            stable_id_baseline,
            summary,
        } => {
            if input.is_empty() {
                return Err(ForgeError::BatchConversion("No input files provided".to_string()));
            }
            let opts = convert::ConvertOptions {
                input: &input[0],
                strategy,
                format,
                output: output.as_deref(),
                max_size: *max_size,
                source_profile: source_profile.as_deref(),
                stable_id_baseline: stable_id_baseline.as_deref(),
                summary: *summary,
                quiet: cli.quiet,
                jobs: *jobs,
            };
            convert::execute_dispatch(input, &opts)
        }
        Commands::Export { input, format, output } => {
            export::execute(input, format, output.as_deref())
        }
        Commands::Validate { input, schema_type, format } => {
            validate::execute(input, schema_type.as_ref(), format)
        }
        Commands::Resolve { input, output, check, timeout, oscal_cli_path } => resolve::execute(
            input.as_deref(),
            output.as_deref(),
            *check,
            *timeout,
            oscal_cli_path.as_deref(),
        ),
        Commands::Diff { old_artifact, new_artifact } => {
            diff::execute(old_artifact, new_artifact).and_then(|has_changes| {
                if has_changes { Err(ForgeError::DiffHasChanges) } else { Ok(()) }
            })
        }
        Commands::Trace { artifact, source, output } => {
            trace::execute(artifact, source, output.as_deref())
        }
        Commands::Profile { catalog, include, exclude, format, output, set_params } => {
            profile::execute(
                catalog,
                include.as_deref(),
                exclude.as_deref(),
                format,
                output.as_deref(),
                set_params,
            )
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
            assert_eq!(input, vec![PathBuf::from("test.md")]);
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
            assert_eq!(input, vec![PathBuf::from("test.md")]);
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

    // --- T023: Parse forge resolve basic ---

    #[test]
    fn parse_resolve_subcommand() {
        let cli = Cli::try_parse_from(["forge", "resolve", "profile.json"]).unwrap();
        if let Commands::Resolve { input, output, check, timeout, oscal_cli_path } = cli.command {
            assert_eq!(input, Some(PathBuf::from("profile.json")));
            assert!(output.is_none());
            assert!(!check);
            assert_eq!(timeout, 60);
            assert!(oscal_cli_path.is_none());
        } else {
            panic!("Expected Resolve command");
        }
    }

    // --- T024: Parse forge resolve with all options ---

    #[test]
    fn parse_resolve_with_all_options() {
        let cli = Cli::try_parse_from([
            "forge",
            "resolve",
            "profile.json",
            "--output",
            "resolved.json",
            "--timeout",
            "30",
            "--oscal-cli-path",
            "/usr/local/bin/oscal-cli",
        ])
        .unwrap();
        if let Commands::Resolve { input, output, check, timeout, oscal_cli_path } = cli.command {
            assert_eq!(input, Some(PathBuf::from("profile.json")));
            assert_eq!(output, Some(PathBuf::from("resolved.json")));
            assert!(!check);
            assert_eq!(timeout, 30);
            assert_eq!(oscal_cli_path, Some(PathBuf::from("/usr/local/bin/oscal-cli")));
        } else {
            panic!("Expected Resolve command");
        }
    }

    // --- T053: Parse forge resolve --check ---

    #[test]
    fn parse_resolve_check_flag() {
        let cli = Cli::try_parse_from(["forge", "resolve", "--check"]).unwrap();
        if let Commands::Resolve { input, check, .. } = cli.command {
            assert!(check);
            assert!(input.is_none());
        } else {
            panic!("Expected Resolve command");
        }
    }

    // --- T038: Other commands don't check oscal-cli ---

    #[test]
    fn convert_command_unaffected_by_oscal_cli() {
        let cli =
            Cli::try_parse_from(["forge", "convert", "test.md", "--strategy", "catalog"]).unwrap();
        assert!(matches!(cli.command, Commands::Convert { .. }));
    }

    // T050: CLI parsing tests for trace subcommand

    #[test]
    fn parse_trace_subcommand() {
        let cli = Cli::try_parse_from(["forge", "trace", "artifact.json", "--source", "policy.md"])
            .unwrap();
        if let Commands::Trace { artifact, source, output } = cli.command {
            assert_eq!(artifact, PathBuf::from("artifact.json"));
            assert_eq!(source, PathBuf::from("policy.md"));
            assert!(output.is_none());
        } else {
            panic!("Expected Trace command");
        }
    }

    #[test]
    fn parse_trace_with_output() {
        let cli = Cli::try_parse_from([
            "forge",
            "trace",
            "artifact.json",
            "--source",
            "policy.md",
            "--output",
            "report.txt",
        ])
        .unwrap();
        if let Commands::Trace { output, .. } = cli.command {
            assert_eq!(output, Some(PathBuf::from("report.txt")));
        } else {
            panic!("Expected Trace command");
        }
    }

    #[test]
    fn parse_trace_missing_source_fails() {
        let result = Cli::try_parse_from(["forge", "trace", "artifact.json"]);
        assert!(result.is_err(), "Should fail when --source is omitted");
    }

    #[test]
    fn parse_convert_multiple_inputs() {
        let cli =
            Cli::try_parse_from(["forge", "convert", "a.md", "b.md", "--strategy", "catalog"])
                .unwrap();
        if let Commands::Convert { input, .. } = cli.command {
            assert_eq!(input, vec![PathBuf::from("a.md"), PathBuf::from("b.md")]);
        } else {
            panic!("Expected Convert command");
        }
    }

    // T037: --jobs flag parsing tests

    #[test]
    fn parse_jobs_flag_explicit_value() {
        let cli = Cli::try_parse_from([
            "forge",
            "convert",
            "a.md",
            "--strategy",
            "catalog",
            "--jobs",
            "4",
        ])
        .unwrap();
        if let Commands::Convert { jobs, .. } = cli.command {
            assert_eq!(jobs, 4);
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn parse_jobs_flag_one_is_sequential() {
        let cli = Cli::try_parse_from([
            "forge",
            "convert",
            "a.md",
            "--strategy",
            "catalog",
            "--jobs",
            "1",
        ])
        .unwrap();
        if let Commands::Convert { jobs, .. } = cli.command {
            assert_eq!(jobs, 1);
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn parse_jobs_flag_zero_accepted_as_auto() {
        // 0 is valid — means auto (use all available cores)
        let cli = Cli::try_parse_from([
            "forge",
            "convert",
            "a.md",
            "--strategy",
            "catalog",
            "--jobs",
            "0",
        ])
        .unwrap();
        if let Commands::Convert { jobs, .. } = cli.command {
            assert_eq!(jobs, 0);
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn parse_jobs_flag_257_rejected() {
        let result = Cli::try_parse_from([
            "forge",
            "convert",
            "a.md",
            "--strategy",
            "catalog",
            "--jobs",
            "257",
        ]);
        assert!(result.is_err(), "257 should be rejected (max 256)");
    }

    #[test]
    fn parse_jobs_flag_default_is_zero() {
        let cli =
            Cli::try_parse_from(["forge", "convert", "a.md", "--strategy", "catalog"]).unwrap();
        if let Commands::Convert { jobs, .. } = cli.command {
            assert_eq!(jobs, 0, "Default should be 0 (auto)");
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn parse_convert_with_summary_flag() {
        let cli = Cli::try_parse_from([
            "forge",
            "convert",
            "test.md",
            "--strategy",
            "catalog",
            "--summary",
        ])
        .unwrap();
        if let Commands::Convert { summary, .. } = cli.command {
            assert!(summary, "Expected --summary to be true");
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn parse_convert_without_summary_flag_defaults_false() {
        let cli =
            Cli::try_parse_from(["forge", "convert", "test.md", "--strategy", "catalog"]).unwrap();
        if let Commands::Convert { summary, .. } = cli.command {
            assert!(!summary, "Expected --summary to default to false");
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn verbose_and_quiet_conflict() {
        let result = Cli::try_parse_from(["forge", "-v", "-q", "convert", "test.md"]);
        assert!(result.is_err(), "Expected error when both --verbose and --quiet are provided");
    }

    #[test]
    fn parse_diff_subcommand() {
        let cli = Cli::try_parse_from(["forge", "diff", "old.json", "new.json"]).unwrap();
        if let Commands::Diff { old_artifact, new_artifact } = cli.command {
            assert_eq!(old_artifact, PathBuf::from("old.json"));
            assert_eq!(new_artifact, PathBuf::from("new.json"));
        } else {
            panic!("Expected Diff command");
        }
    }

    #[test]
    fn parse_diff_missing_new_artifact_fails() {
        let result = Cli::try_parse_from(["forge", "diff", "old.json"]);
        assert!(result.is_err(), "Should fail when new_artifact is omitted");
    }
}
