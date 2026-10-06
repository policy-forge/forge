//! Two minimized current Catalog App projections over the genuine complete /2 owner.
//! Every derived allocation uses that owner's existing byte/work tickets. No DTO is proof.

use super::super::super::capture_v2::{Admission, AdmittedValue};
use super::super::super::declarations_v2::{DeclarationAdmission, SearchSourceTuple};
use super::super::super::native_sources_v2::ServerNativeScopeV2;
use super::super::super::{
    catalog::Catalog, native_applicability_v2, native_work_v2 as work, wire,
};
use super::super::{
    CanonicalQuery, Citation, Data, Query, QueryError, QueryResponse, QueryResult, Reason,
    SourceSpan, page_result, page_start, safe_token, validate_identity,
};
use super::{ApplicabilityRow, GapData, GapRow};
use crate::applicability::model::{ClassificationCounts, GapClassification};
use crate::workspace::preparation::{Stage, WorkControl};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::{self, Write};

/// Retains complete admitted output and borrows its actual owner until publication completes.
pub(crate) struct PreparedQueryV2<'scope, 'control, C: WorkControl + ?Sized> {
    /// Real source owner, never a reconstructed query scope.
    scope: &'scope ServerNativeScopeV2<'control, C>,
    /// Exact minimized page produced only after the complete denominator was admitted.
    response: AdmittedValue<QueryResponse>,
}
impl<C: WorkControl + ?Sized> PreparedQueryV2<'_, '_, C> {
    /// Borrow the exact typed result without transferring its original byte ticket.
    pub(crate) fn response(&self) -> &QueryResponse {
        self.response.value()
    }
    /// Final whole-input verification after output construction/validation, without recapture.
    pub(crate) fn verify_inputs(&self) -> crate::workspace::preparation::WorkResult<()> {
        self.scope.verify_inputs()
    }
}

/// Keep ordinary query errors until the same original post-phase checkpoint.
fn phase<C: WorkControl + ?Sized, T>(
    admission: &mut Admission<'_, C>,
    result: QueryResult<T>,
) -> QueryResult<T> {
    work::fence(admission, Stage::PrepareDomain)?;
    result
}

/// Prepare only recorded applicability/gaps; every other advertised tool remains unavailable.
pub(in crate::mcp::artifact_status::queries) fn prepare<
    'scope,
    'control,
    C: WorkControl + ?Sized,
>(
    scope: &'scope ServerNativeScopeV2<'control, C>,
    query: &Query,
) -> QueryResult<PreparedQueryV2<'scope, 'control, C>> {
    let mut admission = scope.admission();
    work::fence(&mut admission, Stage::PrepareDomain)?;
    let result = prepare_inner(scope, query);
    // A domain failure still fences all actual hidden/config/present/absence originals.
    scope.verify_inputs()?;
    phase(&mut admission, result)
}

/// Reconcile the full current native framework before any subject filter or page selection.
fn prepare_inner<'scope, 'control, C: WorkControl + ?Sized>(
    scope: &'scope ServerNativeScopeV2<'control, C>,
    query: &Query,
) -> QueryResult<PreparedQueryV2<'scope, 'control, C>> {
    let mut admission = scope.admission();
    let Selection { key, subject, page, gaps } = select_query(scope, query, &mut admission)?;
    let sources = scope.sources();
    let plan = native_applicability_v2::prepare_selected(sources, key)?
        .ok_or(Reason::IncompleteClosure)?;
    let framework = &sources.policies()[plan.framework_index()];
    if !safe_token(&framework.citation_label) {
        return Err(Reason::VisibilityRefused.into());
    }
    let counts = plan.facts().counts();
    let (matched, mut logical) =
        complete_rows(scope, plan.facts(), counts, &framework.key, subject, &mut admission)?;
    let generation = generation(scope, query)?;
    let start = phase(&mut admission, page_start(generation.value(), page, matched))?;
    let emitted = matched.saturating_sub(start).min(page.limit);
    let slot = std::mem::size_of::<ApplicabilityRow>().max(std::mem::size_of::<GapRow>());
    logical = work::add(logical, work::multiply(emitted, slot, &mut admission)?, &mut admission)?;
    let mut ordinary_error = None;
    let response = admission.retain(logical, |admission| {
        let mut app_rows = Vec::with_capacity(if gaps { 0 } else { emitted });
        let mut gap_rows = Vec::with_capacity(if gaps { emitted } else { 0 });
        let mut offset = 0;
        for tuple in scope.search_sources() {
            admission
                .charge(1 + work::byte_work(tuple.artifact_key.len() + framework.key.len()))?;
        }
        let mut native =
            scope.search_sources().iter().filter(|row| row.artifact_key == framework.key);
        for row in plan.facts().control_rows() {
            admission.charge(1 + work::byte_work(row.control_id().len()))?;
            let tuple = native.next().ok_or_else(super::super::super::policy_failure)?;
            if subject.is_some_and(|selected| selected != row.control_id()) {
                continue;
            }
            let selected = offset >= start && offset < start + emitted;
            offset += 1;
            if !selected {
                continue;
            }
            // All strings and vector slots were reserved over the complete actual row operands.
            let width = tuple_width(tuple, row.control_id(), admission)?;
            admission.charge(work::byte_work(width))?;
            let cited = match citation(tuple, &framework.citation_label) {
                Ok(value) => value,
                Err(QueryError::Work(error)) => return Err(error),
                Err(QueryError::Unavailable(reason)) => {
                    ordinary_error = Some(reason);
                    work::fence(admission, Stage::RetainPrepared)?;
                    return Ok(super::super::unavailable(query, reason));
                }
            };
            let citations = vec![cited];
            if gaps {
                gap_rows.push(GapRow {
                    control_id: row.control_id().to_owned(),
                    classification: row.classification(),
                    citations,
                });
            } else {
                let decision = row.decision_state();
                app_rows.push(ApplicabilityRow {
                    control_id: row.control_id().to_owned(),
                    decision_state: decision,
                    decision_source: if decision.is_some() {
                        "explicit-record"
                    } else {
                        "implicit-under-review"
                    },
                    classification: row.classification(),
                    citations,
                });
            }
        }
        let data = response_data(
            gaps,
            generation.value().clone(),
            matched,
            start,
            app_rows,
            gap_rows,
            counts,
        );
        let response = QueryResponse {
            schema_version: "forge.mcp-query/1",
            tool: query.tool(),
            availability: "available",
            reason: None,
            data: Some(data),
        };
        // Allocation-free actual serializer counting charges every supplied chunk first.
        let mut counter = ChargedWriter::new(admission, super::super::MAX_RESPONSE);
        let result = serde_json::to_writer(&mut counter, &response);
        if result.is_err() {
            ordinary_error = Some(Reason::OutputBoundExceeded);
        }
        work::fence(admission, Stage::RetainPrepared)?;
        Ok(response)
    })?;
    if let Some(reason) = ordinary_error {
        return Err(reason.into());
    }
    Ok(PreparedQueryV2 { scope, response })
}

/// Plain borrowed query operands; no capture, currentness, native or disclosure authority.
struct Selection<'query> {
    /// Exact caller selector bytes retained through the original prepared operation.
    key: &'query str,
    /// Explicit native subject filter, or no filter over the complete denominator.
    subject: Option<&'query str>,
    /// Original finite generation-bound page controls, without a copied allowance.
    page: &'query super::super::Page,
    /// Exact existing choice of the two minimized App output families.
    gaps: bool,
}

/// Execute the existing complete query/profile operand admissions in their original order.
fn select_query<'query, C: WorkControl + ?Sized>(
    scope: &ServerNativeScopeV2<'_, C>,
    query: &'query Query,
    admission: &mut Admission<'_, C>,
) -> QueryResult<Selection<'query>> {
    admission.charge(1)?;
    let (key, subject, page, gaps) = match query {
        Query::GetRecordedApplicability { artifact_key, subject_id, page } => {
            (artifact_key.as_str(), subject_id.as_deref(), page, false)
        }
        Query::GetGapSummary { artifact_key, page } => (artifact_key.as_str(), None, page, true),
        _ => return Err(Reason::UnsupportedRole.into()),
    };
    let selection_width = work::add(key.len(), subject.map_or(0, str::len), admission)?;
    let selection_width =
        work::add(selection_width, page.cursor.as_ref().map_or(0, String::len), admission)?;
    admission.charge(1 + work::byte_work(selection_width))?;
    super::super::validate_query(query)?;
    let sources = scope.sources();
    let profile = sources.profile();
    for name in &profile.enabled_tools {
        admission.charge(1 + work::byte_work(name.len() + query.tool().len()))?;
    }
    for visible in &profile.visible_resource_keys {
        admission.charge(1 + work::byte_work(visible.len() + key.len()))?;
    }
    if !profile.enabled_tools.iter().any(|name| name == query.tool())
        || !profile.visible_resource_keys.iter().any(|visible| visible == key)
    {
        return Err(Reason::VisibilityRefused.into());
    }
    Ok(Selection { key, subject, page, gaps })
}

/// Inspect the complete actual native rows before filtering/paging or output growth.
fn complete_rows<C: WorkControl + ?Sized>(
    scope: &ServerNativeScopeV2<'_, C>,
    facts: &crate::applicability::captured::PreparedApplicability<'_>,
    counts: &ClassificationCounts,
    framework_key: &str,
    subject: Option<&str>,
    admission: &mut Admission<'_, C>,
) -> QueryResult<(usize, usize)> {
    if counts.total > 10_000 {
        return Err(Reason::OutputBoundExceeded.into());
    }
    let mut totals = [0usize; 7];
    let mut previous: Option<&str> = None;
    for tuple in scope.search_sources() {
        admission.charge(1 + work::byte_work(tuple.artifact_key.len() + framework_key.len()))?;
    }
    let mut native = scope.search_sources().iter().filter(|row| row.artifact_key == framework_key);
    let mut matched = 0usize;
    let mut logical = std::mem::size_of::<QueryResponse>() + 512;
    for row in facts.control_rows() {
        let id = row.control_id();
        admission.charge(1 + work::byte_work(id.len()))?;
        if !safe_token(id) || previous.is_some_and(|old| old >= id) {
            return Err(Reason::VisibilityRefused.into());
        }
        previous = Some(id);
        let tuple = native.next().ok_or(Reason::IncompleteClosure)?;
        admit_tuple(tuple, admission)?;
        if tuple.native_id != id || tuple.control_id != id {
            return Err(Reason::IncompleteClosure.into());
        }
        if totals[0] >= 10_000 {
            return Err(Reason::OutputBoundExceeded.into());
        }
        totals[0] += 1;
        totals[class_index(row.classification())] += 1;
        if subject.is_none_or(|selected| selected == id) {
            matched += 1;
        }
        // Admit copies for the complete denominator, including fields outside the page.
        logical = work::add(logical, tuple_width(tuple, id, admission)?, admission)?;
    }
    admission.charge(1)?;
    if native.next().is_some() || totals != count_tuple(counts) {
        return Err(Reason::IncompleteClosure.into());
    }
    Ok((matched, logical))
}

/// Consume already admitted complete row vectors into the unchanged minimized data shape.
fn response_data(
    gaps: bool,
    generation: String,
    matched: usize,
    start: usize,
    app_rows: Vec<ApplicabilityRow>,
    gap_rows: Vec<GapRow>,
    counts: &ClassificationCounts,
) -> Data {
    if gaps {
        let paged = page_result(generation, matched, start, gap_rows);
        Data::Gaps(GapData {
            generation: paged.generation,
            summary: counts.clone(),
            matched: paged.matched,
            emitted: paged.emitted,
            next_cursor: paged.next_cursor,
            rows: paged.rows,
        })
    } else {
        Data::Applicability(page_result(generation, matched, start, app_rows))
    }
}

/// Exact maintained classification labels, with checked denominator bounded before increments.
fn class_index(class: GapClassification) -> usize {
    match class {
        GapClassification::ApplicableMapped => 1,
        GapClassification::ApplicableReviewedNoRelationship => 2,
        GapClassification::ApplicableUnmapped => 3,
        GapClassification::NotApplicable => 4,
        GapClassification::Deferred => 5,
        GapClassification::UnderReview => 6,
    }
}
/// Compare all maintained complete totals without granting typed report equality from a subset.
fn count_tuple(counts: &ClassificationCounts) -> [usize; 7] {
    [
        counts.total,
        counts.applicable_mapped,
        counts.applicable_reviewed_no_relationship,
        counts.applicable_unmapped,
        counts.not_applicable,
        counts.deferred,
        counts.under_review,
    ]
}
/// Complete actually copied private/public tuple strings, not a raw file multiplier.
fn tuple_width<C: WorkControl + ?Sized>(
    tuple: &SearchSourceTuple,
    id: &str,
    admission: &mut Admission<'_, C>,
) -> crate::workspace::preparation::WorkResult<usize> {
    let mut width = std::mem::size_of::<Citation>()
        + std::mem::size_of::<Vec<Citation>>()
        + std::mem::size_of::<ApplicabilityRow>();
    for text in [
        id,
        tuple.artifact_key.as_str(),
        tuple.artifact_identity.model.as_str(),
        tuple.artifact_identity.root_id.as_str(),
        tuple.artifact_identity.document_version.as_str(),
        tuple.artifact_identity.oscal_version.as_str(),
        tuple.artifact_raw_sha256.as_str(),
        tuple.native_pointer.as_str(),
        tuple.source_key.as_str(),
        tuple.source_raw_sha256.as_str(),
    ] {
        admission.charge(1 + work::byte_work(text.len()))?;
        width = work::add(width, text.len(), admission)?;
    }
    // Citation labels are separately drawn from the actual policy; max128 is the closed schema domain.
    work::add(width, 128, admission)
}
/// Inspect every actual tuple before the page; no unsafe public ID/version is silently skipped.
fn admit_tuple<C: WorkControl + ?Sized>(
    tuple: &SearchSourceTuple,
    admission: &mut Admission<'_, C>,
) -> QueryResult<()> {
    let width = tuple_width(tuple, &tuple.native_id, admission)?;
    admission.charge(work::byte_work(width))?;
    validate_identity(&tuple.artifact_identity)?;
    if !safe_token(&tuple.artifact_key)
        || !safe_token(&tuple.source_key)
        || !safe_token(&tuple.control_id)
        || !safe_token(&tuple.native_id)
    {
        return Err(Reason::VisibilityRefused.into());
    }
    Ok(())
}
/// Native/source bytes, spans and identity came from the factory's actual complete tuple producer.
fn citation(tuple: &SearchSourceTuple, label: &str) -> QueryResult<Citation> {
    let start_byte =
        usize::try_from(tuple.source_span.start_byte).map_err(|_| Reason::SourceSpanUnavailable)?;
    let end_byte =
        usize::try_from(tuple.source_span.end_byte).map_err(|_| Reason::SourceSpanUnavailable)?;
    Ok(Citation {
        artifact_key: tuple.artifact_key.clone(),
        artifact_identity: Some(tuple.artifact_identity.clone()),
        artifact_raw_sha256: tuple.artifact_raw_sha256.clone(),
        citation_label: label.to_owned(),
        native_pointer: tuple.native_pointer.clone(),
        source_key: Some(tuple.source_key.clone()),
        source_raw_sha256: Some(tuple.source_raw_sha256.clone()),
        source_span: Some(SourceSpan { start_byte, end_byte }),
        source_state: "exact-captured-source",
        artifact_role: tuple.artifact_role,
    })
}

/// Hash canonical query data without cursor or page size, exactly as the existing /1 behavior.
fn generation<C: WorkControl + ?Sized>(
    scope: &ServerNativeScopeV2<'_, C>,
    query: &Query,
) -> QueryResult<AdmittedValue<String>> {
    let mut admission = scope.admission();
    let mut error = None;
    let digest = admission.retain(64 + std::mem::size_of::<String>(), |admission| {
        let mut writer = ChargedWriter::new(admission, super::super::MAX_ARGUMENTS);
        let result = (|| {
            writer.write_all(b"forge.mcp-query-generation/2\0")?;
            writer.write_all(scope.scope_generation().as_bytes())?;
            writer.write_all(b"\0")?;
            serde_json::to_writer(&mut writer, &CanonicalQuery::from(query))
                .map_err(io::Error::other)
        })();
        if result.is_err() {
            error = Some(Reason::OutputBoundExceeded);
        }
        let hash = writer.hash.finalize();
        work::fence(admission, Stage::PrepareDomain)?;
        Ok(crate::hashing::lower_hex(&hash))
    })?;
    if let Some(reason) = error {
        return Err(reason.into());
    }
    Ok(digest)
}

/// Allocation-free counting/hashing writer charges every exact serializer chunk before touching it.
struct ChargedWriter<'a, 'control, C: WorkControl + ?Sized> {
    /// Original shared admission, never a fresh serializer budget.
    admission: &'a mut Admission<'control, C>,
    /// Fixed complete encoded-byte domain.
    limit: usize,
    /// Actual admitted complete encoded bytes.
    bytes: usize,
    /// Incremental SHA state; counting also retains only this fixed state.
    hash: Sha256,
    /// Actual admitted serializer chunk count, used before later complete wire passes.
    chunks: usize,
}
impl<'a, 'control, C: WorkControl + ?Sized> ChargedWriter<'a, 'control, C> {
    /// Build fixed-size writer state; no copied input or retained canonical JSON.
    fn new(admission: &'a mut Admission<'control, C>, limit: usize) -> Self {
        Self { admission, limit, bytes: 0, hash: Sha256::new(), chunks: 0 }
    }
}
impl<C: WorkControl + ?Sized> Write for ChargedWriter<'_, '_, C> {
    /// Precharge each serializer chunk and preserve Capacity/Failed/Interrupted in the original ledger.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let next = self
            .bytes
            .checked_add(bytes.len())
            .filter(|n| *n <= self.limit)
            .ok_or_else(|| io::Error::other("complete-response-bound"))?;
        self.admission
            .charge(1 + work::byte_work(bytes.len()))
            .map_err(|_| io::Error::other("original-owner-stop"))?;
        self.chunks = self.chunks.checked_add(1).ok_or_else(|| {
            let _first = self.admission.capacity();
            io::Error::other("original-owner-stop")
        })?;
        self.hash.update(bytes);
        self.bytes = next;
        Ok(bytes.len())
    }
    /// No hidden retained output exists.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Reserve the finite whole envelope before either actual wire serialization pass and schema parse.
pub(in crate::mcp::artifact_status::queries) fn encode<C: WorkControl + ?Sized>(
    scope: &ServerNativeScopeV2<'_, C>,
    response: &QueryResponse,
    id: &wire::RequestId,
    catalog: &Catalog,
    is_error: bool,
) -> QueryResult<AdmittedValue<Vec<u8>>> {
    // This stack-only exact envelope view has the same Serialize field/order shape as
    // wire::Success. Count the complete real ToolResult and actual request ID, including
    // escaped string-ID chunks, before reserving or invoking either actual wire pass.
    /// Borrow the complete fixed JSON-RPC envelope without allocating another response.
    #[derive(Serialize)]
    struct Envelope<'a> {
        /// Fixed JSON-RPC envelope revision.
        jsonrpc: &'static str,
        /// Actual admitted request ID with original representation.
        id: &'a wire::RequestId,
        /// Actual fixed complete tool wrapper around the already retained response.
        result: wire::ToolResult<'a, QueryResponse>,
    }
    let mut admission = scope.admission();
    work::fence(&mut admission, Stage::RetainPrepared)?;
    let mut counter = ChargedWriter::new(&mut admission, wire::MAX_RESPONSE - 1);
    let count_result = serde_json::to_writer(
        &mut counter,
        &Envelope { jsonrpc: "2.0", id, result: wire::ToolResult::new(response, is_error) },
    );
    let bytes = counter.bytes;
    let chunks = counter.chunks;
    let ordinary = count_result.map_err(|_| QueryError::Unavailable(Reason::OutputBoundExceeded));
    phase(&mut admission, ordinary)?;
    let complete = work::add(bytes, 1, &mut admission)?;
    let serialization = work::multiply(chunks, 2, &mut admission)?;
    let serialization = work::add(
        serialization,
        work::multiply(work::byte_work(bytes), 2, &mut admission)?,
        &mut admission,
    )?;
    let logical_output = work::add(complete, std::mem::size_of::<Vec<u8>>(), &mut admission)?;
    let mut failed = None;
    let encoded = admission.retain(logical_output, |admission| {
        work::run(admission, serialization, Stage::RetainPrepared, |_| {
            if let Ok(bytes) = wire::success(id, &wire::ToolResult::new(response, is_error)) {
                Ok(bytes)
            } else {
                failed = Some(Reason::OutputBoundExceeded);
                Ok(Vec::new())
            }
        })
    })?;
    if let Some(reason) = failed {
        return phase(&mut admission, Err(reason.into()));
    }
    if encoded.value().len() != complete {
        return phase(&mut admission, Err(Reason::InvalidArtifact.into()));
    }
    let emitted = match response.data.as_ref() {
        Some(Data::Applicability(page)) => page.emitted,
        Some(Data::Gaps(page)) => page.emitted,
        Some(Data::Policies(page)) => page.emitted,
        Some(Data::Search(page)) => page.emitted,
        Some(Data::Trace(page)) => page.emitted,
        Some(Data::Requirement(_) | Data::Status(_)) => 1,
        None => 0,
    };
    let raw = encoded.value();
    let (nodes, logical) = work::raw_bound(raw, wire::MAX_RESPONSE, &mut admission)?;
    let schema = catalog.output_schema(response.tool).ok_or(Reason::InvalidArtifact)?;
    let schema_operand = work::operand(schema, &mut admission)?;
    // Each fixed schema repeats its complete item predicates once per emitted row;
    // full schema traversal includes both anyOf and allOf copies, required/enums/patterns.
    let validations = work::multiply(schema_operand.work, emitted + 1, &mut admission)?;
    let raw_work = work::add(nodes, work::byte_work(raw.len()), &mut admission)?;
    let validations = work::add(validations, raw_work, &mut admission)?;
    let checked = admission.retain(logical, |admission| {
        work::run(admission, validations, Stage::ValidateResource, |_| {
            Ok(catalog.validate_encoded(response.tool, raw).is_ok())
        })
    })?;
    let result = if *checked.value() { Ok(encoded) } else { Err(Reason::InvalidArtifact.into()) };
    phase(&mut admission, result)
}
