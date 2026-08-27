/// Config-check subcommand: inspect and validate project configuration.
pub mod config_check;
pub mod convert;
/// Diff subcommand: compare two OSCAL artifacts.
pub mod diff;
/// Drift subcommand: content-safe, canonical artifact comparison for CI.
pub mod drift;
pub mod export;
pub mod migrate;
pub mod output;
pub mod profile;
pub mod resolve;
/// Trace subcommand: traceability report generator.
pub mod trace;
/// Validate subcommand: OSCAL schema validation.
pub mod validate;

use std::path::Path;
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

pub use crate::types::{OutputFormat, OutputType, Strategy};

use crate::ForgeError;
use crate::config::{self, ConvertCliValues};

/// Top-level CLI configuration for the `forge` tool.
///
/// Holds global flags (verbose, quiet) and the active subcommand.
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
    /// The subcommand to execute, including mapping, applicability, lifecycle, and framework workflows.
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output showing pipeline stage information
    #[arg(short, long, global = true, conflicts_with = "quiet")]
    pub verbose: bool,

    /// Suppress all non-essential output (only OSCAL artifact on stdout)
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Path to a project configuration file (overrides `$FORGE_CONFIG` and .forge.toml discovery)
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,
}

/// CLI subcommands for the `forge` tool.
///
/// Each variant maps to a distinct pipeline or utility operation.
#[derive(Subcommand)]
pub enum Commands {
    /// Convert a policy document to OSCAL format
    Convert {
        /// Path(s) to the input Markdown policy document(s) (.md)
        #[arg(num_args = 1.., required = true)]
        input: Vec<PathBuf>,

        /// Conversion strategy: 'catalog' for OSCAL Catalog, 'component' for Component Definition.
        /// May also be supplied by project configuration (.forge.toml).
        #[arg(long)]
        strategy: Option<Strategy>,

        /// Output format: json, xml, or yaml (default: json or project configuration)
        #[arg(long)]
        format: Option<OutputFormat>,

        /// Write output to a file instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,

        /// Maximum input file size in MB (default: 10 or project configuration)
        #[arg(long, value_parser = clap::value_parser!(u64).range(1..=51_200))]
        max_size: Option<u64>,

        /// Source profile/baseline reference for component strategy (e.g., path to OSCAL profile JSON)
        #[arg(long)]
        source_profile: Option<String>,

        /// Number of parallel jobs for batch conversion (0 = auto, 1 = sequential;
        /// default: 0, `$FORGE_JOBS`, or project configuration)
        #[arg(long, value_parser = clap::value_parser!(u16).range(0..=256))]
        jobs: Option<u16>,

        /// Optional baseline policy used to compare stable IDs and emit substantive-change warnings.
        ///
        /// When provided, FORGE computes stable IDs for both documents and emits a warning if
        /// matching requirement locations have different stable IDs (indicating non-whitespace
        /// substantive text changes).
        #[arg(long)]
        stable_id_baseline: Option<PathBuf>,

        /// Output artifact type: catalog, component-definition, or ssp.
        ///
        /// When omitted, `--strategy` determines the output type (catalog or component-definition).
        /// Use `--to ssp` to generate a System Security Plan skeleton from the policy document.
        #[arg(long, value_enum)]
        to: Option<OutputType>,

        /// SSP reference for Assessment Plan generation.
        /// When provided, an Assessment Plan skeleton is written to
        /// {output_dir}/{policy_stem}-assessment-plan.json alongside the converted artifact.
        /// Ignored in batch mode (2+ input files).
        #[arg(long)]
        import_ssp: Option<String>,

        /// Print a conversion summary dashboard to stderr after conversion
        #[arg(long, overrides_with = "no_summary")]
        summary: bool,

        /// Explicitly disable the summary dashboard (overrides convert.summary = true in project configuration)
        #[arg(long)]
        no_summary: bool,
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

        /// Override auto-detected OSCAL model type (catalog, component-definition, or mapping)
        #[arg(long, value_enum, conflicts_with = "round_trip")]
        schema_type: Option<SchemaType>,

        /// Output format for validation results (text or json; default: text or project configuration)
        #[arg(long, value_enum)]
        format: Option<ValidateOutputFormat>,

        /// Run round-trip validation (JSON → XML → YAML → JSON) via oscal-cli
        #[arg(long)]
        round_trip: bool,

        /// Write validation results to a file instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,

        /// Timeout for each oscal-cli conversion step in seconds
        /// (default: 30 or project configuration)
        #[arg(long, requires = "round_trip")]
        timeout: Option<u64>,

        /// Explicit path to oscal-cli binary (overrides PATH search)
        #[arg(long, requires = "round_trip")]
        oscal_cli_path: Option<PathBuf>,
    },

    /// Resolve an OSCAL Profile into a flat Catalog baseline via oscal-cli
    Resolve {
        /// Path to the OSCAL Profile JSON file
        input: Option<PathBuf>,

        /// Output file path (default: `&lt;input-stem&gt;-resolved.json`)
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

    /// Check generated OSCAL JSON for substantive drift without printing policy content
    Drift {
        /// Path to the committed OSCAL artifact (JSON)
        committed_artifact: PathBuf,

        /// Path to the newly generated OSCAL artifact (JSON)
        generated_artifact: PathBuf,

        /// Status output format; neither format includes policy content or file paths
        #[arg(long, value_enum, default_value = "text")]
        format: DriftOutputFormat,
    },

    /// Analyze stable-ID migration between two source-policy revisions.
    ///
    /// Inputs support `.md`, `.markdown`, `.pdf`, and `.docx`; other extensions
    /// are rejected as unsupported input formats (exit 1).
    Migrate {
        /// Path to the old policy revision (.md, .markdown, .pdf, or .docx)
        old_policy: PathBuf,

        /// Path to the new policy revision (.md, .markdown, .pdf, or .docx)
        new_policy: PathBuf,

        /// Migration report format
        #[arg(long, value_enum, default_value = "text")]
        format: MigrationOutputFormat,

        /// Write the completed report to a file instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,

        /// Reviewer-authored `forge.successor-map/1` declarations
        #[arg(long)]
        successor_map: Option<PathBuf>,

        /// Maximum size of each policy input in MB
        #[arg(
            long,
            default_value = "10",
            value_parser = clap::value_parser!(u64).range(1..=51_200)
        )]
        max_size: u64,
    },

    /// Build or check human-reviewed OSCAL Control Mapping artifacts
    Mapping {
        #[command(subcommand)]
        command: MappingCommand,
    },

    /// Manage deterministic local policy lifecycle records
    Lifecycle {
        #[command(subcommand)]
        command: LifecycleCommand,
    },

    /// Declare framework applicability and analyze policy-mapping gaps
    Applicability {
        #[command(subcommand)]
        command: ApplicabilityCommand,
    },

    /// Analyze the impact of a caller-supplied framework revision
    Framework {
        #[command(subcommand)]
        command: FrameworkCommand,
    },

    /// Inspect and validate project configuration (.forge.toml)
    Config {
        /// Config subcommands
        #[command(subcommand)]
        command: ConfigCommand,
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

        /// Override the last-modified timestamp (RFC 3339) for reproducible output.
        #[arg(long)]
        timestamp: Option<chrono::DateTime<chrono::Utc>>,
    },
}

/// Subcommands for `forge config`.
#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Validate the selected project configuration without side effects
    ///
    /// Uses the global `--config` selector, so both
    /// `forge --config <path> config check` and
    /// `forge config check --config <path>` resolve identically.
    Check,
}

/// Control Mapping workflow commands.
#[derive(Subcommand)]
pub enum MappingCommand {
    /// Create an unapproved deterministic manifest skeleton without relationship claims
    Init {
        /// Local source Catalog or Profile JSON
        #[arg(long)]
        source: PathBuf,
        /// Local target Catalog or Profile JSON
        #[arg(long)]
        target: PathBuf,
        /// Explicit resolved Catalog companion when source is a Profile
        #[arg(long)]
        source_resolved_catalog: Option<PathBuf>,
        /// Explicit resolved Catalog companion when target is a Profile
        #[arg(long)]
        target_resolved_catalog: Option<PathBuf>,
        /// Write the scaffold to a file instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Build schema-valid OSCAL Mapping JSON from a reviewer-authored manifest
    Build {
        /// Versioned `forge.mapping-manifest/1` JSON file
        #[arg(long)]
        manifest: PathBuf,
        /// Write OSCAL Mapping JSON to a file instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,
        /// Write the deterministic review report to a separate file
        #[arg(long)]
        report: Option<PathBuf>,
        /// Review report format
        #[arg(long, value_enum, default_value = "text")]
        report_format: MappingReportFormat,
        /// Prior FORGE Mapping JSON used for change-impact analysis
        #[arg(long)]
        baseline: Option<PathBuf>,
        /// Baseline finding policy that produces exit status 1
        #[arg(long, value_enum, default_value = "never", requires = "baseline")]
        fail_on: MappingFailOn,
        /// Include bounded subject titles/prose in the sensitive review report
        #[arg(long, requires = "report")]
        include_excerpts: bool,
    },
    /// Check a manifest and current resources against an existing Mapping baseline
    Check {
        /// Versioned `forge.mapping-manifest/1` JSON file
        #[arg(long)]
        manifest: PathBuf,
        /// Prior FORGE Mapping JSON used for change-impact analysis
        #[arg(long)]
        baseline: PathBuf,
        /// Write the deterministic impact report to a file instead of stdout
        #[arg(long)]
        report: Option<PathBuf>,
        /// Review report format
        #[arg(long, value_enum, default_value = "text")]
        report_format: MappingReportFormat,
        /// Finding policy that produces exit status 1
        #[arg(long, value_enum, default_value = "any")]
        fail_on: MappingFailOn,
        /// Include bounded subject titles/prose in the sensitive review report
        #[arg(long)]
        include_excerpts: bool,
    },
}

/// Policy lifecycle workflow commands.
#[derive(Subcommand)]
pub enum LifecycleCommand {
    /// Create a draft record bound to current source and OSCAL artifact bytes
    Init {
        /// Source policy file
        #[arg(long)]
        source: PathBuf,
        /// Generated OSCAL JSON artifact; repeat for each artifact
        #[arg(long = "artifact")]
        artifacts: Vec<PathBuf>,
        /// Lifecycle record to create
        #[arg(long)]
        output: PathBuf,
        /// Stable logical policy key
        #[arg(long)]
        policy_key: String,
        /// Immutable reviewed version key
        #[arg(long)]
        version_key: String,
        /// Human-readable policy title
        #[arg(long)]
        title: String,
        /// Owner party key; repeat for multiple owners
        #[arg(long = "owner", required = true)]
        owners: Vec<String>,
        /// Party declaration in KEY=ROLE[,ROLE] form
        #[arg(long = "party")]
        parties: Vec<String>,
        /// Explicit next review date (YYYY-MM-DD)
        #[arg(long)]
        next_review: chrono::NaiveDate,
        /// Review cadence in days
        #[arg(long, default_value = "365", value_parser = clap::value_parser!(u16).range(1..))]
        cadence_days: u16,
        /// Number of days before review to classify as due-soon
        #[arg(long, default_value = "30")]
        due_soon_days: u16,
        /// Distinct reviewer assertions required for approval
        #[arg(long, default_value = "1", value_parser = clap::value_parser!(u16).range(1..))]
        required_reviewers: u16,
        /// Distinct approver assertions required for approval
        #[arg(long, default_value = "1", value_parser = clap::value_parser!(u16).range(1..))]
        required_approvers: u16,
        /// Require different declared actors for author and reviewer roles
        #[arg(long)]
        separate_author_reviewer: bool,
        /// Require different declared actors for author and approver roles
        #[arg(long)]
        separate_author_approver: bool,
        /// Require different declared actors for reviewer and approver roles
        #[arg(long)]
        separate_reviewer_approver: bool,
    },
    /// Validate records, current artifact identity, and supplied supersession graph
    Check {
        /// Lifecycle record; repeat to validate a portfolio
        #[arg(long = "record", required = true)]
        records: Vec<PathBuf>,
        /// Report format
        #[arg(long, value_enum, default_value = "text")]
        format: LifecycleOutputFormat,
        /// Write the report atomically instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Migrate a legacy /1 record to a context-bound /2 record
    Migrate {
        /// Valid legacy lifecycle record
        #[arg(long)]
        record: PathBuf,
        /// New lifecycle /2 record; the legacy input is never modified
        #[arg(long)]
        output: PathBuf,
    },
    /// Propose a transition, or append it atomically with --apply
    Transition {
        /// Lifecycle record
        #[arg(long)]
        record: PathBuf,
        /// Next lifecycle state
        #[arg(long = "to", value_enum)]
        next_state: crate::lifecycle::record::LifecycleState,
        /// Declared actor key
        #[arg(long)]
        actor: String,
        /// Actor's declared role
        #[arg(long, value_enum)]
        role: crate::lifecycle::record::DeclaredRole,
        /// Explicit RFC 3339 event timestamp
        #[arg(long)]
        at: chrono::DateTime<chrono::Utc>,
        /// Bounded rationale for the transition
        #[arg(long)]
        rationale: String,
        /// Additional approval evidence in ACTOR=ROLE form
        #[arg(long = "assertion")]
        assertions: Vec<String>,
        /// PRD-057 impact finding ID that triggered re-review; repeat as needed
        #[arg(long = "impact-finding-id")]
        impact_finding_ids: Vec<String>,
        /// Replacement policy key for supersession
        #[arg(long, requires = "replacement_version_key")]
        replacement_policy_key: Option<String>,
        /// Replacement version key for supersession
        #[arg(long, requires = "replacement_policy_key")]
        replacement_version_key: Option<String>,
        /// Append the validated event to the record atomically
        #[arg(long, conflicts_with = "output")]
        apply: bool,
        /// Write a proposed complete record atomically instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Report deterministic lifecycle status for an explicit date
    Status {
        /// Lifecycle record; repeat for a deterministic review queue
        #[arg(long = "record", required = true)]
        records: Vec<PathBuf>,
        /// Explicit status date (YYYY-MM-DD)
        #[arg(long)]
        as_of: chrono::NaiveDate,
        /// Status output format
        #[arg(long, value_enum, default_value = "text")]
        format: LifecycleOutputFormat,
        /// Gate that determines whether action-required status exits 1
        #[arg(long, value_enum, default_value = "publication")]
        gate: LifecycleGate,
        /// Write the report atomically instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Emit a deterministic review queue grouped by owner and next-review date
    Queue {
        /// Lifecycle record; repeat to build a portfolio queue
        #[arg(long = "record", required = true)]
        records: Vec<PathBuf>,
        /// Explicit queue date (YYYY-MM-DD)
        #[arg(long)]
        as_of: chrono::NaiveDate,
        /// Queue output format
        #[arg(long, value_enum, default_value = "json")]
        format: LifecycleOutputFormat,
        /// Gate that determines whether action-required status exits 1
        #[arg(long, value_enum, default_value = "publication")]
        gate: LifecycleGate,
        /// Write the queue atomically instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Emit deterministic unsigned approval evidence for external signing
    Attest {
        /// Approved, non-drifted lifecycle record
        #[arg(long)]
        record: PathBuf,
        /// Write attestation JSON atomically instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

/// Deterministic lifecycle report serialization.
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum LifecycleOutputFormat {
    Text,
    Json,
}

/// Lifecycle status gate.
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum LifecycleGate {
    Publication,
    None,
}

/// Framework applicability workflow commands.
#[derive(Subcommand)]
pub enum ApplicabilityCommand {
    /// Inventory a framework and create an undecided manifest scaffold
    Init {
        /// Local OSCAL Catalog or Profile JSON
        #[arg(long)]
        framework: PathBuf,
        /// Explicit resolved Catalog companion when the framework is a Profile
        #[arg(long)]
        resolved_catalog: Option<PathBuf>,
        /// Write the scaffold to a file instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Validate decisions and Mapping Collections, then classify every framework control
    Analyze {
        /// Versioned `forge.applicability/1` JSON manifest
        #[arg(long)]
        manifest: PathBuf,
        /// Deterministic report format
        #[arg(long, value_enum, default_value = "text")]
        format: ApplicabilityReportFormat,
        /// Write the report to a file instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,
        /// Category gate that produces exit status 1 after a valid report is written
        #[arg(long, value_enum, default_value = "never")]
        fail_on: ApplicabilityFailOn,
        /// Deterministic date used by the overdue-deferred gate (YYYY-MM-DD)
        #[arg(long)]
        as_of: Option<chrono::NaiveDate>,
        /// Show controls in this group (including nested group membership)
        #[arg(long)]
        group: Option<String>,
        /// Show controls whose stable ID starts with this prefix
        #[arg(long)]
        control_prefix: Option<String>,
        /// Show controls with this primary classification
        #[arg(long, value_enum)]
        state: Option<ApplicabilityStateFilter>,
        /// Show controls assigned to or reviewed by this reviewer key
        #[arg(long)]
        reviewer: Option<String>,
        /// Show controls participating in mappings from this source href
        #[arg(long)]
        policy_source: Option<String>,
    },
}

/// Deterministic applicability report serialization.
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum ApplicabilityReportFormat {
    Text,
    Json,
    Html,
}

/// Valid analysis categories that can require human review in CI.
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum ApplicabilityFailOn {
    #[value(name = "applicable-unmapped")]
    ApplicableUnmapped,
    #[value(name = "any-review-action")]
    AnyReviewAction,
    #[value(name = "overdue-deferred")]
    OverdueDeferred,
    Never,
}

/// Applicability classification used to filter report details.
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum ApplicabilityStateFilter {
    ApplicableMapped,
    ApplicableReviewedNoRelationship,
    ApplicableUnmapped,
    NotApplicable,
    Deferred,
    UnderReview,
}

/// Framework revision analysis commands.
#[derive(Subcommand)]
pub enum FrameworkCommand {
    /// Compare an exact old/new Catalog pair and emit a deterministic review queue
    Impact {
        /// Versioned `forge.framework-impact/1` portfolio manifest
        #[arg(long)]
        manifest: PathBuf,
        /// Write the report to a file instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,
        /// Deterministic report format
        #[arg(long, value_enum, default_value = "text")]
        format: FrameworkReportFormat,
        /// Minimum finding priority that produces exit status 1
        #[arg(long, value_enum, default_value = "review-required")]
        fail_on: FrameworkFailOn,
        /// Show findings for controls in this framework group
        #[arg(long)]
        group: Option<String>,
        /// Show findings tied to this prior applicability decision state
        #[arg(long, value_enum)]
        decision_state: Option<FrameworkDecisionStateFilter>,
        /// Show findings tied to this exact policy source href
        #[arg(long)]
        policy_source: Option<String>,
        /// Show findings with this exact impact priority
        #[arg(long, value_enum)]
        priority: Option<FrameworkImpactPriorityFilter>,
        /// Show findings assigned to this exact reviewer or migration approver key
        #[arg(long)]
        owner: Option<String>,
    },
}

/// Deterministic framework-impact report serialization.
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum FrameworkReportFormat {
    Text,
    Json,
    Markdown,
    Html,
    Github,
}

/// Framework-impact gate threshold. Thresholds can be made stricter, but not disabled.
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum FrameworkFailOn {
    Blocking,
    #[value(name = "review-required")]
    ReviewRequired,
    Any,
}

/// Human applicability decision state used to filter framework-impact findings.
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum FrameworkDecisionStateFilter {
    Applicable,
    NotApplicable,
    Deferred,
    UnderReview,
}

/// Exact framework-impact finding priority used to filter report details.
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum FrameworkImpactPriorityFilter {
    Blocking,
    ReviewRequired,
    Informational,
}

/// Deterministic Control Mapping report serialization.
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum MappingReportFormat {
    Text,
    Json,
}

/// Baseline impact categories that may require human review.
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum MappingFailOn {
    Stale,
    #[value(name = "subject-change")]
    SubjectChange,
    #[value(name = "gap-increase")]
    GapIncrease,
    Any,
    Never,
}

/// Output format for `forge validate` results (WI-20).
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum ValidateOutputFormat {
    /// Human-readable text output (default)
    Text,
    /// Machine-parseable JSON output (PRD S-1)
    Json,
}

/// Output format for the content-safe drift result.
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum DriftOutputFormat {
    /// Three-line, human-readable status output.
    Text,
    /// Single-object, machine-readable JSON status output.
    Json,
}

/// Output format for policy migration reports.
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum MigrationOutputFormat {
    /// Human-readable audit report.
    Text,
    /// Versioned machine-readable report.
    Json,
}

/// CLI enum for --schema-type override.
///
/// Allows explicitly specifying the OSCAL model type when auto-detection
/// is ambiguous or incorrect.
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum SchemaType {
    /// Validate against the OSCAL Catalog schema.
    Catalog,
    /// Validate against the OSCAL Component Definition schema.
    #[value(name = "component-definition")]
    ComponentDefinition,
    /// Validate against the OSCAL Control Mapping schema.
    Mapping,
}

/// Raw convert-command arguments captured from clap before resolution.
#[derive(Clone, Copy)]
struct ConvertArgs<'a> {
    input: &'a [PathBuf],
    strategy: Option<Strategy>,
    format: Option<OutputFormat>,
    output: Option<&'a Path>,
    max_size: Option<u64>,
    source_profile: Option<&'a str>,
    jobs: Option<u16>,
    stable_id_baseline: Option<&'a Path>,
    import_ssp: Option<&'a str>,
    to: Option<OutputType>,
    /// `Some(true)` for `--summary`, `Some(false)` for `--no-summary`.
    summary: Option<bool>,
}

/// Resolve effective convert settings and dispatch conversion.
fn run_convert(
    args: &ConvertArgs<'_>,
    explicit_config: Option<&Path>,
    quiet: bool,
) -> Result<(), ForgeError> {
    if args.input.is_empty() {
        return Err(ForgeError::BatchConversion("No input files provided".to_string()));
    }
    let project_config = config::load_selected(explicit_config)?;
    let cli_values = ConvertCliValues {
        strategy: args.strategy,
        format: args.format,
        output: args.output,
        max_size: args.max_size,
        source_profile: args.source_profile,
        jobs: args.jobs,
        stable_id_baseline: args.stable_id_baseline,
        to: args.to,
        import_ssp: args.import_ssp,
        summary: args.summary,
    };
    let effective =
        config::resolve_convert(cli_values, project_config.as_ref(), config::env_jobs())?;
    // Borrow locals for the option struct (ConvertOptions borrows paths/strings).
    let source_profile_str =
        effective.source_profile.clone().map(|p| p.to_string_lossy().into_owned());
    let opts = convert::ConvertOptions {
        input: &args.input[0],
        strategy: &effective.strategy,
        format: &effective.format,
        output: effective.output.as_deref(),
        max_size: effective.max_size_mb,
        source_profile: source_profile_str.as_deref(),
        stable_id_baseline: effective.stable_id_baseline.as_deref(),
        import_ssp: effective.import_ssp.as_deref(),
        summary: effective.summary,
        quiet,
        jobs: effective.jobs,
        to: effective.to.as_ref(),
    };
    convert::execute_dispatch(args.input, &opts)
}

/// Raw validate-command arguments captured from clap before resolution.
#[derive(Clone)]
struct ValidateArgs<'a> {
    input: &'a Path,
    schema_type: Option<SchemaType>,
    format: Option<ValidateOutputFormat>,
    output: Option<&'a Path>,
    timeout: Option<u64>,
    round_trip: bool,
    oscal_cli_path: Option<&'a Path>,
}

/// Resolve effective validate settings and dispatch validation.
fn run_validate(
    args: &ValidateArgs<'_>,
    project_config: Option<&crate::config::ProjectConfig>,
) -> Result<(), ForgeError> {
    let effective = config::resolve_validate(
        args.schema_type.clone(),
        args.format.clone(),
        args.output,
        args.timeout,
        project_config,
    )?;
    if args.round_trip {
        validate::execute_round_trip(
            args.input,
            &effective.format,
            effective.output.as_deref(),
            effective.timeout_seconds,
            args.oscal_cli_path,
        )
    } else {
        validate::execute(
            args.input,
            effective.schema_type.as_ref(),
            &effective.format,
            effective.output.as_deref(),
        )
    }
}

/// The global `--config` selector is only consumed by the commands whose
/// settings schema version 1 supports (PRD 051 M-2/M-12). Rejecting it
/// explicitly prevents it from being silently ignored elsewhere.
fn reject_unsupported_config_selector(cli: &Cli) -> Result<(), ForgeError> {
    let config_supported = matches!(
        &cli.command,
        Commands::Convert { .. } | Commands::Validate { .. } | Commands::Config { .. }
    );
    if cli.config.is_some() && !config_supported {
        return Err(ForgeError::Config(
            "--config is only supported by the 'convert', 'validate', and 'config' commands"
                .to_string(),
        ));
    }
    Ok(())
}

fn run_drift(
    committed_artifact: &Path,
    generated_artifact: &Path,
    format: &DriftOutputFormat,
) -> Result<(), ForgeError> {
    if drift::execute(committed_artifact, generated_artifact, format)? {
        Err(ForgeError::DriftDetected)
    } else {
        Ok(())
    }
}

fn run_migrate(
    old_policy: &Path,
    new_policy: &Path,
    format: &MigrationOutputFormat,
    output: Option<&Path>,
    successor_map: Option<&Path>,
    max_size: u64,
) -> Result<(), ForgeError> {
    if migrate::execute(old_policy, new_policy, format, output, successor_map, max_size)? {
        Err(ForgeError::MigrationHasChanges)
    } else {
        Ok(())
    }
}

/// Execute the CLI command, dispatching to the appropriate subcommand handler.
///
/// # Errors
///
/// Returns `ForgeError` if the subcommand handler fails.
#[allow(clippy::too_many_lines)] // Flat exhaustive dispatch keeps every clap variant visible here.
pub fn execute(cli: &Cli) -> Result<(), ForgeError> {
    reject_unsupported_config_selector(cli)?;
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
            import_ssp,
            to,
            summary,
            no_summary,
        } => {
            let summary = if *summary {
                Some(true)
            } else if *no_summary {
                Some(false)
            } else {
                None
            };
            run_convert(
                &ConvertArgs {
                    input,
                    strategy: *strategy,
                    format: *format,
                    output: output.as_deref(),
                    max_size: *max_size,
                    source_profile: source_profile.as_deref(),
                    jobs: *jobs,
                    stable_id_baseline: stable_id_baseline.as_deref(),
                    import_ssp: import_ssp.as_deref(),
                    to: *to,
                    summary,
                },
                cli.config.as_deref(),
                cli.quiet,
            )
        }
        Commands::Export { input, format, output } => {
            export::execute(input, format, output.as_deref())
        }
        Commands::Validate {
            input,
            schema_type,
            format,
            round_trip,
            output,
            timeout,
            oscal_cli_path,
        } => {
            let project_config = config::load_selected(cli.config.as_deref())?;
            run_validate(
                &ValidateArgs {
                    input,
                    schema_type: schema_type.clone(),
                    format: format.clone(),
                    output: output.as_deref(),
                    timeout: *timeout,
                    round_trip: *round_trip,
                    oscal_cli_path: oscal_cli_path.as_deref(),
                },
                project_config.as_ref(),
            )
        }
        Commands::Config { command } => match command {
            // The global `--config` selector applies here too; because it is a
            // clap `global = true` argument it binds identically whether the
            // user writes `forge --config <path> config check` or
            // `forge config check --config <path>`.
            ConfigCommand::Check => config_check::execute(cli.config.as_deref()),
        },
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
        Commands::Migrate { old_policy, new_policy, format, output, successor_map, max_size } => {
            run_migrate(
                old_policy,
                new_policy,
                format,
                output.as_deref(),
                successor_map.as_deref(),
                *max_size,
            )
        }
        Commands::Mapping { command } => match command {
            MappingCommand::Init {
                source,
                target,
                source_resolved_catalog,
                target_resolved_catalog,
                output,
            } => crate::mapping::execute_init(
                source,
                target,
                source_resolved_catalog.as_deref(),
                target_resolved_catalog.as_deref(),
                output.as_deref(),
            ),
            MappingCommand::Build {
                manifest,
                output,
                report,
                report_format,
                baseline,
                fail_on,
                include_excerpts,
            } => {
                if crate::mapping::execute_build(
                    manifest,
                    output.as_deref(),
                    report.as_deref(),
                    report_format,
                    baseline.as_deref(),
                    fail_on,
                    *include_excerpts,
                )? {
                    Err(ForgeError::MappingReviewRequired)
                } else {
                    Ok(())
                }
            }
            MappingCommand::Check {
                manifest,
                baseline,
                report,
                report_format,
                fail_on,
                include_excerpts,
            } => {
                if crate::mapping::execute_check(
                    manifest,
                    baseline,
                    report.as_deref(),
                    report_format,
                    fail_on,
                    *include_excerpts,
                )? {
                    Err(ForgeError::MappingReviewRequired)
                } else {
                    Ok(())
                }
            }
        },
        Commands::Lifecycle { command } => match command {
            LifecycleCommand::Init {
                source,
                artifacts,
                output,
                policy_key,
                version_key,
                title,
                owners,
                parties,
                next_review,
                cadence_days,
                due_soon_days,
                required_reviewers,
                required_approvers,
                separate_author_reviewer,
                separate_author_approver,
                separate_reviewer_approver,
            } => crate::lifecycle::execute_init(&crate::lifecycle::InitOptions {
                source,
                artifacts,
                output,
                policy_key,
                version_key,
                title,
                owners,
                parties,
                next_review: *next_review,
                cadence_days: *cadence_days,
                due_soon_days: *due_soon_days,
                required_reviewers: *required_reviewers,
                required_approvers: *required_approvers,
                separate_author_reviewer: *separate_author_reviewer,
                separate_author_approver: *separate_author_approver,
                separate_reviewer_approver: *separate_reviewer_approver,
            }),
            LifecycleCommand::Check { records, format, output } => {
                if crate::lifecycle::execute_check(records, format, output.as_deref())? {
                    Err(ForgeError::LifecycleActionRequired)
                } else {
                    Ok(())
                }
            }
            LifecycleCommand::Migrate { record, output } => {
                crate::lifecycle::execute_migrate(record, output)
            }
            LifecycleCommand::Transition {
                record,
                next_state,
                actor,
                role,
                at,
                rationale,
                assertions,
                impact_finding_ids,
                replacement_policy_key,
                replacement_version_key,
                apply,
                output,
            } => {
                let timestamp = at.to_rfc3339();
                crate::lifecycle::execute_transition(&crate::lifecycle::TransitionOptions {
                    record_path: record,
                    next_state: *next_state,
                    actor_key: actor,
                    role: *role,
                    timestamp: &timestamp,
                    rationale,
                    assertions,
                    impact_finding_ids,
                    replacement_policy_key: replacement_policy_key.as_deref(),
                    replacement_version_key: replacement_version_key.as_deref(),
                    apply: *apply,
                    output: output.as_deref(),
                })
            }
            LifecycleCommand::Status { records, as_of, format, gate, output } => {
                if crate::lifecycle::execute_status(
                    records,
                    *as_of,
                    format,
                    gate,
                    output.as_deref(),
                )? {
                    Err(ForgeError::LifecycleActionRequired)
                } else {
                    Ok(())
                }
            }
            LifecycleCommand::Queue { records, as_of, format, gate, output } => {
                if crate::lifecycle::execute_queue(
                    records,
                    *as_of,
                    format,
                    gate,
                    output.as_deref(),
                )? {
                    Err(ForgeError::LifecycleActionRequired)
                } else {
                    Ok(())
                }
            }
            LifecycleCommand::Attest { record, output } => {
                crate::lifecycle::execute_attest(record, output.as_deref())
            }
        },
        Commands::Applicability { command } => match command {
            ApplicabilityCommand::Init { framework, resolved_catalog, output } => {
                crate::applicability::execute_init(
                    framework,
                    resolved_catalog.as_deref(),
                    output.as_deref(),
                )
            }
            ApplicabilityCommand::Analyze {
                manifest,
                format,
                output,
                fail_on,
                as_of,
                group,
                control_prefix,
                state,
                reviewer,
                policy_source,
            } => {
                if crate::applicability::execute_analyze(
                    manifest,
                    format,
                    output.as_deref(),
                    fail_on,
                    *as_of,
                    crate::applicability::model::ReportFilters {
                        group: group.clone(),
                        control_prefix: control_prefix.clone(),
                        state: state.as_ref().map(crate::applicability::classification_filter),
                        reviewer: reviewer.clone(),
                        policy_source: policy_source.clone(),
                    },
                )? {
                    Err(ForgeError::ApplicabilityReviewRequired)
                } else {
                    Ok(())
                }
            }
        },
        Commands::Framework { command } => match command {
            FrameworkCommand::Impact {
                manifest,
                output,
                format,
                fail_on,
                group,
                decision_state,
                policy_source,
                priority,
                owner,
            } => {
                let filters = crate::framework::model::ImpactFilters {
                    group: group.clone(),
                    decision_state: decision_state
                        .as_ref()
                        .map(crate::framework::decision_state_filter),
                    policy_source: policy_source.clone(),
                    priority: priority.as_ref().map(crate::framework::priority_filter),
                    owner: owner.clone(),
                };
                if crate::framework::execute_impact(
                    manifest,
                    output.as_deref(),
                    format,
                    fail_on,
                    filters,
                )? {
                    Err(ForgeError::FrameworkReviewRequired)
                } else {
                    Ok(())
                }
            }
        },
        Commands::Drift { committed_artifact, generated_artifact, format } => {
            run_drift(committed_artifact, generated_artifact, format)
        }
        Commands::Trace { artifact, source, output } => {
            trace::execute(artifact, source, output.as_deref())
        }
        Commands::Profile { catalog, include, exclude, format, output, set_params, timestamp } => {
            let timestamp = timestamp.as_ref().map(chrono::DateTime::to_rfc3339);
            profile::execute(
                catalog,
                include.as_deref(),
                exclude.as_deref(),
                format,
                output.as_deref(),
                set_params,
                timestamp.as_deref(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn rejects_invalid_profile_timestamp_at_parse_time() {
        let result = Cli::try_parse_from([
            "forge",
            "profile",
            "--catalog",
            "catalog.json",
            "--include",
            "ac-1",
            "--timestamp",
            "not-a-timestamp",
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_invalid_transition_timestamp_at_parse_time() {
        let result = Cli::try_parse_from([
            "forge",
            "lifecycle",
            "transition",
            "--record",
            "record.json",
            "--to",
            "approved",
            "--actor",
            "reviewer",
            "--role",
            "reviewer",
            "--at",
            "not-a-timestamp",
            "--rationale",
            "reviewed",
        ]);
        assert!(result.is_err());
    }

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
        if let Commands::Validate {
            input,
            schema_type,
            format,
            round_trip,
            output,
            timeout,
            oscal_cli_path,
        } = cli.command
        {
            assert_eq!(input, PathBuf::from("artifact.json"));
            assert!(schema_type.is_none());
            assert_eq!(format, None, "No explicit --format; default resolved later");
            assert!(!round_trip);
            assert!(output.is_none());
            assert_eq!(timeout, None);
            assert!(oscal_cli_path.is_none());
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
            assert!(matches!(strategy, Some(Strategy::Catalog)));
            assert!(matches!(format, Some(OutputFormat::Json)));
            assert_eq!(output, Some(PathBuf::from("out.json")));
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn parse_convert_missing_strategy_is_accepted_at_parse_time() {
        // PRD 051 M-7: --strategy may be supplied by project configuration,
        // so clap no longer requires it; the resolver enforces the choice.
        let result = Cli::try_parse_from(["forge", "convert", "test.md", "--format", "json"]);
        assert!(result.is_ok(), "Parsing succeeds; requirement enforced by config resolver");
    }

    #[test]
    fn parse_convert_missing_format_defaults_to_json() {
        // T001 (EC-1): --format omitted → resolved to Json by the config layer
        let cli = Cli::try_parse_from(["forge", "convert", "test.md", "--strategy", "catalog"])
            .expect("Should succeed when --format is omitted");
        if let Commands::Convert { format, .. } = cli.command {
            assert!(format.is_none(), "No explicit --format; default resolved later");
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
            assert_eq!(jobs, Some(4));
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
            assert_eq!(jobs, Some(1));
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
            assert_eq!(jobs, Some(0));
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
            assert_eq!(jobs, None, "No explicit --jobs; default resolved later");
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
    fn parse_convert_summary_then_no_summary_resolves_false() {
        // Last flag wins (clap `overrides_with`): `--summary --no-summary`
        // must resolve to disabled.
        let cli = Cli::try_parse_from([
            "forge",
            "convert",
            "test.md",
            "--strategy",
            "catalog",
            "--summary",
            "--no-summary",
        ])
        .unwrap();
        if let Commands::Convert { summary, no_summary, .. } = cli.command {
            assert!(!summary, "`--summary` must be overridden by a later `--no-summary`");
            assert!(no_summary);
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn parse_convert_no_summary_then_summary_resolves_true() {
        // Reverse order: `--no-summary --summary` must resolve to enabled.
        let cli = Cli::try_parse_from([
            "forge",
            "convert",
            "test.md",
            "--strategy",
            "catalog",
            "--no-summary",
            "--summary",
        ])
        .unwrap();
        if let Commands::Convert { summary, no_summary, .. } = cli.command {
            assert!(summary, "`--no-summary` must be overridden by a later `--summary`");
            assert!(!no_summary);
        } else {
            panic!("Expected Convert command");
        }
    }

    // ─── T012: --import-ssp CLI parsing tests ────────────────────────────

    #[test]
    fn parse_import_ssp_flag_with_value() {
        let cli = Cli::try_parse_from([
            "forge",
            "convert",
            "test.md",
            "--strategy",
            "catalog",
            "--import-ssp",
            "./ssp.json",
        ])
        .unwrap();
        if let Commands::Convert { import_ssp, .. } = cli.command {
            assert_eq!(import_ssp, Some("./ssp.json".to_string()));
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn parse_import_ssp_flag_absent() {
        let cli =
            Cli::try_parse_from(["forge", "convert", "test.md", "--strategy", "catalog"]).unwrap();
        if let Commands::Convert { import_ssp, .. } = cli.command {
            assert!(import_ssp.is_none());
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn parse_import_ssp_flag_empty_string() {
        let cli = Cli::try_parse_from([
            "forge",
            "convert",
            "test.md",
            "--strategy",
            "catalog",
            "--import-ssp",
            "",
        ])
        .unwrap();
        if let Commands::Convert { import_ssp, .. } = cli.command {
            assert_eq!(
                import_ssp,
                Some(String::new()),
                "Clap should pass through empty string as Some(\"\")"
            );
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

    #[test]
    fn parse_drift_subcommand_with_json_output() {
        let cli = Cli::try_parse_from([
            "forge",
            "drift",
            "committed.json",
            "generated.json",
            "--format",
            "json",
        ])
        .unwrap();
        if let Commands::Drift { committed_artifact, generated_artifact, format } = cli.command {
            assert_eq!(committed_artifact, PathBuf::from("committed.json"));
            assert_eq!(generated_artifact, PathBuf::from("generated.json"));
            assert_eq!(format, DriftOutputFormat::Json);
        } else {
            panic!("Expected Drift command");
        }
    }

    #[test]
    fn parse_drift_missing_generated_artifact_fails() {
        let result = Cli::try_parse_from(["forge", "drift", "committed.json"]);
        assert!(result.is_err(), "Should fail when generated_artifact is omitted");
    }

    // ─── Issue #64: --round-trip CLI parsing tests ─────────────────────

    #[test]
    fn parse_validate_round_trip_flag() {
        let cli =
            Cli::try_parse_from(["forge", "validate", "artifact.json", "--round-trip"]).unwrap();
        if let Commands::Validate { input, round_trip, output, timeout, oscal_cli_path, .. } =
            cli.command
        {
            assert_eq!(input, PathBuf::from("artifact.json"));
            assert!(round_trip);
            assert!(output.is_none());
            assert_eq!(timeout, None);
            assert!(oscal_cli_path.is_none());
        } else {
            panic!("Expected Validate command");
        }
    }

    #[test]
    fn parse_validate_round_trip_with_output() {
        let cli = Cli::try_parse_from([
            "forge",
            "validate",
            "artifact.json",
            "--round-trip",
            "--output",
            "divergences.json",
        ])
        .unwrap();
        if let Commands::Validate { round_trip, output, .. } = cli.command {
            assert!(round_trip);
            assert_eq!(output, Some(PathBuf::from("divergences.json")));
        } else {
            panic!("Expected Validate command");
        }
    }

    #[test]
    fn parse_validate_round_trip_with_timeout() {
        let cli = Cli::try_parse_from([
            "forge",
            "validate",
            "artifact.json",
            "--round-trip",
            "--timeout",
            "120",
        ])
        .unwrap();
        if let Commands::Validate { round_trip, timeout, .. } = cli.command {
            assert!(round_trip);
            assert_eq!(timeout, Some(120));
        } else {
            panic!("Expected Validate command");
        }
    }

    #[test]
    fn parse_validate_round_trip_with_oscal_cli_path() {
        let cli = Cli::try_parse_from([
            "forge",
            "validate",
            "artifact.json",
            "--round-trip",
            "--oscal-cli-path",
            "/usr/local/bin/oscal-cli",
        ])
        .unwrap();
        if let Commands::Validate { round_trip, oscal_cli_path, .. } = cli.command {
            assert!(round_trip);
            assert_eq!(oscal_cli_path, Some(PathBuf::from("/usr/local/bin/oscal-cli")));
        } else {
            panic!("Expected Validate command");
        }
    }

    #[test]
    fn parse_validate_round_trip_with_all_options() {
        let cli = Cli::try_parse_from([
            "forge",
            "validate",
            "artifact.json",
            "--round-trip",
            "--output",
            "div.json",
            "--timeout",
            "45",
            "--oscal-cli-path",
            "/opt/oscal-cli",
            "--format",
            "json",
        ])
        .unwrap();
        if let Commands::Validate {
            input,
            round_trip,
            output,
            timeout,
            oscal_cli_path,
            format,
            ..
        } = cli.command
        {
            assert_eq!(input, PathBuf::from("artifact.json"));
            assert!(round_trip);
            assert_eq!(output, Some(PathBuf::from("div.json")));
            assert_eq!(timeout, Some(45));
            assert_eq!(oscal_cli_path, Some(PathBuf::from("/opt/oscal-cli")));
            assert_eq!(format, Some(ValidateOutputFormat::Json));
        } else {
            panic!("Expected Validate command");
        }
    }

    #[test]
    fn parse_validate_without_round_trip_defaults_false() {
        let cli = Cli::try_parse_from(["forge", "validate", "artifact.json"]).unwrap();
        if let Commands::Validate { round_trip, .. } = cli.command {
            assert!(!round_trip, "Expected --round-trip to default to false");
        } else {
            panic!("Expected Validate command");
        }
    }

    #[test]
    fn parse_validate_timeout_without_round_trip_fails() {
        let result = Cli::try_parse_from(["forge", "validate", "artifact.json", "--timeout", "60"]);
        assert!(result.is_err(), "--timeout without --round-trip should fail");
    }

    #[test]
    fn parse_validate_oscal_cli_path_without_round_trip_fails() {
        let result = Cli::try_parse_from([
            "forge",
            "validate",
            "artifact.json",
            "--oscal-cli-path",
            "/usr/bin/oscal-cli",
        ]);
        assert!(result.is_err(), "--oscal-cli-path without --round-trip should fail");
    }

    #[test]
    fn parse_validate_schema_type_with_round_trip_fails() {
        let result = Cli::try_parse_from([
            "forge",
            "validate",
            "artifact.json",
            "--round-trip",
            "--schema-type",
            "catalog",
        ]);
        assert!(result.is_err(), "--schema-type with --round-trip should conflict");
    }

    #[test]
    fn parse_validate_output_without_round_trip_succeeds() {
        let cli =
            Cli::try_parse_from(["forge", "validate", "artifact.json", "--output", "report.json"])
                .unwrap();
        if let Commands::Validate { output, round_trip, .. } = cli.command {
            assert!(!round_trip);
            assert_eq!(output, Some(PathBuf::from("report.json")));
        } else {
            panic!("Expected Validate command");
        }
    }

    // T0XX: --to ssp flag parsing tests

    #[test]
    fn parse_convert_to_ssp() {
        let cli = Cli::try_parse_from([
            "forge",
            "convert",
            "policy.md",
            "--strategy",
            "catalog",
            "--to",
            "ssp",
        ])
        .unwrap();
        if let Commands::Convert { to, .. } = cli.command {
            assert_eq!(to, Some(OutputType::Ssp));
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn parse_convert_to_component_definition() {
        let cli = Cli::try_parse_from([
            "forge",
            "convert",
            "policy.md",
            "--strategy",
            "catalog",
            "--to",
            "component-definition",
        ])
        .unwrap();
        if let Commands::Convert { to, .. } = cli.command {
            assert_eq!(to, Some(OutputType::ComponentDefinition));
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn parse_convert_to_catalog() {
        let cli = Cli::try_parse_from([
            "forge",
            "convert",
            "policy.md",
            "--strategy",
            "component",
            "--to",
            "catalog",
        ])
        .unwrap();
        if let Commands::Convert { to, .. } = cli.command {
            assert_eq!(to, Some(OutputType::Catalog));
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn parse_convert_without_to_defaults_to_none() {
        let cli = Cli::try_parse_from(["forge", "convert", "policy.md", "--strategy", "catalog"])
            .unwrap();
        if let Commands::Convert { to, .. } = cli.command {
            assert!(to.is_none(), "--to should default to None when not provided");
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn parse_convert_to_ssp_with_output_file() {
        let cli = Cli::try_parse_from([
            "forge",
            "convert",
            "policy.md",
            "--strategy",
            "catalog",
            "--to",
            "ssp",
            "--output",
            "ssp.json",
        ])
        .unwrap();
        if let Commands::Convert { to, output, .. } = cli.command {
            assert_eq!(to, Some(OutputType::Ssp));
            assert_eq!(output, Some(PathBuf::from("ssp.json")));
        } else {
            panic!("Expected Convert command");
        }
    }

    #[test]
    fn parse_convert_rejects_out_of_range_max_size() {
        for value in ["0", "51201"] {
            let error =
                match Cli::try_parse_from(["forge", "convert", "policy.md", "--max-size", value]) {
                    Ok(_) => panic!("out-of-range --max-size {value} unexpectedly parsed"),
                    Err(error) => error.to_string(),
                };

            assert!(error.contains("1..=51200"), "unexpected error: {error}");
        }
    }
}
