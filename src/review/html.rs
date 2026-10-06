//! Static recorded-review HTML, with no scripts, source prose or fresh authority.
//! Every output byte consumes the caller's existing logical storage ledger before
//! growth; spare Vec capacity is not another retained payload. This is logical
//! payload accounting, not allocator/total-heap confinement or
//! preemption; the final caller still owns file publication and any fresh proof.

use super::decode::{ContractError, ContractLedger, Decoded};
use super::wire::{
    Disposition, DispositionsDocument, ItemDisposition, ItemState, RecordedCurrentness,
    RecordedResponse, ResponseClassification, SourceModel, SourcePin,
};
use crate::workspace::preparation::WorkControl;

/// Complete encoded HTML ceiling; the same ledger also retains prior command work.
const OUTPUT_LIMIT: usize = 33_554_432;

/// Constant inert document shell, local navigation and responsive/print styling.
const OPEN: &str = r##"<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Recorded review dispositions</title>
<style>
:root{color-scheme:light}*{box-sizing:border-box}body{margin:auto;max-width:90rem;padding:1rem;font:1rem/1.5 system-ui,sans-serif;color:#18202a;background:#fff}a{color:#003b80;text-underline-offset:.15em}a:focus-visible{outline:3px solid #003b80;outline-offset:3px}h1,h2{line-height:1.2}nav ul{display:flex;flex-wrap:wrap;gap:.5rem 1.5rem;padding-inline-start:1.5rem}section{margin-block:2rem}table{width:100%;border-collapse:collapse;table-layout:fixed;margin-block:1rem}caption{text-align:left;font-weight:bold;padding-block:.5rem}th,td{border:1px solid #66717f;padding:.5rem;text-align:left;vertical-align:top;overflow-wrap:anywhere}th{background:#eef2f6}ul{padding-inline-start:1.25rem;margin-block:.25rem}dt{font-weight:bold}dd{margin-inline-start:0;margin-block-end:.5rem;overflow-wrap:anywhere}.notice{border-inline-start:.3rem solid #003b80;padding:.75rem;background:#eef2f6}.skip{display:inline-block;padding:.25rem}main{min-width:0}small{font-size:.9em}@media(max-width:40rem){body{padding:.5rem}th,td{padding:.25rem;font-size:.9rem}nav ul{display:block}}@media print{body{max-width:none;padding:0;font-size:10pt}nav,.skip{display:none}thead{display:table-header-group}tr{break-inside:avoid}h2{break-after:avoid}a{color:inherit}.notice{background:transparent}}
</style></head><body>
<a class="skip" href="#summary">Skip to recorded summary</a>
<header><h1>Recorded review dispositions</h1>
<p class="notice">Recorded status only. Fresh source capture and complete currentness checks are required to assert current status. Review quorum is a declared review-policy result, not domain approval or a native state transition.</p>
<p>Keys, roles, authors and timestamps are asserted, not authenticated, signed or non-repudiable. Private response rationale and source excerpt fields are not exported. Declared identifiers, schema labels and hashes may contain sensitive information.</p>
<nav aria-label="Recorded review sections"><ul><li><a href="#summary">Summary</a></li><li><a href="#sources">Source pins</a></li><li><a href="#items">Items and dissent</a></li><li><a href="#responses">All unique responses</a></li></ul></nav></header><main>
"##;

/// One output owner borrowing the original ledger and cooperative control.
struct HtmlWriter<'a> {
    /// Measured encoded payload; no parallel rendered copy is retained.
    output: Vec<u8>,
    /// Same monotonic command owner used by decoding/capture/matching callers.
    ledger: &'a mut ContractLedger,
    /// Original caller stop/deadline source; no timer or no-op substitution.
    control: &'a mut dyn WorkControl,
}

impl<'a> HtmlWriter<'a> {
    /// Start an empty output without allocating or resetting the command ledger.
    fn new(ledger: &'a mut ContractLedger, control: &'a mut dyn WorkControl) -> Self {
        Self { output: Vec::new(), ledger, control }
    }

    /// Precharge work/storage and encoded length before growth; geometric reserve
    /// also charges the complete prior payload before a possible allocation copy.
    fn raw(&mut self, bytes: &[u8]) -> Result<(), ContractError> {
        self.ledger.checkpoint(self.control)?;
        self.ledger.visits(1)?;
        self.ledger.bytes(bytes.len())?;
        let next =
            self.output.len().checked_add(bytes.len()).ok_or_else(|| self.ledger.capacity())?;
        if next > OUTPUT_LIMIT {
            return Err(self.ledger.capacity());
        }
        self.ledger.derived(bytes.len())?;
        if next > self.output.capacity() {
            self.ledger.bytes(self.output.len())?;
            let capacity =
                self.output.capacity().saturating_mul(2).clamp(4_096, OUTPUT_LIMIT).max(next);
            self.output
                .try_reserve_exact(capacity - self.output.len())
                .map_err(|_| self.ledger.capacity())?;
        }
        self.output.extend_from_slice(bytes);
        Ok(())
    }

    /// Escape every dynamic text fragment, preserving UTF-8 and never creating markup.
    /// The same encodings are safe in quoted attributes, although none are dynamic.
    fn text(&mut self, text: &str) -> Result<(), ContractError> {
        self.ledger.checkpoint(self.control)?;
        self.ledger.bytes(text.len())?;
        let mut start = 0;
        for (index, byte) in text.bytes().enumerate() {
            let escaped: &[u8] = match byte {
                b'&' => b"&amp;",
                b'<' => b"&lt;",
                b'>' => b"&gt;",
                b'\"' => b"&quot;",
                b'\'' => b"&#39;",
                _ => continue,
            };
            self.raw(&text.as_bytes()[start..index])?;
            self.raw(escaped)?;
            start = index + 1;
        }
        self.raw(&text.as_bytes()[start..])
    }

    /// Encode an unsigned count using a fixed twenty-byte stack buffer, not a String.
    fn number(&mut self, mut value: u64) -> Result<(), ContractError> {
        self.ledger.checkpoint(self.control)?;
        self.ledger.bytes(20)?;
        let mut digits = [0_u8; 20];
        let mut start = digits.len();
        loop {
            start -= 1;
            digits[start] = b'0' + u8::try_from(value % 10).map_err(|_| ContractError::Invalid)?;
            value /= 10;
            if value == 0 {
                break;
            }
        }
        self.raw(&digits[start..])
    }

    /// Convert a complete collection extent without narrowing its denominator.
    fn count(&mut self, value: usize) -> Result<(), ContractError> {
        let value = u64::try_from(value).map_err(|_| self.ledger.capacity())?;
        self.number(value)
    }

    /// Emit one dynamic text cell; tag/attribute spelling is always constant.
    fn cell(&mut self, value: &str) -> Result<(), ContractError> {
        self.raw(b"<td>")?;
        self.text(value)?;
        self.raw(b"</td>")
    }

    /// Emit one numeric cell without formatting allocation.
    fn numeric_cell(&mut self, value: u64) -> Result<(), ContractError> {
        self.raw(b"<td>")?;
        self.number(value)?;
        self.raw(b"</td>")
    }

    /// Preserve an explicit null claim as a visible absence, without guessing a value.
    fn optional_cell(&mut self, value: Option<&str>) -> Result<(), ContractError> {
        self.cell(value.unwrap_or("Not declared"))
    }

    /// Emit every ordered token/reference as text; empty lists remain explicitly empty.
    fn list(&mut self, values: &[String]) -> Result<(), ContractError> {
        if values.is_empty() {
            return self.raw(b"<p>None recorded</p>");
        }
        self.raw(b"<ul>")?;
        for value in values {
            self.raw(b"<li>")?;
            self.text(value)?;
            self.raw(b"</li>")?;
        }
        self.raw(b"</ul>")
    }

    /// Check the same sticky control after all bytes, before releasing the complete output.
    fn finish(self) -> Result<Vec<u8>, ContractError> {
        self.ledger.checkpoint(self.control)?;
        Ok(self.output)
    }
}

/// Render complete redacted recorded declarations, without currentness or approval proof.
/// All dynamic values remain text nodes; callers own any later fresh fence/file publication.
pub(crate) fn render(
    decoded: &Decoded<'_, DispositionsDocument>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<u8>, ContractError> {
    ledger.bound(|ledger| {
        let mut writer = HtmlWriter::new(ledger, control);
        writer.raw(OPEN.as_bytes())?;
        summary(&mut writer, decoded)?;
        sources(&mut writer, &decoded.document().source_pins)?;
        items(&mut writer, &decoded.document().items)?;
        responses(&mut writer, &decoded.document().responses)?;
        writer.raw(b"</main><footer><p>Complete recorded evidence; no live status or domain approval is asserted by this export.</p></footer></body></html>\n")?;
        writer.finish()
    })
}

/// Show original package identity and complete recorded denominators with no live inference.
fn summary(
    writer: &mut HtmlWriter<'_>,
    decoded: &Decoded<'_, DispositionsDocument>,
) -> Result<(), ContractError> {
    let doc = decoded.document();
    writer.raw(
        b"<section id=\"summary\" tabindex=\"-1\"><h2>Recorded summary</h2><dl><dt>Schema</dt><dd>",
    )?;
    writer.text(&doc.schema_version)?;
    writer.raw(b"</dd><dt>Queue ID</dt><dd>")?;
    writer.text(&doc.queue_id)?;
    writer.raw(b"</dd><dt>Original queue SHA-256</dt><dd>")?;
    writer.text(&doc.queue_raw_sha256)?;
    writer.raw(b"</dd><dt>Recorded as_of (asserted UTC)</dt><dd>")?;
    writer.text(&doc.as_of)?;
    writer.raw(b"</dd><dt>Recorded currentness claim</dt><dd>")?;
    writer.text(match doc.currentness {
        RecordedCurrentness::Unverified => "unverified",
        RecordedCurrentness::RecordedCurrent => {
            "recorded-current (not freshly verified by this export)"
        }
    })?;
    writer.raw(b"</dd><dt>Recorded closure generation</dt><dd>")?;
    writer.text(doc.closure_generation.as_deref().unwrap_or("Not declared"))?;
    writer.raw(b"</dd><dt>Original dispositions SHA-256</dt><dd>")?;
    writer.text(decoded.raw_sha256())?;
    writer.raw(b"</dd><dt>Original dispositions bytes</dt><dd>")?;
    writer.count(decoded.raw().len())?;
    writer.raw(b"</dd><dt>Mandatory identity disclaimer</dt><dd>")?;
    writer.text(&doc.identity_disclaimer)?;
    writer.raw(b"</dd></dl><table><caption>Complete recorded evidence denominators</caption><thead><tr><th scope=\"col\">Denominator</th><th scope=\"col\">Count</th></tr></thead><tbody>")?;
    for (label, count) in [
        ("Items", u64::from(doc.counts.items)),
        ("Original response files", u64::from(doc.counts.response_files)),
        ("Unique responses", u64::from(doc.counts.unique_responses)),
        ("Additional exact duplicate files", u64::from(doc.counts.exact_duplicates)),
    ] {
        count_row(writer, label, count)?;
    }
    writer.raw(b"<tr><th scope=\"row\">Source pins</th><td>")?;
    writer.count(doc.source_pins.len())?;
    writer.raw(b"</td></tr></tbody></table>")?;
    state_counts(writer, doc)?;
    writer.raw(b"</section>\n")
}

/// Emit one constant-labelled complete denominator row.
fn count_row(writer: &mut HtmlWriter<'_>, label: &str, count: u64) -> Result<(), ContractError> {
    writer.raw(b"<tr><th scope=\"row\">")?;
    writer.text(label)?;
    writer.raw(b"</th>")?;
    writer.numeric_cell(count)?;
    writer.raw(b"</tr>")
}

/// Preserve all eight recorded state counters, including explicit zeros.
fn state_counts(
    writer: &mut HtmlWriter<'_>,
    doc: &DispositionsDocument,
) -> Result<(), ContractError> {
    let states = &doc.counts.states;
    writer.raw(b"<table><caption>Complete recorded item-state counts; quorum-met is review policy only</caption><thead><tr><th scope=\"col\">Recorded state</th><th scope=\"col\">Items</th></tr></thead><tbody>")?;
    for (label, count) in [
        ("unassigned", states.unassigned),
        ("assigned", states.assigned),
        ("in-review", states.in_review),
        ("conflicted", states.conflicted),
        ("changes-requested", states.changes_requested),
        ("quorum-met", states.quorum_met),
        ("expired", states.expired),
        ("stale", states.stale),
    ] {
        count_row(writer, label, u64::from(count))?;
    }
    writer.raw(b"</tbody></table>")
}

/// Emit every source pin in declared order, without reading any target/source path.
fn sources(writer: &mut HtmlWriter<'_>, pins: &[SourcePin]) -> Result<(), ContractError> {
    writer.raw(b"<section id=\"sources\"><h2>All recorded source pins</h2><table><caption>Complete declared original source identities; not freshly captured</caption><thead><tr><th scope=\"col\">Artifact key</th><th scope=\"col\">Model</th><th scope=\"col\">Native root UUID</th><th scope=\"col\">Original SHA-256</th><th scope=\"col\">Bytes</th><th scope=\"col\">Schema identity</th></tr></thead><tbody>")?;
    for pin in pins {
        writer.raw(b"<tr data-row=\"source\"><th scope=\"row\">")?;
        writer.text(&pin.artifact_key)?;
        writer.raw(b"</th>")?;
        writer.cell(source_model(pin.model))?;
        writer.optional_cell(pin.native_root_uuid.as_deref())?;
        writer.cell(&pin.raw_sha256)?;
        writer.numeric_cell(pin.byte_length)?;
        writer.cell(&pin.schema_identity)?;
        writer.raw(b"</tr>")?;
    }
    writer.raw(b"</tbody></table></section>\n")
}

/// Emit all items, complete witnesses/references and dissent without selecting winners.
fn items(writer: &mut HtmlWriter<'_>, rows: &[ItemDisposition]) -> Result<(), ContractError> {
    writer.raw(b"<section id=\"items\"><h2>All recorded items and dissent</h2><table><caption>Complete item statuses and seat denominators; recorded claims only</caption><thead><tr><th scope=\"col\">Item key and ID</th><th scope=\"col\">Recorded state and blockers</th><th scope=\"col\">Reason codes</th><th scope=\"col\">Required seats and recorded witnesses</th><th scope=\"col\">All response IDs</th><th scope=\"col\">All dissent IDs</th></tr></thead><tbody>")?;
    for item in rows {
        writer.raw(b"<tr data-row=\"item\"><th scope=\"row\">")?;
        writer.text(&item.item_key)?;
        writer.raw(b"<br>")?;
        writer.text(&item.item_id)?;
        writer.raw(b"</th><td>")?;
        writer.text(item_state(item.state))?;
        writer.raw(b"<p>Recorded blocking: ")?;
        writer.text(if item.blocking { "true" } else { "false" })?;
        writer.raw(b"</p></td><td>")?;
        writer.list(&item.reason_codes)?;
        writer.raw(b"</td><td>Required: ")?;
        writer.number(u64::from(item.required_seats))?;
        seats(writer, item)?;
        writer.raw(b"</td><td>")?;
        writer.list(&item.response_ids)?;
        writer.raw(b"</td><td>")?;
        writer.list(&item.dissent_ids)?;
        writer.raw(b"</td></tr>")?;
    }
    writer.raw(b"</tbody></table></section>\n")
}

/// Show all met and unmet witnesses, including role, ordinal, asserted key and response ID.
fn seats(writer: &mut HtmlWriter<'_>, item: &ItemDisposition) -> Result<(), ContractError> {
    writer.raw(b"<p>Recorded met: ")?;
    writer.count(item.met_seats.len())?;
    writer.raw(b"</p><ul>")?;
    for seat in &item.met_seats {
        writer.raw(b"<li>Role ")?;
        writer.text(&seat.role_key)?;
        writer.raw(b"; ordinal ")?;
        writer.number(u64::from(seat.ordinal))?;
        writer.raw(b"; asserted reviewer key ")?;
        writer.text(&seat.reviewer_key)?;
        writer.raw(b"; response ID ")?;
        writer.text(&seat.response_id)?;
        writer.raw(b"</li>")?;
    }
    writer.raw(b"</ul><p>Recorded unmet: ")?;
    writer.count(item.unmet_seats.len())?;
    writer.raw(b"</p><ul>")?;
    for seat in &item.unmet_seats {
        writer.raw(b"<li>Role ")?;
        writer.text(&seat.role_key)?;
        writer.raw(b"; ordinal ")?;
        writer.number(u64::from(seat.ordinal))?;
        writer.raw(b"</li>")?;
    }
    writer.raw(b"</ul>")
}

/// Preserve every unique recorded original, including foreign/stale/historical dissent.
fn responses(writer: &mut HtmlWriter<'_>, rows: &[RecordedResponse]) -> Result<(), ContractError> {
    writer.raw(b"<section id=\"responses\"><h2>All unique recorded responses</h2><p>Exact duplicate files are counted separately above; no unrecorded originals or live classifications are inferred.</p><table><caption>Complete distinct response evidence, including all dissent dispositions</caption><thead><tr><th scope=\"col\">Response ID</th><th scope=\"col\">Original SHA-256 and bytes</th><th scope=\"col\">Asserted item key</th><th scope=\"col\">Asserted reviewer key and role</th><th scope=\"col\">Disposition</th><th scope=\"col\">Asserted response time (UTC)</th><th scope=\"col\">Recorded classification</th></tr></thead><tbody>")?;
    for row in rows {
        writer.raw(b"<tr data-row=\"response\"><th scope=\"row\">")?;
        writer.text(&row.response_id)?;
        writer.raw(b"</th><td>")?;
        writer.text(&row.raw_sha256)?;
        writer.raw(b"<br>Bytes: ")?;
        writer.number(row.byte_length)?;
        writer.raw(b"</td>")?;
        writer.cell(&row.item_key)?;
        writer.raw(b"<td>Key: ")?;
        writer.text(&row.reviewer_key)?;
        writer.raw(b"<br>Role: ")?;
        writer.text(&row.reviewer_role)?;
        writer.raw(b"</td>")?;
        writer.cell(disposition(row.disposition))?;
        writer.cell(&row.responded_at)?;
        writer.cell(classification(row.classification))?;
        writer.raw(b"</tr>")?;
    }
    writer.raw(b"</tbody></table></section>\n")
}

/// Closed source labels; no debug rendering or unqualified user URI is emitted.
fn source_model(value: SourceModel) -> &'static str {
    match value {
        SourceModel::Mapping => "mapping",
        SourceModel::Catalog => "catalog",
        SourceModel::Profile => "profile",
        SourceModel::ResolvedCatalog => "resolved-catalog",
        SourceModel::ComponentDefinition => "component-definition",
        SourceModel::MappingManifest => "mapping-manifest",
        SourceModel::ApplicabilityManifest => "applicability-manifest",
        SourceModel::ApplicabilityReport => "applicability-report",
        SourceModel::LifecycleRecord => "lifecycle-record",
    }
}

/// Closed recorded item-state labels; quorum carries no domain transition authority.
fn item_state(value: ItemState) -> &'static str {
    match value {
        ItemState::Unassigned => "unassigned",
        ItemState::Assigned => "assigned",
        ItemState::InReview => "in-review",
        ItemState::Conflicted => "conflicted",
        ItemState::ChangesRequested => "changes-requested",
        ItemState::QuorumMet => "quorum-met",
        ItemState::Expired => "expired",
        ItemState::Stale => "stale",
    }
}

/// Closed disposition labels retain nonapprovals and historical supersession equally.
fn disposition(value: Disposition) -> &'static str {
    match value {
        Disposition::Approve => "approve",
        Disposition::Reject => "reject",
        Disposition::RequestChanges => "request-changes",
        Disposition::Abstain => "abstain",
        Disposition::Superseded => "superseded",
    }
}

/// Closed recorded classification labels remain assertions, not recalculated freshness.
fn classification(value: ResponseClassification) -> &'static str {
    match value {
        ResponseClassification::Current => "current",
        ResponseClassification::Stale => "stale",
        ResponseClassification::Foreign => "foreign",
        ResponseClassification::Future => "future",
        ResponseClassification::Late => "late",
        ResponseClassification::Unassigned => "unassigned",
        ResponseClassification::Superseded => "superseded",
        ResponseClassification::Conflicted => "conflicted",
    }
}

#[cfg(test)]
#[path = "html_tests.rs"]
mod tests;
