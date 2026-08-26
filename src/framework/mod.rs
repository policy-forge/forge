//! Read-only framework revision impact analysis.

pub mod analysis;
pub mod disposition;
pub mod manifest;
pub mod model;

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::cli::{
    FrameworkDecisionStateFilter, FrameworkFailOn, FrameworkImpactPriorityFilter,
    FrameworkReportFormat,
};
use crate::{ForgeError, io};

#[must_use]
pub(crate) const fn decision_state_filter(
    value: &FrameworkDecisionStateFilter,
) -> crate::applicability::manifest::DecisionState {
    match value {
        FrameworkDecisionStateFilter::Applicable => {
            crate::applicability::manifest::DecisionState::Applicable
        }
        FrameworkDecisionStateFilter::NotApplicable => {
            crate::applicability::manifest::DecisionState::NotApplicable
        }
        FrameworkDecisionStateFilter::Deferred => {
            crate::applicability::manifest::DecisionState::Deferred
        }
        FrameworkDecisionStateFilter::UnderReview => {
            crate::applicability::manifest::DecisionState::UnderReview
        }
    }
}

#[must_use]
pub(crate) const fn priority_filter(
    value: &FrameworkImpactPriorityFilter,
) -> model::FindingPriority {
    match value {
        FrameworkImpactPriorityFilter::Blocking => model::FindingPriority::Blocking,
        FrameworkImpactPriorityFilter::ReviewRequired => model::FindingPriority::ReviewRequired,
        FrameworkImpactPriorityFilter::Informational => model::FindingPriority::Informational,
    }
}

/// Execute `forge framework impact` and return whether the selected gate fires.
///
/// # Errors
///
/// Returns [`ForgeError::FrameworkImpact`] when the analysis cannot complete or an output aliases
/// an input. No report is written until every input and destination check succeeds.
pub fn execute_impact(
    manifest_path: &Path,
    output: Option<&Path>,
    format: &FrameworkReportFormat,
    fail_on: &FrameworkFailOn,
    filters: model::ImpactFilters,
) -> Result<bool, ForgeError> {
    crate::io::regular_file_metadata(manifest_path, "manifest").map_err(impact_error)?;
    io::check_file_size(manifest_path, manifest::MAX_MANIFEST_BYTES)
        .map_err(|error| impact_error(format!("manifest: {error}")))?;
    let bytes =
        std::fs::read(manifest_path).map_err(|error| impact_error(format!("manifest: {error}")))?;
    let manifest = manifest::parse(&bytes)?;
    let manifest_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let (report, mut inputs) = analysis::analyze(manifest_dir, &manifest, filters)?;
    inputs.push(manifest_path.to_path_buf());
    validate_destination(&inputs, output)?;
    let rendered = render_report(&report, format)?;
    crate::cli::output::write_output(&rendered, output)?;
    Ok(gate_fires(&report, fail_on))
}

fn render_report(
    report: &model::ImpactReport,
    format: &FrameworkReportFormat,
) -> Result<String, ForgeError> {
    match format {
        FrameworkReportFormat::Json => {
            let mut output = serde_json::to_string_pretty(report)
                .map_err(|error| impact_error(format!("report serialization failed: {error}")))?;
            output.push('\n');
            Ok(output)
        }
        FrameworkReportFormat::Text => Ok(render_text(report)),
        FrameworkReportFormat::Markdown => Ok(render_markdown(report)),
        FrameworkReportFormat::Html => Ok(render_html(report)),
        FrameworkReportFormat::Github => Ok(render_github_annotations(report)),
    }
}

fn render_markdown(report: &model::ImpactReport) -> String {
    let mut output = String::new();
    output.push_str("# FORGE framework change impact report\n\n");
    let _ = writeln!(output, "- Schema: {}", markdown_escape(report.schema_version));
    let _ = writeln!(output, "- Status: {}", markdown_escape(report.status));
    output.push_str("\n## Resources\n\n");
    output.push_str(
        "| Revision | Type | SHA-256 | Root UUID | Document version | OSCAL version | Resolved catalog SHA-256 |\n",
    );
    output.push_str("| --- | --- | --- | --- | --- | --- | --- |\n");
    for (revision, resource) in [("Old", &report.old), ("New", &report.new)] {
        let _ = writeln!(
            output,
            "| {revision} | {} | {} | {} | {} | {} | {} |",
            resource.resource_type.as_str(),
            markdown_escape(&resource.raw_sha256),
            markdown_escape(&resource.root_uuid),
            markdown_escape(&resource.document_version),
            markdown_escape(&resource.oscal_version),
            resource
                .resolved_catalog_sha256
                .as_deref()
                .map_or_else(|| "none".to_owned(), markdown_escape)
        );
    }

    output.push_str("\n## Summary\n\n");
    output.push_str("| Metric | Count |\n| --- | ---: |\n");
    for (metric, count) in summary_counts(report) {
        let _ = writeln!(output, "| {metric} | {count} |");
    }

    output.push_str("\n## Control changes\n\n");
    output.push_str("| Classification | Control | Old SHA-256 | New SHA-256 |\n");
    output.push_str("| --- | --- | --- | --- |\n");
    for change in &report.changes {
        let _ = writeln!(
            output,
            "| {} | {} | {} | {} |",
            change.change_class.as_str(),
            markdown_escape(&change.subject_id),
            change.old_sha256.as_deref().unwrap_or("none"),
            change.new_sha256.as_deref().unwrap_or("none")
        );
    }

    output.push_str("\n## Review findings\n\n");
    output.push_str(
        "| Priority | Reason | Finding | Control | Action | Disposition | Dependency path |\n",
    );
    output.push_str("| --- | --- | --- | --- | --- | --- | --- |\n");
    for finding in &report.findings {
        let _ = writeln!(
            output,
            "| {} | {} | {} | {} | {} | {} | {} |",
            finding.priority.as_str(),
            finding.reason_code.as_str(),
            markdown_escape(&finding.finding_id),
            markdown_escape(&finding.subject_id),
            finding.required_action.as_str(),
            finding.disposition.as_ref().map_or("none", |value| value.status.as_str()),
            finding
                .dependency_path
                .iter()
                .map(|segment| markdown_escape(segment))
                .collect::<Vec<_>>()
                .join(" -> ")
        );
    }
    output
}

fn render_html(report: &model::ImpactReport) -> String {
    let mut output = String::from(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>FORGE framework change impact report</title>\n</head>\n<body>\n<h1>FORGE framework change impact report</h1>\n",
    );
    let _ = writeln!(
        output,
        "<dl><dt>Schema</dt><dd><code>{}</code></dd><dt>Status</dt><dd><code>{}</code></dd></dl>",
        html_escape(report.schema_version),
        html_escape(report.status)
    );
    output.push_str("<h2>Resources</h2>\n<table>\n<thead><tr><th>Revision</th><th>Type</th><th>SHA-256</th><th>Root UUID</th><th>Document version</th><th>OSCAL version</th><th>Resolved catalog SHA-256</th></tr></thead>\n<tbody>\n");
    for (revision, resource) in [("Old", &report.old), ("New", &report.new)] {
        let _ = writeln!(
            output,
            "<tr><th>{revision}</th><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            resource.resource_type.as_str(),
            html_escape(&resource.raw_sha256),
            html_escape(&resource.root_uuid),
            html_escape(&resource.document_version),
            html_escape(&resource.oscal_version),
            resource
                .resolved_catalog_sha256
                .as_deref()
                .map_or_else(|| "none".to_owned(), html_escape)
        );
    }
    output.push_str("</tbody>\n</table>\n<h2>Summary</h2>\n<table>\n<thead><tr><th>Metric</th><th>Count</th></tr></thead>\n<tbody>\n");
    for (metric, count) in summary_counts(report) {
        let _ = writeln!(output, "<tr><th>{metric}</th><td>{count}</td></tr>");
    }
    output.push_str("</tbody>\n</table>\n<h2>Control changes</h2>\n<table>\n<thead><tr><th>Classification</th><th>Control</th><th>Old SHA-256</th><th>New SHA-256</th></tr></thead>\n<tbody>\n");
    for change in &report.changes {
        let _ = writeln!(
            output,
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            change.change_class.as_str(),
            html_escape(&change.subject_id),
            change.old_sha256.as_deref().unwrap_or("none"),
            change.new_sha256.as_deref().unwrap_or("none")
        );
    }
    output.push_str("</tbody>\n</table>\n<h2>Review findings</h2>\n<table>\n<thead><tr><th>Priority</th><th>Reason</th><th>Finding</th><th>Control</th><th>Action</th><th>Disposition</th><th>Dependency path</th></tr></thead>\n<tbody>\n");
    for finding in &report.findings {
        let _ = writeln!(
            output,
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            finding.priority.as_str(),
            finding.reason_code.as_str(),
            html_escape(&finding.finding_id),
            html_escape(&finding.subject_id),
            finding.required_action.as_str(),
            finding.disposition.as_ref().map_or("none", |value| value.status.as_str()),
            html_escape(&finding.dependency_path.join(" -> "))
        );
    }
    output.push_str("</tbody>\n</table>\n</body>\n</html>\n");
    output
}

fn summary_counts(report: &model::ImpactReport) -> [(&'static str, usize); 18] {
    [
        ("Old controls", report.summary.old_controls),
        ("New controls", report.summary.new_controls),
        ("Added", report.summary.added),
        ("Removed", report.summary.removed),
        ("Content changed", report.summary.content_changed),
        ("Identity migrated", report.summary.identity_migrated),
        ("Unchanged", report.summary.unchanged),
        ("Findings", report.summary.findings),
        ("Blocking", report.summary.blocking),
        ("Review required", report.summary.review_required),
        ("Informational", report.summary.informational),
        ("Resolved", report.summary.dispositioned_resolved),
        ("Accepted risk", report.summary.dispositioned_accepted_risk),
        ("Still open", report.summary.dispositioned_still_open),
        ("Undispositioned", report.summary.undispositioned),
        ("Prior-only dispositions", report.prior_only_dispositions.len()),
        ("Control changes", report.changes.len()),
        ("Review findings", report.findings.len()),
    ]
}

fn markdown_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '\n' => escaped.push_str("&#10;"),
            '\r' => escaped.push_str("&#13;"),
            '\\' | '`' | '*' | '_' | '{' | '}' | '[' | ']' | '(' | ')' | '#' | '+' | '-' | '.'
            | '!' | '|' => {
                escaped.push('\\');
                escaped.push(character);
            }
            _ => escaped.push(character),
        }
    }
    escaped
}

fn html_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn render_github_annotations(report: &model::ImpactReport) -> String {
    let mut output = String::new();
    for finding in &report.findings {
        let closed = finding.disposition.as_ref().is_some_and(|disposition| {
            matches!(
                disposition.status,
                disposition::DispositionStatus::Resolved
                    | disposition::DispositionStatus::AcceptedRisk
            )
        });
        let command = if closed {
            "notice"
        } else {
            match finding.priority {
                model::FindingPriority::Blocking => "error",
                model::FindingPriority::ReviewRequired => "warning",
                model::FindingPriority::Informational => "notice",
            }
        };
        let title =
            github_property(&format!("FORGE framework impact: {}", finding.reason_code.as_str()));
        let disposition =
            finding.disposition.as_ref().map_or("none", |value| value.status.as_str());
        let message = github_data(&format!(
            "finding={} control={} change={} action={} disposition={} path={}",
            finding.finding_id,
            finding.subject_id,
            finding.change_class.as_str(),
            finding.required_action.as_str(),
            disposition,
            finding.dependency_path.join(" -> ")
        ));
        let _ = writeln!(output, "::{command} title={title}::{message}");
    }
    output
}

fn github_data(value: &str) -> String {
    value.replace('%', "%25").replace('\r', "%0D").replace('\n', "%0A")
}

fn github_property(value: &str) -> String {
    github_data(value).replace(':', "%3A").replace(',', "%2C")
}

fn render_text(report: &model::ImpactReport) -> String {
    let mut output = String::new();
    output.push_str("FORGE framework change impact report\n");
    let _ = writeln!(output, "schema: {}", report.schema_version);
    let _ = writeln!(output, "status: {}", report.status);
    let _ = writeln!(
        output,
        "old: sha256={} root-uuid={} version={} oscal-version={}",
        report.old.raw_sha256,
        report.old.root_uuid,
        escape(&report.old.document_version),
        escape(&report.old.oscal_version)
    );
    let _ = writeln!(
        output,
        "new: sha256={} root-uuid={} version={} oscal-version={}",
        report.new.raw_sha256,
        report.new.root_uuid,
        escape(&report.new.document_version),
        escape(&report.new.oscal_version)
    );
    let _ = writeln!(
        output,
        "controls: old={} new={} added={} removed={} content-changed={} identity-migrated={} unchanged={}",
        report.summary.old_controls,
        report.summary.new_controls,
        report.summary.added,
        report.summary.removed,
        report.summary.content_changed,
        report.summary.identity_migrated,
        report.summary.unchanged
    );
    let _ = writeln!(
        output,
        "review queue: total={} blocking={} review-required={} informational={}",
        report.summary.findings,
        report.summary.blocking,
        report.summary.review_required,
        report.summary.informational
    );
    let _ = writeln!(
        output,
        "dispositions: resolved={} accepted-risk={} still-open={} undispositioned={} prior-only={}",
        report.summary.dispositioned_resolved,
        report.summary.dispositioned_accepted_risk,
        report.summary.dispositioned_still_open,
        report.summary.undispositioned,
        report.prior_only_dispositions.len()
    );
    let _ = writeln!(output, "control changes: {}", report.changes.len());
    for change in &report.changes {
        let _ = writeln!(
            output,
            "- {} control={} old-sha256={} new-sha256={}",
            change.change_class.as_str(),
            escape(&change.subject_id),
            change.old_sha256.as_deref().unwrap_or("none"),
            change.new_sha256.as_deref().unwrap_or("none")
        );
    }
    let _ = writeln!(output, "review findings: {}", report.findings.len());
    for finding in &report.findings {
        let _ = writeln!(
            output,
            "- {} {} {} control={} action={} disposition={} path={}",
            finding.priority.as_str(),
            finding.reason_code.as_str(),
            finding.finding_id,
            escape(&finding.subject_id),
            finding.required_action.as_str(),
            finding.disposition.as_ref().map_or("none", |disposition| disposition.status.as_str()),
            finding
                .dependency_path
                .iter()
                .map(|value| escape(value))
                .collect::<Vec<_>>()
                .join(" -> ")
        );
    }
    output
}

fn gate_fires(report: &model::ImpactReport, fail_on: &FrameworkFailOn) -> bool {
    report.findings.iter().chain(&report.filtered_out_findings).any(|finding| {
        let remains_open = finding.disposition.as_ref().is_none_or(|disposition| {
            disposition.status == disposition::DispositionStatus::StillOpen
        });
        remains_open
            && match fail_on {
                FrameworkFailOn::Blocking => finding.priority == model::FindingPriority::Blocking,
                FrameworkFailOn::ReviewRequired => matches!(
                    finding.priority,
                    model::FindingPriority::Blocking | model::FindingPriority::ReviewRequired
                ),
                FrameworkFailOn::Any => true,
            }
    })
}

fn validate_destination(inputs: &[PathBuf], output: Option<&Path>) -> Result<(), ForgeError> {
    let Some(output) = output else { return Ok(()) };
    for input in inputs {
        if paths_alias(output, input)? {
            return Err(impact_error(format!(
                "output '{}' aliases a framework impact input",
                output.display()
            )));
        }
    }
    Ok(())
}

fn paths_alias(left: &Path, right: &Path) -> Result<bool, ForgeError> {
    crate::mapping::paths_alias(left, right).map_err(|error| {
        impact_error(error.to_string().replace("Control Mapping build error: ", ""))
    })
}

fn escape(value: &str) -> String {
    value.chars().flat_map(char::escape_default).collect()
}

fn impact_error(message: impl Into<String>) -> ForgeError {
    ForgeError::FrameworkImpact(message.into())
}

#[cfg(test)]
mod tests {
    use crate::applicability::manifest::DecisionState;
    use crate::cli::{
        FrameworkDecisionStateFilter, FrameworkFailOn, FrameworkImpactPriorityFilter,
    };
    use crate::framework::disposition::{DispositionRecord, DispositionStatus};
    use crate::framework::model::{
        ChangeClass, ChangeSummary, ControlChange, FindingPriority, ImpactFinding, ImpactReport,
        ReasonCode, RequiredAction,
    };
    use crate::mapping::inventory::ResourceEvidence;
    use crate::mapping::manifest::ResourceType;

    use super::{
        decision_state_filter, gate_fires, github_data, github_property, html_escape,
        markdown_escape, priority_filter, render_html, render_markdown,
    };

    #[test]
    fn github_workflow_command_fields_are_escaped() {
        assert_eq!(github_data("100%\nnext\r"), "100%25%0Anext%0D");
        assert_eq!(github_property("a:b,c%"), "a%3Ab%2Cc%25");
    }

    #[test]
    fn cli_filters_map_exhaustively_to_report_model_values() {
        assert_eq!(
            decision_state_filter(&FrameworkDecisionStateFilter::Applicable),
            DecisionState::Applicable
        );
        assert_eq!(
            decision_state_filter(&FrameworkDecisionStateFilter::NotApplicable),
            DecisionState::NotApplicable
        );
        assert_eq!(
            decision_state_filter(&FrameworkDecisionStateFilter::Deferred),
            DecisionState::Deferred
        );
        assert_eq!(
            decision_state_filter(&FrameworkDecisionStateFilter::UnderReview),
            DecisionState::UnderReview
        );
        assert_eq!(
            priority_filter(&FrameworkImpactPriorityFilter::Blocking),
            FindingPriority::Blocking
        );
        assert_eq!(
            priority_filter(&FrameworkImpactPriorityFilter::ReviewRequired),
            FindingPriority::ReviewRequired
        );
        assert_eq!(
            priority_filter(&FrameworkImpactPriorityFilter::Informational),
            FindingPriority::Informational
        );
    }

    #[test]
    fn markdown_and_html_reports_are_deterministic_and_escape_injected_content() {
        let report = report_with_injected_content();

        let markdown = render_markdown(&report);
        assert_eq!(markdown, render_markdown(&report));
        assert!(markdown.starts_with("# FORGE framework change impact report\n"));
        assert!(markdown.contains("- Status: complete"));
        assert!(markdown.contains("control\\|with\\*markup&lt;script&gt;"));
        assert!(markdown.contains("source&#10;\\|row -> &lt;img src=x onerror=alert\\(1\\)&gt;"));
        assert!(!markdown.contains("<script>"));
        assert!(!markdown.contains("\n|row"));

        let html = render_html(&report);
        assert_eq!(html, render_html(&report));
        assert!(html.starts_with("<!doctype html>\n<html lang=\"en\">"));
        assert!(html.contains("<dt>Status</dt><dd><code>complete</code></dd>"));
        assert!(html.contains("control|with*markup&lt;script&gt;"));
        assert!(html.contains("source\n|row"));
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<img src=x onerror=alert(1)>"));
        assert!(html.ends_with("</html>\n"));
    }

    #[test]
    fn report_escaping_covers_markdown_and_html_control_characters() {
        assert_eq!(
            markdown_escape("a|b_*<tag>&\nnext\\"),
            "a\\|b\\_\\*&lt;tag&gt;&amp;&#10;next\\\\"
        );
        assert_eq!(
            html_escape("<tag a=\"b\">Tom & 'Sue'</tag>"),
            "&lt;tag a=&quot;b&quot;&gt;Tom &amp; &#39;Sue&#39;&lt;/tag&gt;"
        );
    }

    #[test]
    fn gates_cover_every_priority_disposition_and_filtered_out_findings() {
        let mut report = report_with_injected_content();
        let gates =
            [FrameworkFailOn::Blocking, FrameworkFailOn::ReviewRequired, FrameworkFailOn::Any];
        for (priority, expected) in [
            (FindingPriority::Blocking, [true, true, true]),
            (FindingPriority::ReviewRequired, [false, true, true]),
            (FindingPriority::Informational, [false, false, true]),
        ] {
            report.findings[0].priority = priority;
            for (gate, expected) in gates.iter().zip(expected) {
                assert_eq!(gate_fires(&report, gate), expected, "priority={priority:?}");
            }
        }

        report.findings[0].priority = FindingPriority::Blocking;
        for status in [DispositionStatus::Resolved, DispositionStatus::AcceptedRisk] {
            report.findings[0].disposition = Some(disposition(status));
            assert!(gates.iter().all(|gate| !gate_fires(&report, gate)));
        }
        report.findings[0].disposition = Some(disposition(DispositionStatus::StillOpen));
        assert!(gates.iter().all(|gate| gate_fires(&report, gate)));

        report.findings[0].disposition = None;
        report.filtered_out_findings = std::mem::take(&mut report.findings);
        assert!(report.findings.is_empty());
        for (priority, expected) in [
            (FindingPriority::Blocking, [true, true, true]),
            (FindingPriority::ReviewRequired, [false, true, true]),
            (FindingPriority::Informational, [false, false, true]),
        ] {
            report.filtered_out_findings[0].priority = priority;
            for (gate, expected) in gates.iter().zip(expected) {
                assert_eq!(gate_fires(&report, gate), expected, "filtered priority={priority:?}");
            }
        }
    }

    fn disposition(status: DispositionStatus) -> DispositionRecord {
        DispositionRecord {
            finding_id: "00000000-0000-4000-8000-000000000002".to_owned(),
            status,
            decided_by: "reviewer".to_owned(),
            decided_at: "2026-08-25T12:00:00Z".to_owned(),
            rationale: "reviewed".to_owned(),
        }
    }

    fn report_with_injected_content() -> ImpactReport {
        let resource = ResourceEvidence {
            resource_type: ResourceType::Catalog,
            href: "ignored.json".to_owned(),
            raw_sha256: "a".repeat(64),
            root_uuid: "00000000-0000-4000-8000-000000000001".to_owned(),
            document_version: "1<script>".to_owned(),
            oscal_version: "1.2.3".to_owned(),
            resolved_catalog_sha256: None,
        };
        ImpactReport {
            schema_version: crate::framework::model::REPORT_SCHEMA_VERSION,
            status: "complete",
            old: resource.clone(),
            new: resource,
            summary: ChangeSummary {
                old_controls: 1,
                new_controls: 1,
                content_changed: 1,
                findings: 1,
                review_required: 1,
                undispositioned: 1,
                ..ChangeSummary::default()
            },
            filters: crate::framework::model::ImpactFilters::default(),
            matched_findings: 1,
            changes: vec![ControlChange {
                subject_id: "control|with*markup<script>".to_owned(),
                change_class: ChangeClass::ContentChanged,
                old_sha256: Some("b".repeat(64)),
                new_sha256: Some("c".repeat(64)),
                old_subjects: Vec::new(),
                new_subjects: Vec::new(),
                migration: None,
            }],
            findings: vec![ImpactFinding {
                finding_id: "00000000-0000-4000-8000-000000000002".to_owned(),
                priority: FindingPriority::ReviewRequired,
                reason_code: ReasonCode::ControlContentChanged,
                required_action: RequiredAction::ReviewControlChange,
                subject_id: "control|with*markup<script>".to_owned(),
                change_class: ChangeClass::ContentChanged,
                old_sha256: Some("b".repeat(64)),
                new_sha256: Some("c".repeat(64)),
                old_subjects: Vec::new(),
                new_subjects: Vec::new(),
                migration: None,
                dependency_path: vec![
                    "source\n|row".to_owned(),
                    "<img src=x onerror=alert(1)>".to_owned(),
                ],
                affected_artifact_id: None,
                dependency_id: None,
                policy_resource_identity: None,
                prior_gap_classification: None,
                prior_decision_state: None,
                owner: None,
                policy_sources: Vec::new(),
                framework_groups: Vec::new(),
                disposition: None,
            }],
            filtered_out_findings: Vec::new(),
            prior_only_dispositions: Vec::new(),
        }
    }
}
