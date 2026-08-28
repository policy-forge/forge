//! Deterministic, content-minimizing Assessment Results review reports.

use std::fmt::Write as _;

use serde::Serialize;

use super::context::ArtifactIdentity;
use super::manifest::{AssessmentResultsManifest, ConclusionType};
use crate::ForgeError;

/// Versioned machine-readable report contract.
pub const REPORT_SCHEMA_VERSION: &str = "forge.assessment-results-report/1";

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ReportStatus {
    Complete,
    ReviewRequired,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssessmentResultsReport {
    pub schema_version: &'static str,
    pub status: ReportStatus,
    pub artifact_uuid: String,
    pub counts: ReportCounts,
    pub validation: ValidationSummary,
    pub context: Vec<ContextSummary>,
    pub findings: Vec<BaselineFinding>,
    pub trust_boundary: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportCounts {
    pub observations: usize,
    pub findings: usize,
    pub risks: usize,
    pub relationships: usize,
    pub evidence_references: usize,
}

#[derive(Debug, Clone, Serialize)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "the versioned report exposes independent validation gates to machine consumers"
)]
pub struct ValidationSummary {
    pub manifest_valid: bool,
    pub context_valid: bool,
    pub references_valid: bool,
    pub graph_valid: bool,
    pub assessment_results_schema_valid: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextSummary {
    pub kind: &'static str,
    pub sha256: String,
    pub root_uuid: String,
    pub document_version: String,
    pub oscal_version: String,
}

impl From<&ArtifactIdentity> for ContextSummary {
    fn from(identity: &ArtifactIdentity) -> Self {
        Self {
            kind: identity.kind,
            sha256: identity.sha256.clone(),
            root_uuid: identity.root_uuid.clone(),
            document_version: identity.document_version.clone(),
            oscal_version: identity.oscal_version.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BaselineFinding {
    pub id: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_type: Option<ConclusionType>,
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_fingerprint: Option<String>,
}

impl AssessmentResultsReport {
    #[must_use]
    pub fn new(
        manifest: &AssessmentResultsManifest,
        artifact_uuid: String,
        identities: impl IntoIterator<Item = ContextSummary>,
    ) -> Self {
        Self {
            schema_version: REPORT_SCHEMA_VERSION,
            status: ReportStatus::Complete,
            artifact_uuid,
            counts: ReportCounts {
                observations: manifest.result.observations.len(),
                findings: manifest.result.findings.len(),
                risks: manifest.result.risks.len(),
                relationships: manifest.result.relationships.len(),
                evidence_references: manifest
                    .result
                    .observations
                    .iter()
                    .map(|observation| observation.evidence_keys.len())
                    .sum(),
            },
            validation: ValidationSummary {
                manifest_valid: true,
                context_valid: true,
                references_valid: true,
                graph_valid: true,
                assessment_results_schema_valid: true,
            },
            context: identities.into_iter().collect(),
            findings: Vec::new(),
            trust_boundary: "All conclusions are reviewer-authored. FORGE validates structure, identity, and references only.",
        }
    }

    pub fn finalize(&mut self) {
        self.findings.sort_by(|left, right| {
            (&left.code, &left.object_type, &left.key, &left.id).cmp(&(
                &right.code,
                &right.object_type,
                &right.key,
                &right.id,
            ))
        });
        self.status = if self.findings.is_empty() {
            ReportStatus::Complete
        } else {
            ReportStatus::ReviewRequired
        };
    }
}

/// Render the report in the selected CLI format.
///
/// # Errors
///
/// Returns an error only if the versioned report cannot be serialized as JSON.
pub fn render(
    report: &AssessmentResultsReport,
    format: &crate::cli::AssessmentResultsReportFormat,
) -> Result<String, ForgeError> {
    match format {
        crate::cli::AssessmentResultsReportFormat::Json => {
            let mut output = serde_json::to_string_pretty(report).map_err(|cause| {
                ForgeError::AssessmentResultsBuild(format!(
                    "review report serialization failed: {cause}"
                ))
            })?;
            output.push('\n');
            Ok(output)
        }
        crate::cli::AssessmentResultsReportFormat::Text => Ok(render_text(report)),
        crate::cli::AssessmentResultsReportFormat::Html => Ok(render_html(report)),
    }
}

fn render_text(report: &AssessmentResultsReport) -> String {
    let mut output = String::new();
    output.push_str("FORGE OSCAL Assessment Results review report\n");
    let _ = writeln!(output, "schema: {}", report.schema_version);
    let _ = writeln!(
        output,
        "status: {}",
        match report.status {
            ReportStatus::Complete => "complete",
            ReportStatus::ReviewRequired => "review-required",
        }
    );
    let _ = writeln!(output, "artifact UUID: {}", escape(&report.artifact_uuid));
    let _ = writeln!(
        output,
        "objects: observations={} findings={} risks={} relationships={} evidence-references={}",
        report.counts.observations,
        report.counts.findings,
        report.counts.risks,
        report.counts.relationships,
        report.counts.evidence_references
    );
    for context in &report.context {
        let _ = writeln!(
            output,
            "context: kind={} sha256={} root-uuid={} document-version={} oscal-version={}",
            context.kind,
            context.sha256,
            context.root_uuid,
            escape(&context.document_version),
            escape(&context.oscal_version)
        );
    }
    let _ = writeln!(output, "review findings: {}", report.findings.len());
    for finding in &report.findings {
        let kind = finding.object_type.map_or("context", ConclusionType::as_str);
        let _ = writeln!(
            output,
            "- {} {} {}: {}",
            escape(&finding.id),
            escape(&finding.code),
            kind,
            escape(&finding.key)
        );
    }
    let _ = writeln!(output, "trust boundary: {}", report.trust_boundary);
    output
}

fn render_html(report: &AssessmentResultsReport) -> String {
    let status = match report.status {
        ReportStatus::Complete => "complete",
        ReportStatus::ReviewRequired => "review-required",
    };
    let mut output = String::from(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>FORGE Assessment Results review</title></head><body>",
    );
    output.push_str("<main><h1>FORGE OSCAL Assessment Results review</h1>");
    let _ = write!(
        output,
        "<dl><dt>Schema</dt><dd>{}</dd><dt>Status</dt><dd>{}</dd><dt>Artifact UUID</dt><dd>{}</dd></dl>",
        html_escape(report.schema_version),
        status,
        html_escape(&report.artifact_uuid)
    );
    let _ = write!(
        output,
        "<p>Observations: {}. Findings: {}. Risks: {}. Relationships: {}. Evidence references: {}.</p>",
        report.counts.observations,
        report.counts.findings,
        report.counts.risks,
        report.counts.relationships,
        report.counts.evidence_references
    );
    output.push_str("<h2>Context identities</h2><table><thead><tr><th>Kind</th><th>SHA-256</th><th>Root UUID</th><th>Document version</th><th>OSCAL version</th></tr></thead><tbody>");
    for context in &report.context {
        let _ = write!(
            output,
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            html_escape(context.kind),
            html_escape(&context.sha256),
            html_escape(&context.root_uuid),
            html_escape(&context.document_version),
            html_escape(&context.oscal_version)
        );
    }
    output.push_str("</tbody></table><h2>Review findings</h2><ul>");
    for finding in &report.findings {
        let kind = finding.object_type.map_or("context", ConclusionType::as_str);
        let _ = write!(
            output,
            "<li><code>{}</code> <strong>{}</strong> {} <code>{}</code></li>",
            html_escape(&finding.id),
            html_escape(&finding.code),
            html_escape(kind),
            html_escape(&finding.key)
        );
    }
    let _ = writeln!(
        output,
        "</ul><p><strong>Trust boundary:</strong> {}</p></main></body></html>",
        html_escape(report.trust_boundary)
    );
    output
}

fn escape(value: &str) -> String {
    value.chars().flat_map(char::escape_default).collect()
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_escaping_neutralizes_terminal_controls_and_html_markup() {
        assert_eq!(escape("line\n\u{1b}[31m"), "line\\n\\u{1b}[31m");
        let escaped = html_escape("<script>alert('x') & \"y\"</script>");
        assert_eq!(escaped, "&lt;script&gt;alert(&#39;x&#39;) &amp; &quot;y&quot;&lt;/script&gt;");
        assert!(!escaped.contains('<'));
    }
}
