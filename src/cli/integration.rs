//! `forge integration` subcommand handlers (PRD 065 Phase 1).
//!
//! Export connector-neutral change sets from existing FORGE reports, dry-run
//! plan them, apply the exact plan through the fake loopback transport, and
//! reconcile remote observations. No live outbound transport exists in Phase 1.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use clap::Subcommand;

use crate::ForgeError;
use crate::cli::output::write_output;
use crate::workflow::{change_set, config, fake, plan, read_bounded};

/// Explicit integration operations.
#[derive(Subcommand)]
pub enum IntegrationCommand {
    /// Export a deterministic connector-neutral change set from existing FORGE reports
    Export {
        /// Project root used for repository-relative source links
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Framework impact report (forge.framework-impact-report/1); repeatable
        #[arg(long = "framework-report")]
        framework_reports: Vec<PathBuf>,
        /// Stored applicability report (forge.applicability-report/1); repeatable
        #[arg(long = "applicability-report")]
        applicability_reports: Vec<PathBuf>,
        /// Lifecycle record (forge.policy-lifecycle/1 or /2); repeatable
        #[arg(long = "lifecycle-record")]
        lifecycle_records: Vec<PathBuf>,
        /// POA&M workflow manifest (forge.poam/1); repeatable
        #[arg(long = "poam-manifest")]
        poam_manifests: Vec<PathBuf>,
        /// Portable review queue (forge.review-queue/1); repeatable
        #[arg(long = "review-queue")]
        review_queues: Vec<PathBuf>,
        /// Opt in to bounded source excerpts (default payloads are minimized IDs/links)
        #[arg(long)]
        include_excerpts: bool,
        /// Write the change set to a file instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Dry-run plan: bind a change set to allowlisted targets and content hashes
    Plan {
        /// Project root used to re-verify source artifact hashes
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Change set produced by `integration export`
        #[arg(long)]
        change_set: PathBuf,
        /// Connector configuration with the explicit target allowlist
        #[arg(long = "connector-config")]
        connector_config: PathBuf,
        /// Local connector state file (created empty when absent)
        #[arg(long)]
        state: PathBuf,
        /// Explicit plan timestamp (RFC 3339); defaults to now
        #[arg(long)]
        as_of: Option<DateTime<Utc>>,
        /// Plan validity window in hours
        #[arg(long, default_value_t = plan::DEFAULT_TTL_HOURS)]
        ttl_hours: i64,
        /// Write the plan to a file instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Apply the exact unexpired dry-run plan through the fake Phase 1 transport
    Apply {
        /// Project root used to re-verify source artifact hashes
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Change set the plan was built from
        #[arg(long)]
        change_set: PathBuf,
        /// Connector configuration the plan was built from
        #[arg(long = "connector-config")]
        connector_config: PathBuf,
        /// Dry-run plan produced by `integration plan`
        #[arg(long)]
        plan: PathBuf,
        /// Local connector state file (updated only for successful operations)
        #[arg(long)]
        state: PathBuf,
        /// Explicit apply timestamp (RFC 3339); defaults to now
        #[arg(long)]
        as_of: Option<DateTime<Utc>>,
        /// Confirm the printed target/count/sensitivity summary and apply
        #[arg(long)]
        confirm: bool,
        /// Write the redacted audit report to a file instead of stdout
        #[arg(long)]
        report: Option<PathBuf>,
    },
    /// Reconcile recorded remote identities as observations only (never mutates FORGE sources)
    Reconcile {
        /// Connector configuration with the explicit target allowlist
        #[arg(long = "connector-config")]
        connector_config: PathBuf,
        /// Local connector state file
        #[arg(long)]
        state: PathBuf,
        /// Write the redacted audit report to a file instead of stdout
        #[arg(long)]
        report: Option<PathBuf>,
    },
}

/// Execute one integration subcommand.
///
/// # Errors
///
/// Returns [`ForgeError`] when inputs are invalid, a plan gate fails, or output
/// cannot be written. Per-operation transport failures are recorded in the
/// redacted audit report rather than returned.
#[allow(clippy::too_many_lines)] // Flat exhaustive dispatch keeps every clap variant visible here.
pub fn execute(command: &IntegrationCommand) -> Result<(), ForgeError> {
    match command {
        IntegrationCommand::Export {
            root,
            framework_reports,
            applicability_reports,
            lifecycle_records,
            poam_manifests,
            review_queues,
            include_excerpts,
            output,
        } => {
            if framework_reports.is_empty()
                && applicability_reports.is_empty()
                && lifecycle_records.is_empty()
                && poam_manifests.is_empty()
                && review_queues.is_empty()
            {
                return Err(ForgeError::Integration(
                    "export requires at least one source report or record".to_string(),
                ));
            }
            let options = change_set::ExportOptions { root, include_excerpts: *include_excerpts };
            let mut items = Vec::new();
            for path in framework_reports {
                items.extend(change_set::export_framework_report(path, &options)?);
            }
            for path in applicability_reports {
                items.extend(change_set::export_applicability_report(path, &options)?);
            }
            items.extend(change_set::export_lifecycle_records(lifecycle_records, &options)?);
            for path in poam_manifests {
                items.extend(change_set::export_poam_milestones(path, &options)?);
            }
            for path in review_queues {
                items.extend(change_set::export_review_queue(path, &options)?);
            }
            let set = change_set::ChangeSet::new(items)?;
            write_output(&set.render()?, output.as_deref())
        }
        IntegrationCommand::Plan {
            root,
            change_set: set_path,
            connector_config: config_path,
            state: state_path,
            as_of,
            ttl_hours,
            output,
        } => {
            let config_bytes = read_bounded(config_path, config::MAX_CONFIG_BYTES)?;
            let connector = config::ConnectorConfig::parse(&config_bytes)?;
            let set_bytes = read_bounded(set_path, change_set::MAX_CHANGE_SET_BYTES)?;
            let set = change_set::parse(&set_bytes)?;
            let state = plan::load_state(state_path)?;
            let now = as_of.unwrap_or_else(Utc::now);
            let document = plan::build_plan(
                root,
                &connector,
                &config_bytes,
                &set,
                &set_bytes,
                &state,
                now,
                *ttl_hours,
            )?;
            write_output(&document.render()?, output.as_deref())
        }
        IntegrationCommand::Apply {
            root,
            change_set: set_path,
            connector_config: config_path,
            plan: plan_path,
            state: state_path,
            as_of,
            confirm,
            report,
        } => {
            let config_bytes = read_bounded(config_path, config::MAX_CONFIG_BYTES)?;
            let connector = config::ConnectorConfig::parse(&config_bytes)?;
            let set_bytes = read_bounded(set_path, change_set::MAX_CHANGE_SET_BYTES)?;
            let set = change_set::parse(&set_bytes)?;
            let plan_bytes = read_bounded(plan_path, plan::MAX_DOCUMENT_BYTES)?;
            let document = plan::Plan::parse(&plan_bytes)?;
            let state = plan::load_state(state_path)?;
            let now = as_of.unwrap_or_else(Utc::now);
            plan::verify_plan_binding(&document, &connector, &config_bytes, &set_bytes, now)?;
            plan::verify_sources(root, &set)?;
            if !confirm {
                return write_output(
                    &plan::confirmation_summary(&connector, &document, &set),
                    None,
                );
            }
            // Phase 1 transport: in-process loopback fake only (M-15). Phase 2
            // replaces this with the approved live HTTPS adapter.
            let server = fake::FakeServer::start()?;
            let mut transport =
                fake::FakeTransport::new(server.addr(), Box::new(fake::FakeCredentials));
            let (next_state, audit) = plan::apply(
                &connector,
                &config_bytes,
                &document,
                &plan_bytes,
                &set_bytes,
                &state,
                &mut transport,
                now,
            )?;
            crate::io::write_atomic(state_path, next_state.render()?.as_bytes())?;
            write_output(&audit.render()?, report.as_deref())
        }
        IntegrationCommand::Reconcile {
            connector_config: config_path,
            state: state_path,
            report,
        } => {
            let connector = config::load(config_path)?;
            let state = plan::load_state(state_path)?;
            let server = fake::FakeServer::start()?;
            let mut transport =
                fake::FakeTransport::new(server.addr(), Box::new(fake::FakeCredentials));
            let audit = plan::reconcile(&connector, &state, &mut transport)?;
            write_output(&audit.render()?, report.as_deref())
        }
    }
}
