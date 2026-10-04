//! S2 static HTML for the complete internally prepared S1 portfolio.
//!
//! No author/native text enters markup, executable code, URLs or styles. Preserved
//! native references are escaped display text in their original bundle-relative
//! spelling, never active links and never rebased to the HTML location. The renderer
//! performs no input reads, source edits, network requests or output writes. A fresh
//! PreparedPortfolio::verify_inputs check remains the publisher's responsibility.

use super::portfolio::{self, PlanRow, PortfolioReport, RecordRow, TimelineRow, TraceRow};
use crate::ForgeError;

/// Render every plan/record/event/selection through a pre-growth ten-MiB writer.
///
/// The report is internally constructed by full current workflow/source admission,
/// not deserialized from an arbitrary report or native extension. Identity metadata
/// remains potentially sensitive: titles, assessment prose, party names, private
/// paths and plaintext rationale/evidence locations are deliberately omitted.
/// # Errors
/// Refuses inconsistent complete denominators, undisposed terminal assertions or
/// the complete output bound; no incomplete prefix is returned.
pub fn render(report: &PortfolioReport) -> Result<Vec<u8>, ForgeError> {
    render_bounded(report, portfolio::MAX_OUTPUT_BYTES)
}

/// Render all rows or fail the entire result; a smaller ceiling is used only in tests.
fn render_bounded(report: &PortfolioReport, limit: usize) -> Result<Vec<u8>, ForgeError> {
    portfolio::validate_report(report)?;
    if limit > portfolio::MAX_OUTPUT_BYTES {
        return Err(error("HTML limit exceeds the complete report bound"));
    }
    let mut out = Html { bytes: Vec::new(), limit };
    out.markup("<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'; img-src 'none'\"><title>Forge POA&amp;M portfolio</title><style>body{font:1rem/1.5 system-ui,sans-serif;max-width:100rem;margin:2rem auto;padding:0 1rem;color:#17202a;background:#fff}table{border-collapse:collapse;width:100%;margin:1rem 0}caption{text-align:left;font-weight:bold}th,td{border:1px solid #80909e;padding:.5rem;text-align:left;vertical-align:top}th{background:#edf2f5}code,.identity{white-space:pre-wrap;overflow-wrap:anywhere;unicode-bidi:isolate}dt{font-weight:bold}dd{margin-bottom:.5rem}section{margin-top:2rem}.scroll{overflow-x:auto}.note{border-left:.3rem solid #536878;padding-left:1rem}</style></head><body><h1>Forge POA&amp;M portfolio</h1><p class=\"note\">")?;
    out.text(report.boundary)?;
    out.markup("</p><p>Input validation scope: <code>")?;
    out.text(report.validation_scope)?;
    out.markup("</code>. Authoring-only or mixed inputs are previews; an artifact portfolio requires a supplied native POA&amp;M for every plan.</p><p>Explicit as-of: <code>")?;
    out.text(&report.as_of)?;
    out.markup("</code>. Inclusive due-soon interval: ")?;
    out.number(usize::from(report.due_soon_days))?;
    out.markup(" days. Events are author assertions; events after the as-of UTC date remain in the timeline. Source hrefs below are display text relative to each original bundle, regardless of this HTML file's location.</p><p>Keys, UUIDs and rationale digests can still reveal sensitive identities or permit correlation. No party authority, evidence freshness, remediation effectiveness or source eligibility was established.</p>")?;
    render_counts(&mut out, report)?;
    for plan in &report.plans {
        render_plan(&mut out, plan)?;
    }
    out.markup("</body></html>\n")?;
    Ok(out.bytes)
}

/// Emit all summed denominators without treating shared source objects as a unique union.
fn render_counts(out: &mut Html, report: &PortfolioReport) -> Result<(), ForgeError> {
    let counts = &report.counts;
    out.markup("<div class=\"scroll\"><table><caption>Complete summed per-plan occurrences</caption><thead><tr><th scope=\"col\">Denominator or classification</th><th scope=\"col\">Count</th></tr></thead><tbody>")?;
    for (label, value) in [
        ("Explicit plans", counts.plans),
        ("Actual supplied native artifact inputs", counts.supplied_native_artifacts),
        ("Authoring-only preview plans", counts.authoring_only_plans),
        ("Five-source file occurrences", counts.source_files),
        ("Source raw-byte occurrences", counts.source_bytes),
        ("All selected-result source object occurrences", counts.source_objects),
        ("Selected items (all states)", counts.items),
        ("Milestones (all states)", counts.milestones),
        ("Complete item plus milestone rows", counts.records),
        ("All declared history events", counts.history_events),
        ("Exact source selection occurrences", counts.source_selections),
        ("Owner declarations", counts.owner_declarations),
        ("Milestone dependency edges", counts.dependency_edges),
        ("Overdue open rows", counts.overdue),
        ("Due-soon open rows", counts.due_soon),
        ("Blocked rows at as-of", counts.blocked),
        ("Rows before their first assertion", counts.before_first_assertion),
        ("Cancelled rows at as-of", counts.cancelled),
    ] {
        out.markup("<tr><th scope=\"row\">")?;
        out.text(label)?;
        out.markup("</th><td>")?;
        out.number(value)?;
        out.markup("</td></tr>")?;
    }
    out.markup("</tbody></table></div>")
}

/// Display one plan's complete receipts, schedule, timeline and source-to-work trace.
fn render_plan(out: &mut Html, plan: &PlanRow) -> Result<(), ForgeError> {
    out.markup("<section><h2>Plan <code>")?;
    out.text(&plan.key)?;
    out.markup("</code></h2><dl>")?;
    for (label, value) in [
        ("Plan UUID", &plan.uuid),
        ("Original authoring file SHA256", &plan.authoring_sha256),
        ("Generated native bytes SHA256", &plan.generated_native_sha256),
        ("Source result UUID", &plan.result_uuid),
        ("Source result key", &plan.result_key),
    ] {
        out.markup("<dt>")?;
        out.text(label)?;
        out.markup("</dt><dd><code>")?;
        out.text(value)?;
        out.markup("</code></dd>")?;
    }
    out.markup("<dt>Supplied native original SHA256</dt><dd><code>")?;
    match &plan.supplied_native_sha256 {
        Some(value) => out.text(value)?,
        None => out.text("none supplied")?,
    }
    out.markup("</code></dd></dl><p>Generated supported native projection passed its pinned offline schema during preparation. ")?;
    if plan.supplied_native_matched {
        out.text("An explicitly supplied native file matched the entire generated decoded value, including the exact namespaced declaration strings and existing source href/hash values. Outer JSON formatting may differ.")?;
    } else {
        out.text("Only the explicit authoring companion was supplied; no existing native artifact was imported or claimed to match.")?;
    }
    out.markup(
        "</p><p>Complete selected-result source object occurrences before work selection: ",
    )?;
    out.number(plan.source_objects)?;
    out.markup(".</p><div class=\"scroll\"><table><caption>Five actual source file occurrences</caption><thead><tr><th scope=\"col\">Kind</th><th scope=\"col\">Preserved native href (display text)</th><th scope=\"col\">Whole-file SHA256</th><th scope=\"col\">Bytes</th></tr></thead><tbody>")?;
    for source in &plan.sources {
        out.markup("<tr>")?;
        out.cell(source.kind)?;
        out.cell(&source.native_href)?;
        out.cell(&source.sha256)?;
        out.markup("<td>")?;
        out.number(source.bytes)?;
        out.markup("</td></tr>")?;
    }
    out.markup("</tbody></table></div><div class=\"scroll\"><table><caption>Complete item and milestone schedule</caption><thead><tr><th scope=\"col\">Work identity</th><th scope=\"col\">UUID</th><th scope=\"col\">Target</th><th scope=\"col\">Assertion at as-of</th><th scope=\"col\">Current declaration</th><th scope=\"col\">Accountability declarations</th><th scope=\"col\">Dependencies</th><th scope=\"col\">Schedule flags</th></tr></thead><tbody>")?;
    for row in &plan.records {
        render_record(out, row)?;
    }
    out.markup("</tbody></table></div><div class=\"scroll\"><table><caption>Complete author assertion timeline, ordered by actual instant</caption><thead><tr><th scope=\"col\">Work identity</th><th scope=\"col\">Event</th><th scope=\"col\">Authored timestamp</th><th scope=\"col\">Transition</th><th scope=\"col\">Explicit actor</th><th scope=\"col\">Declared rationale SHA256</th><th scope=\"col\">As-of scope</th></tr></thead><tbody>")?;
    for row in &plan.timeline {
        render_event(out, row)?;
    }
    out.markup("</tbody></table></div><div class=\"scroll\"><table><caption>Complete selected source-to-remediation trace</caption><thead><tr><th scope=\"col\">Work key and UUID</th><th scope=\"col\">Source kind and key</th><th scope=\"col\">Original source UUID</th><th scope=\"col\">Result UUID</th><th scope=\"col\">Canonical object SHA256</th><th scope=\"col\">Whole AR file SHA256</th><th scope=\"col\">Preserved native href (display text)</th></tr></thead><tbody>")?;
    for row in &plan.trace {
        render_trace(out, row)?;
    }
    out.markup("</tbody></table></div></section>")
}

/// Display exact item/milestone labels as text without inventing a join delimiter identity.
fn work_identity(out: &mut Html, item: &str, milestone: Option<&str>) -> Result<(), ForgeError> {
    out.markup("<code>")?;
    out.text(item)?;
    out.markup("</code>")?;
    if let Some(value) = milestone {
        out.markup("<br>milestone <code>")?;
        out.text(value)?;
        out.markup("</code>")?;
    }
    Ok(())
}

/// Display one complete record including null, cancelled, future and blocked scope.
fn render_record(out: &mut Html, row: &RecordRow) -> Result<(), ForgeError> {
    out.markup("<tr><td>")?;
    work_identity(out, &row.item_key, row.milestone_key.as_deref())?;
    out.markup("</td>")?;
    out.cell(&row.uuid)?;
    out.cell(&row.target_date)?;
    out.cell(row.state_at_as_of.map_or("before first assertion", |state| state.as_str()))?;
    out.cell(row.declared_state.as_str())?;
    out.markup("<td>")?;
    for owner in &row.owners {
        out.markup("<div>role <code>")?;
        out.text(&owner.role_id)?;
        out.markup("</code>; party key <code>")?;
        out.text(&owner.party_key)?;
        out.markup("</code>; declared rationale digest <code>")?;
        out.text(&owner.declared_rationale_sha256)?;
        out.markup("</code></div>")?;
    }
    out.markup("</td><td>")?;
    for dependency in &row.depends_on {
        out.markup("<div><code>")?;
        out.text(dependency)?;
        out.markup("</code></div>")?;
    }
    out.markup("</td><td>")?;
    if row.overdue {
        out.text("overdue ")?;
    }
    if row.due_soon {
        out.text("due-soon ")?;
    }
    if row.state_at_as_of == Some(super::workflow::State::Blocked) {
        out.text("blocked")?;
    }
    out.markup("</td></tr>")
}

/// Display every explicit event; future declarations are labelled, never silently omitted.
fn render_event(out: &mut Html, row: &TimelineRow) -> Result<(), ForgeError> {
    out.markup("<tr><td>")?;
    work_identity(out, &row.item_key, row.milestone_key.as_deref())?;
    out.markup("</td>")?;
    out.cell(&row.event_key)?;
    out.cell(&row.at)?;
    out.markup("<td><code>")?;
    out.text(row.from.map_or("initial", |state| state.as_str()))?;
    out.markup("</code> → <code>")?;
    out.text(row.to.as_str())?;
    out.markup("</code></td><td>role <code>")?;
    out.text(&row.actor_role_id)?;
    out.markup("</code>; party key <code>")?;
    out.text(&row.actor_party_key)?;
    out.markup("</code></td>")?;
    out.cell(&row.declared_rationale_sha256)?;
    out.cell(if row.after_as_of { "after as-of UTC date" } else { "on/before as-of UTC date" })?;
    out.markup("</tr>")
}

/// Keep canonical object hashes distinct from exact file hashes and preserve original UUID/href strings.
fn render_trace(out: &mut Html, row: &TraceRow) -> Result<(), ForgeError> {
    out.markup("<tr><td><code>")?;
    out.text(&row.item_key)?;
    out.markup("</code><br><code>")?;
    out.text(&row.item_uuid)?;
    out.markup("</code></td><td>")?;
    out.text(row.kind.as_str())?;
    out.markup(" <code>")?;
    out.text(&row.source_key)?;
    out.markup("</code></td>")?;
    out.cell(&row.source_uuid)?;
    out.cell(&row.result_uuid)?;
    out.cell(&row.canonical_object_sha256)?;
    out.cell(&row.assessment_results_file_sha256)?;
    out.cell(&row.native_href)?;
    out.markup("</tr>")
}

/// Writer whose raw-markup interface only accepts trusted static strings.
struct Html {
    /// Complete retained UTF8 prefix, returned only on full success.
    bytes: Vec<u8>,
    /// Inclusive final byte ceiling.
    limit: usize,
}

impl Html {
    /// Append a complete fragment only when it fits before vector growth.
    fn append(&mut self, bytes: &[u8]) -> Result<(), ForgeError> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(error("complete portfolio HTML exceeds its output bound"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }

    /// Only compile-time trusted static markup can reach this unescaped interface.
    fn markup(&mut self, text: &'static str) -> Result<(), ForgeError> {
        self.append(text.as_bytes())
    }

    /// Escape every dynamic scalar; render controls/directional formatting visibly without normalization.
    fn text(&mut self, text: &str) -> Result<(), ForgeError> {
        for value in text.chars() {
            match value {
                '&' => self.markup("&amp;")?,
                '<' => self.markup("&lt;")?,
                '>' => self.markup("&gt;")?,
                '"' => self.markup("&quot;")?,
                '\'' => self.markup("&#39;")?,
                value
                    if value.is_control()
                        || matches!(value, '\u{061c}' | '\u{200e}' | '\u{200f}'
                    | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}') =>
                {
                    self.append(format!("\\u{{{:X}}}", u32::from(value)).as_bytes())?;
                }
                value => {
                    let mut encoded = [0; 4];
                    self.append(value.encode_utf8(&mut encoded).as_bytes())?;
                }
            }
        }
        Ok(())
    }

    /// Render a bounded integer through the same escaped-text interface.
    fn number(&mut self, value: usize) -> Result<(), ForgeError> {
        self.text(&value.to_string())
    }

    /// Place one escaped dynamic scalar inside a fixed table-cell structure.
    fn cell(&mut self, value: &str) -> Result<(), ForgeError> {
        self.markup("<td class=\"identity\"><code>")?;
        self.text(value)?;
        self.markup("</code></td>")
    }
}

/// Emit a fixed HTML failure without authored prose or input paths.
fn error(message: &'static str) -> ForgeError {
    ForgeError::PoamBuild(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Dynamic malicious text stays inert in every displayed role and source field.
    #[test]
    fn malicious_text_is_escaped_and_source_hrefs_are_never_activated() {
        let mut report = portfolio::test_report();
        let payload =
            "</script><img src=\"https://example.invalid/x\" onerror='run()'>&é\u{1b}\u{202e}";
        report.plans[0].key = payload.to_string();
        report.plans[0].records[0].item_key = payload.to_string();
        report.plans[0].records[0].owners[0].role_id = payload.to_string();
        report.plans[0].records[0].depends_on.push(payload.to_string());
        report.counts.dependency_edges += 1;
        report.plans[0].timeline[0].actor_party_key = payload.to_string();
        report.plans[0].trace[0].source_key = payload.to_string();
        report.plans[0].trace[0].native_href = "javascript:run()#\"<&'".to_string();
        report.plans[0].sources[0].native_href = payload.to_string();
        let original = serde_json::to_vec(&report).unwrap();
        let output = String::from_utf8(render(&report).unwrap()).unwrap();
        assert!(output.contains("&lt;/script&gt;&lt;img src=&quot;https://example.invalid/x&quot; onerror=&#39;run()&#39;&gt;&amp;é\\u{1B}\\u{202E}"));
        assert!(output.contains("javascript:run()#&quot;&lt;&amp;&#39;"));
        for active in ["<script", "<img", "href=", " onerror=\"", "<iframe", "<form", "url("] {
            assert!(!output.contains(active));
        }
        assert_eq!(serde_json::to_vec(&report).unwrap(), original);
    }

    /// Every fixture record and event appears; null/future/cancelled scope is retained.
    #[test]
    fn complete_rows_and_existing_uri_spelling_survive_static_rendering() {
        let report = portfolio::test_report();
        let output = String::from_utf8(render(&report).unwrap()).unwrap();
        assert!(output.contains("authoring-preview-with-optional-native-pairs"));
        assert!(output.contains("Actual supplied native artifact inputs"));
        assert!(output.contains("before first assertion"));
        assert!(output.contains("cancelled"));
        assert!(output.contains("after as-of UTC date"));
        assert!(output.contains("nested/AR%20file%23name.json#original-source-uuid"));
        for plan in &report.plans {
            for row in &plan.records {
                assert!(output.contains(&row.uuid));
            }
            for event in &plan.timeline {
                assert!(output.contains(&event.event_key));
            }
        }
        assert!(!output.contains("SENSITIVE PLAINTEXT RATIONALE"));
        assert!(output.ends_with("</body></html>\n"));
    }

    /// A single missing count, undisposed terminal record or byte ceiling refuses all output.
    #[test]
    fn inconsistent_denominator_terminal_bypass_and_output_overflow_refuse() {
        let mut report = portfolio::test_report();
        report.counts.records -= 1;
        assert!(render(&report).is_err());
        report = portfolio::test_report();
        report.plans[0].records[0].declared_state =
            super::super::workflow::State::CompletedAsserted;
        assert!(render(&report).is_err());
        report = portfolio::test_report();
        assert!(render_bounded(&report, 128).is_err());
        let complete = render(&report).unwrap();
        assert_eq!(render_bounded(&report, complete.len()).unwrap(), complete);
        assert!(render_bounded(&report, complete.len() - 1).is_err());
    }
}
