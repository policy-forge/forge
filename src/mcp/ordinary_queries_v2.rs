//! Five minimized native projections from the actual complete ServerRead owner.
//! Plain rows and query digests never create approval, capture or currentness.

use super::super::ResourcePolicy;
use super::super::capture_v2::{Admission, AdmittedValue};
use super::super::declarations_v2::{DeclarationAdmission, SearchSourceTuple};
use super::super::native_sources_v2::ServerNativeScopeV2;
use super::super::native_work_v2 as work;
use super::{
    CanonicalQuery, Citation, Data, Digest, MAX_ARGUMENTS, MAX_QUERY, MAX_TOKENS, NativeIdentity,
    Page, PolicyRow, Query, QueryError, QueryResponse, QueryResult, Reason, RequirementRow, Role,
    SearchRow, Sha256, SourceSpan, Stage, StatusRow, TraceRow, WorkControl, Write, io, page_result,
    page_start, safe_token, validate_identity,
};
use crate::workspace::preparation::WorkResult;

/// Actual prepared data keeps its byte ticket and genuine original owner until publication.
pub(in crate::mcp::artifact_status) struct PreparedOrdinaryV2<
    'scope,
    'control,
    C: WorkControl + ?Sized,
> {
    /// Only the complete genuine receiver can supply this private native owner.
    scope: &'scope ServerNativeScopeV2<'control, C>,
    /// Complete minimized result, retained against the original operation allowance.
    response: AdmittedValue<QueryResponse>,
}
impl<C: WorkControl + ?Sized> PreparedOrdinaryV2<'_, '_, C> {
    /// Borrow exact output without transferring its retained ticket or native owner.
    pub(super) fn response(&self) -> &QueryResponse {
        self.response.value()
    }
    /// Repeat full physical/config/hidden/absence checks after actual finite output validation.
    pub(super) fn verify_inputs(&self) -> WorkResult<()> {
        self.scope.verify_inputs()
    }
}

/// Fence ordinary results and success on the same original controller; sticky stops win.
fn phase<C: WorkControl + ?Sized, T>(
    a: &mut Admission<'_, C>,
    result: QueryResult<T>,
) -> QueryResult<T> {
    work::fence(a, Stage::PrepareDomain)?;
    result
}

/// Retain complete native preparation before its unconditional full-original and phase fences.
pub(super) fn prepare<'scope, 'control, C: WorkControl + ?Sized>(
    scope: &'scope ServerNativeScopeV2<'control, C>,
    query: &Query,
) -> QueryResult<PreparedOrdinaryV2<'scope, 'control, C>> {
    let mut a = scope.admission();
    work::fence(&mut a, Stage::PrepareDomain)?;
    let result = prepare_inner(scope, query);
    scope.verify_inputs()?;
    phase(&mut a, result)
}

/// Plain borrowed requirement facts are derived from this owner's complete immutable tuples.
struct Fact<'a> {
    /// Exact full native-issued source tuple; no caller-created counterpart is accepted.
    tuple: &'a SearchSourceTuple,
    /// Exact current declaration's minimized token, separate from private source spelling.
    label: &'a str,
    /// Checked actual byte offsets, without truncating either native u64 operand.
    span: SourceSpan,
}

/// Plain immutable projected policy operands established by a real approved/current view.
struct Policy<'a> {
    /// Complete actual declaration, already correlated with native/raw/current closure facts.
    row: &'a ResourcePolicy,
    /// Actual identity equality was tested by the native view; `PolicySource` remains null.
    identity: Option<&'a NativeIdentity>,
}

/// Admit exact whole query operands before validation, traversal or matching.
fn request<C: WorkControl + ?Sized>(
    scope: &ServerNativeScopeV2<'_, C>,
    query: &Query,
    a: &mut Admission<'_, C>,
) -> QueryResult<()> {
    let canonical = CanonicalQuery::from(query);
    let mut width = query.tool().len();
    for text in canonical.artifact_key.into_iter().chain(canonical.selection) {
        width = work::add(width, text.len(), a)?;
    }
    if let Some(page) = query_page(query) {
        width = work::add(width, page.cursor.as_ref().map_or(0, String::len), a)?;
    }
    a.charge(1 + work::byte_work(width))?;
    super::validate_query_v2(query)?;
    let profile = scope.sources().profile();
    for tool in &profile.enabled_tools {
        let width = work::add(tool.len(), query.tool().len(), a)?;
        a.charge(1 + work::byte_work(width))?;
    }
    if !profile.enabled_tools.iter().any(|name| name == query.tool()) {
        // Preserve the retained ordinary-family unsupported-operation reason and postphase.
        return Err(Reason::UnsupportedRole.into());
    }
    if matches!(
        query,
        Query::SearchRequirements { include_excerpt: true, .. }
            | Query::GetRequirement { include_excerpt: true, .. }
    ) {
        return Err(Reason::VisibilityRefused.into());
    }
    if let Some(key) = canonical.artifact_key {
        visible(scope, key, a)?;
    }
    Ok(())
}

/// Whole configured key comparisons precede actual selection; no hidden discovery is inferred.
fn visible<C: WorkControl + ?Sized>(
    scope: &ServerNativeScopeV2<'_, C>,
    key: &str,
    a: &mut Admission<'_, C>,
) -> QueryResult<()> {
    if !is_visible(scope, key, a)? {
        return Err(Reason::VisibilityRefused.into());
    }
    Ok(())
}

/// Charge the complete actual key roster before one visibility comparison pass.
fn is_visible<C: WorkControl + ?Sized>(
    scope: &ServerNativeScopeV2<'_, C>,
    key: &str,
    a: &mut Admission<'_, C>,
) -> WorkResult<bool> {
    for actual in &scope.sources().profile().visible_resource_keys {
        let width = work::add(actual.len(), key.len(), a)?;
        a.charge(1 + work::byte_work(width))?;
    }
    Ok(scope.sources().profile().visible_resource_keys.iter().any(|actual| actual == key))
}

/// Measure every private tuple string before complete projection validation or string copies.
fn tuple_width<C: WorkControl + ?Sized>(
    tuple: &SearchSourceTuple,
    a: &mut Admission<'_, C>,
) -> WorkResult<usize> {
    let mut width = 0;
    for text in [
        &tuple.artifact_key,
        &tuple.artifact_identity.model,
        &tuple.artifact_identity.root_id,
        &tuple.artifact_identity.document_version,
        &tuple.artifact_identity.oscal_version,
        &tuple.artifact_raw_sha256,
        &tuple.native_id,
        &tuple.control_id,
        &tuple.native_pointer,
        &tuple.source_key,
        &tuple.source_raw_sha256,
        &tuple.lifecycle_key,
    ] {
        a.charge(1)?;
        width = work::add(width, text.len(), a)?;
    }
    Ok(width)
}

/// Use actual metadata and complete UTF-8 source bytes before constructing a checked citation.
fn fact<'scope, C: WorkControl + ?Sized>(
    scope: &'scope ServerNativeScopeV2<'_, C>,
    tuple: &'scope SearchSourceTuple,
    a: &mut Admission<'_, C>,
) -> QueryResult<Fact<'scope>> {
    let width = tuple_width(tuple, a)?;
    a.charge(1 + work::byte_work(width))?;
    validate_identity(&tuple.artifact_identity)?;
    if !matches!(tuple.artifact_role, Role::OscalCatalogArtifact | Role::OscalComponentArtifact)
        || !super::native_selector(&tuple.native_id)
        || !super::native_selector(&tuple.control_id)
        || !safe_token(&tuple.artifact_key)
        || !safe_token(&tuple.source_key)
        || tuple.native_pointer.is_empty()
        || tuple.native_pointer.len() > 4096
    {
        return Err(Reason::VisibilityRefused.into());
    }
    for row in scope.sources().policies() {
        let width = work::add(row.key.len(), tuple.artifact_key.len(), a)?;
        a.charge(1 + work::byte_work(width))?;
    }
    let row = scope
        .sources()
        .policies()
        .iter()
        .find(|row| row.key == tuple.artifact_key)
        .ok_or(Reason::IncompleteClosure)?;
    a.charge(1 + work::byte_work(row.citation_label.len()))?;
    if !safe_token(&row.citation_label) {
        return Err(Reason::VisibilityRefused.into());
    }
    let source = scope.sources().resource(&tuple.source_key)?.ok_or(Reason::IncompleteClosure)?;
    let raw = source.bytes()?;
    let span = checked_span(raw, &tuple.source_span, a)?;
    let actual_hash = source.raw_sha256()?;
    let hashes = work::add(actual_hash.len(), tuple.source_raw_sha256.len(), a)?;
    a.charge(1 + work::byte_work(hashes))?;
    if actual_hash != tuple.source_raw_sha256 {
        return Err(Reason::Noncurrent.into());
    }
    Ok(Fact { tuple, label: &row.citation_label, span })
}

/// Actual raw UTF-8 and exact slice passes are charged before checked conversions and boundaries.
fn checked_span<C: WorkControl + ?Sized>(
    raw: &[u8],
    span: &super::super::declarations_v2::SourceSpan,
    a: &mut Admission<'_, C>,
) -> QueryResult<SourceSpan> {
    a.charge(1 + work::byte_work(raw.len()))?;
    let start = usize::try_from(span.start_byte).map_err(|_| Reason::SourceSpanUnavailable)?;
    let end = usize::try_from(span.end_byte).map_err(|_| Reason::SourceSpanUnavailable)?;
    let text = std::str::from_utf8(raw).map_err(|_| Reason::SourceSpanUnavailable)?;
    if start >= end
        || end > raw.len()
        || !text.is_char_boundary(start)
        || !text.is_char_boundary(end)
    {
        return Err(Reason::SourceSpanUnavailable.into());
    }
    a.charge(1 + work::byte_work(end - start))?;
    text.get(start..end).ok_or(Reason::SourceSpanUnavailable)?;
    Ok(SourceSpan { start_byte: start, end_byte: end })
}

/// Reserve the complete actual tuple plan before the first collection grows, including off-page rows.
fn facts<'scope, C: WorkControl + ?Sized>(
    scope: &'scope ServerNativeScopeV2<'_, C>,
    a: &mut Admission<'_, C>,
) -> QueryResult<AdmittedValue<Vec<Fact<'scope>>>> {
    let slots = work::multiply(scope.search_sources().len(), std::mem::size_of::<Fact<'_>>(), a)?;
    let logical = work::add(slots, std::mem::size_of::<Vec<Fact<'_>>>(), a)?;
    let mut ordinary = None;
    let result = a.retain(logical, |a| {
        let mut rows = Vec::with_capacity(scope.search_sources().len());
        for tuple in scope.search_sources() {
            match fact(scope, tuple, a) {
                Ok(row) => rows.push(row),
                Err(QueryError::Work(error)) => return Err(error),
                Err(QueryError::Unavailable(reason)) => {
                    ordinary = Some(reason);
                    return Ok(Vec::new());
                }
            }
        }
        Ok(rows)
    })?;
    phase(a, ordinary.map_or(Ok(result), |reason| Err(reason.into())))
}

/// Measure a complete policy copy and validate the actual neutral native status without a schedule.
fn policy<'scope, C: WorkControl + ?Sized>(
    scope: &'scope ServerNativeScopeV2<'_, C>,
    row: &'scope ResourcePolicy,
    a: &mut Admission<'_, C>,
) -> QueryResult<Policy<'scope>> {
    let approved = scope.sources().approved(&row.key)?.ok_or(Reason::Unapproved)?;
    let resource = approved.resource();
    a.charge(1 + work::byte_work(resource.citation_label().len()))?;
    if !safe_token(resource.citation_label()) {
        return Err(Reason::VisibilityRefused.into());
    }
    let neutral = approved.neutral_status();
    a.charge(1 + work::byte_work(neutral.derived_status.len()))?;
    if neutral.state != crate::lifecycle::record::LifecycleState::Approved
        || neutral.derived_status != "approved"
    {
        return Err(Reason::Noncurrent.into());
    }
    if row.role.native_model().is_some() && resource.native_identity().is_none() {
        return Err(Reason::InvalidArtifact.into());
    }
    if let Some(identity) = resource.native_identity() {
        identity_width(identity, a)?;
        validate_identity(identity)?;
    }
    Ok(Policy { row, identity: row.native_identity.as_ref() })
}

/// Admit every actual identity string before validation and later copies.
fn identity_width<C: WorkControl + ?Sized>(
    identity: &NativeIdentity,
    a: &mut Admission<'_, C>,
) -> WorkResult<usize> {
    let mut width = 0;
    for text in
        [&identity.model, &identity.root_id, &identity.document_version, &identity.oscal_version]
    {
        a.charge(1)?;
        width = work::add(width, text.len(), a)?;
    }
    a.charge(1 + work::byte_work(width))?;
    Ok(width)
}

/// Prepare all visible native policies before canonical ordering and paging, excluding source rows.
fn policies<'scope, C: WorkControl + ?Sized>(
    scope: &'scope ServerNativeScopeV2<'_, C>,
    a: &mut Admission<'_, C>,
) -> QueryResult<AdmittedValue<Vec<Policy<'scope>>>> {
    let count = scope.sources().policies().len();
    let slots = work::multiply(count, std::mem::size_of::<Policy<'_>>(), a)?;
    let logical = work::add(slots, std::mem::size_of::<Vec<Policy<'_>>>(), a)?;
    let mut ordinary = None;
    let rows = a.retain(logical, |a| {
        let mut rows = Vec::with_capacity(count);
        for row in scope.sources().policies() {
            a.charge(1)?;
            if !matches!(row.role, Role::OscalCatalogArtifact | Role::OscalComponentArtifact) {
                continue;
            }
            if !is_visible(scope, &row.key, a)? {
                continue;
            }
            match policy(scope, row, a) {
                Ok(row) => rows.push(row),
                Err(QueryError::Work(error)) => return Err(error),
                Err(QueryError::Unavailable(reason)) => {
                    ordinary = Some(reason);
                    return Ok(Vec::new());
                }
            }
        }
        let mut width = 0;
        for row in &rows {
            a.charge(1)?;
            width = work::add(width, row.row.key.len(), a)?;
        }
        let comparisons = work::multiply(rows.len(), rows.len(), a)?;
        let repeated = work::multiply(comparisons, 1 + work::byte_work(width), a)?;
        work::run(a, repeated, Stage::PrepareDomain, |_| {
            rows.sort_unstable_by(|left, right| left.row.key.cmp(&right.row.key));
            Ok(())
        })?;
        Ok(rows)
    })?;
    phase(a, ordinary.map_or(Ok(rows), |reason| Err(reason.into())))
}

/// Complete response copy envelope covers every native fact, even those outside selection/page.
fn response_width<C: WorkControl + ?Sized>(
    facts: &[Fact<'_>],
    a: &mut Admission<'_, C>,
) -> WorkResult<usize> {
    let mut logical = std::mem::size_of::<QueryResponse>() + 1024;
    let headers = std::mem::size_of::<SearchRow>()
        + std::mem::size_of::<TraceRow>()
        + std::mem::size_of::<Citation>()
        + std::mem::size_of::<NativeIdentity>()
        + 4 * std::mem::size_of::<&str>();
    for row in facts {
        let width = tuple_width(row.tuple, a)?;
        let repeated = work::multiply(width, 3, a)?;
        let strings = work::add(repeated, row.label.len(), a)?;
        let total = work::add(strings, headers, a)?;
        logical = work::add(logical, total, a)?;
    }
    Ok(logical)
}

/// Minimized one-citation row copies only complete already admitted native/source operands.
fn requirement(row: &Fact<'_>) -> RequirementRow {
    let tuple = row.tuple;
    RequirementRow {
        requirement_id: tuple.native_id.clone(),
        artifact_key: tuple.artifact_key.clone(),
        citations: vec![Citation {
            artifact_key: tuple.artifact_key.clone(),
            artifact_identity: Some(tuple.artifact_identity.clone()),
            artifact_raw_sha256: tuple.artifact_raw_sha256.clone(),
            citation_label: row.label.to_owned(),
            native_pointer: tuple.native_pointer.clone(),
            source_key: Some(tuple.source_key.clone()),
            source_raw_sha256: Some(tuple.source_raw_sha256.clone()),
            source_span: Some(row.span.clone()),
            source_state: "exact-captured-source",
            artifact_role: tuple.artifact_role,
        }],
        excerpt: None,
    }
}

/// Copy a genuinely established neutral status; private record fields and schedule stay absent.
fn status(row: &Policy<'_>) -> StatusRow {
    StatusRow {
        artifact_key: row.row.key.clone(),
        native_identity: row.identity.cloned(),
        raw_sha256: row.row.expected_sha256.clone(),
        citation_label: row.row.citation_label.clone(),
        lifecycle_state: "approved",
        freshness_scope: "complete-recorded-fingerprint-current",
        schedule_scope: "not-evaluated",
        schedule: None,
    }
}

/// Closed page-bearing operations; exact requirement/status results have no implicit page.
fn query_page(query: &Query) -> Option<&Page> {
    match query {
        Query::ListPolicies(page)
        | Query::SearchRequirements { page, .. }
        | Query::TraceControl { page, .. } => Some(page),
        _ => None,
    }
}

/// Plain output operands; they hold no native capability or separately accepted allowance.
#[derive(Clone, Copy)]
struct Projection<'a> {
    /// Exact typed requested operation after original-owner admission.
    query: &'a Query,
    /// Plain actual complete query generation used by this operation's page.
    generation: &'a str,
    /// Complete already measured logical output copy envelope.
    logical: usize,
}

/// Actual whole-universe preparation precedes dispatch, selection, matching and row allocation.
fn prepare_inner<'scope, 'control, C: WorkControl + ?Sized>(
    scope: &'scope ServerNativeScopeV2<'control, C>,
    query: &Query,
) -> QueryResult<PreparedOrdinaryV2<'scope, 'control, C>> {
    let mut a = scope.admission();
    request(scope, query, &mut a)?;
    let all = facts(scope, &mut a)?;
    let logical = response_width(all.value(), &mut a)?;
    let generation = generation(scope, query)?;
    let projection = Projection { query, generation: generation.value(), logical };
    let data = match query {
        Query::ListPolicies(page) => list(scope, page, projection, &mut a)?,
        Query::GetArtifactStatus { artifact_key } => {
            artifact_status(scope, query, artifact_key, logical, &mut a)?
        }
        Query::GetRequirement { artifact_key, requirement_id, .. } => {
            exact(all.value(), artifact_key, requirement_id, projection, &mut a)?
        }
        Query::TraceControl { artifact_key, control_id, page } => {
            trace(all.value(), artifact_key, control_id, page, projection, &mut a)?
        }
        Query::SearchRequirements { query: text, artifact_key, page, .. } => {
            search(scope, all.value(), text, artifact_key.as_deref(), page, projection, &mut a)?
        }
        _ => return Err(Reason::UnsupportedRole.into()),
    };
    Ok(PreparedOrdinaryV2 { scope, response: data })
}

/// Copy exact output strings only after measuring the complete policy payload and headers.
fn policy_width<C: WorkControl + ?Sized>(
    row: &Policy<'_>,
    a: &mut Admission<'_, C>,
) -> WorkResult<usize> {
    let mut width = work::add(row.row.key.len(), row.row.expected_sha256.len(), a)?;
    width = work::add(width, row.row.citation_label.len(), a)?;
    if let Some(identity) = row.identity {
        let identity_bytes = identity_width(identity, a)?;
        width = work::add(width, identity_bytes, a)?;
    }
    work::add(width, std::mem::size_of::<PolicyRow>() + std::mem::size_of::<NativeIdentity>(), a)
}

/// Reserve page/cursor payload before existing pure page assembly formats or clones anything.
fn page_plan<C: WorkControl + ?Sized>(
    generation: &str,
    page: &Page,
    matched: usize,
    a: &mut Admission<'_, C>,
) -> QueryResult<(usize, usize)> {
    a.charge(1 + work::byte_work(generation.len() + page.cursor.as_ref().map_or(0, String::len)))?;
    let start = phase(a, page_start(generation, page, matched))?;
    let emitted = matched.saturating_sub(start).min(page.limit);
    let _next = work::add(start, emitted, a)?;
    Ok((start, emitted))
}

/// Actual retained response is built from private complete plans, never detached caller rows.
fn response<'control, C: WorkControl + ?Sized>(
    query: &Query,
    logical: usize,
    a: &mut Admission<'control, C>,
    build: impl FnOnce(&mut Admission<'control, C>) -> WorkResult<Data>,
) -> QueryResult<AdmittedValue<QueryResponse>> {
    a.retain(logical, |a| {
        let data = build(a)?;
        Ok(QueryResponse {
            schema_version: "forge.mcp-query/1",
            tool: query.tool(),
            availability: "available",
            reason: None,
            data: Some(data),
        })
    })
    .map_err(QueryError::Work)
}

/// Whole visible policy count is canonicalized before selecting a complete page.
fn list<C: WorkControl + ?Sized>(
    scope: &ServerNativeScopeV2<'_, C>,
    page: &Page,
    projection: Projection<'_>,
    a: &mut Admission<'_, C>,
) -> QueryResult<AdmittedValue<QueryResponse>> {
    let Projection { query, generation, mut logical } = projection;
    let policies = policies(scope, a)?;
    for row in policies.value() {
        logical = work::add(logical, policy_width(row, a)?, a)?;
    }
    let matched = policies.value().len();
    let (start, emitted) = page_plan(generation, page, matched, a)?;
    response(query, logical, a, |a| {
        a.charge(1 + work::byte_work(generation.len() * 2 + 32))?;
        let mut rows = Vec::with_capacity(emitted);
        for row in policies.value().iter().skip(start).take(emitted) {
            let width = policy_width(row, a)?;
            a.charge(1 + work::byte_work(width))?;
            let row = status(row);
            let native_identity = row.native_identity.ok_or_else(super::super::policy_failure)?;
            rows.push(PolicyRow {
                artifact_key: row.artifact_key,
                native_identity,
                raw_sha256: row.raw_sha256,
                lifecycle_state: row.lifecycle_state,
                freshness_scope: row.freshness_scope,
                schedule_scope: row.schedule_scope,
                citation_label: row.citation_label,
                schedule: row.schedule,
            });
        }
        Ok(Data::Policies(page_result(generation.to_owned(), matched, start, rows)))
    })
}

/// Exact status retains a genuine approved/current view, with explicit null source identity.
fn artifact_status<C: WorkControl + ?Sized>(
    scope: &ServerNativeScopeV2<'_, C>,
    query: &Query,
    key: &str,
    logical: usize,
    a: &mut Admission<'_, C>,
) -> QueryResult<AdmittedValue<QueryResponse>> {
    let index = scope.sources().index(key)?.ok_or(Reason::NotFound)?;
    let row = &scope.sources().policies()[index];
    if !matches!(
        row.role,
        Role::PolicySource | Role::OscalCatalogArtifact | Role::OscalComponentArtifact
    ) {
        return Err(Reason::UnsupportedRole.into());
    }
    let row = policy(scope, row, a)?;
    let width = policy_width(&row, a)?;
    let logical = work::add(logical, width, a)?;
    response(query, logical, a, |a| {
        a.charge(1 + work::byte_work(width))?;
        Ok(Data::Status(status(&row)))
    })
}

/// Charge complete comparison operands before exact native selector matching or lookup.
fn selection<C: WorkControl + ?Sized>(
    all: &[Fact<'_>],
    key: Option<&str>,
    id: &str,
    a: &mut Admission<'_, C>,
) -> WorkResult<()> {
    for row in all {
        let tuple = row.tuple;
        let mut width = work::add(tuple.artifact_key.len(), key.map_or(0, str::len), a)?;
        width = work::add(width, tuple.native_id.len(), a)?;
        width = work::add(width, tuple.control_id.len(), a)?;
        width = work::add(width, id.len(), a)?;
        a.charge(1 + work::byte_work(width))?;
    }
    Ok(())
}

/// Native exact requirement uses its genuine source line and original identifier spelling.
fn exact<C: WorkControl + ?Sized>(
    all: &[Fact<'_>],
    key: &str,
    id: &str,
    projection: Projection<'_>,
    a: &mut Admission<'_, C>,
) -> QueryResult<AdmittedValue<QueryResponse>> {
    let Projection { query, logical, .. } = projection;
    selection(all, Some(key), id, a)?;
    let row = all
        .iter()
        .find(|row| row.tuple.artifact_key == key && row.tuple.native_id == id)
        .ok_or(Reason::NotFound)?;
    let width = tuple_width(row.tuple, a)?;
    response(query, logical, a, |a| {
        a.charge(1 + work::byte_work(width))?;
        Ok(Data::Requirement(requirement(row)))
    })
}

/// Complete direct native control relations conserve every Component/Capability requirement.
fn trace<C: WorkControl + ?Sized>(
    all: &[Fact<'_>],
    key: &str,
    id: &str,
    page: &Page,
    projection: Projection<'_>,
    a: &mut Admission<'_, C>,
) -> QueryResult<AdmittedValue<QueryResponse>> {
    let Projection { query, generation, logical } = projection;
    selection(all, Some(key), id, a)?;
    let matched = all
        .iter()
        .filter(|row| row.tuple.artifact_key == key && row.tuple.control_id == id)
        .count();
    let (start, emitted) = page_plan(generation, page, matched, a)?;
    selection(all, Some(key), id, a)?;
    response(query, logical, a, |a| {
        a.charge(1 + work::byte_work(generation.len() * 2 + 32))?;
        let mut rows = Vec::with_capacity(emitted);
        for row in all
            .iter()
            .filter(|row| row.tuple.artifact_key == key && row.tuple.control_id == id)
            .skip(start)
            .take(emitted)
        {
            let width = tuple_width(row.tuple, a)?;
            a.charge(1 + work::byte_work(width))?;
            let requirement = requirement(row);
            rows.push(TraceRow {
                control_id: row.tuple.control_id.clone(),
                requirement_id: requirement.requirement_id,
                mapping_id: None,
                relation: "exact-recorded-control",
                citations: requirement.citations,
                evidence_metadata: None,
            });
        }
        Ok(Data::Trace(page_result(generation.to_owned(), matched, start, rows)))
    })
}

/// Allocation-free scalar/lowercase measurement precedes every derived token allocation.
fn token_plan<C: WorkControl + ?Sized>(
    text: &str,
    a: &mut Admission<'_, C>,
) -> QueryResult<(usize, usize)> {
    a.charge(1 + work::byte_work(text.len()))?;
    if text.is_empty() || text.len() > MAX_QUERY || text.chars().any(char::is_control) {
        return Err(Reason::NotFound.into());
    }
    let mut width = 0;
    let mut current = 0;
    let mut tokens = 0;
    for character in text.chars().chain(std::iter::once(' ')) {
        a.charge(1)?;
        if character.is_alphanumeric() {
            for lowered in character.to_lowercase() {
                a.charge(1)?;
                current = work::add(current, lowered.len_utf8(), a)?;
                width = work::add(width, lowered.len_utf8(), a)?;
                if current > MAX_QUERY {
                    return Err(Reason::NotFound.into());
                }
            }
        } else if current != 0 {
            tokens = work::add(tokens, 1, a)?;
            current = 0;
        }
    }
    if tokens == 0 {
        return Err(Reason::NotFound.into());
    }
    Ok((width, tokens))
}

/// Build fully measured lowercase tokens; compare duplicates on admitted complete operands.
fn search_tokens<C: WorkControl + ?Sized>(
    text: &str,
    a: &mut Admission<'_, C>,
) -> QueryResult<AdmittedValue<Vec<String>>> {
    let (width, count) = token_plan(text, a)?;
    let slots = work::multiply(count.min(MAX_TOKENS) + 1, std::mem::size_of::<String>(), a)?;
    let payload = work::multiply(width, 2, a)?;
    let logical = work::add(payload, slots + std::mem::size_of::<Vec<String>>(), a)?;
    let mut ordinary = None;
    let tokens = a.retain(logical, |a| {
        let mut result: Vec<String> = Vec::with_capacity(count.min(MAX_TOKENS));
        let mut token = String::with_capacity(width);
        for character in text.chars().chain(std::iter::once(' ')) {
            a.charge(1)?;
            if character.is_alphanumeric() {
                for lowered in character.to_lowercase() {
                    a.charge(1)?;
                    token.push(lowered);
                }
            } else if !token.is_empty() {
                for old in &result {
                    let width = work::add(old.len(), token.len(), a)?;
                    a.charge(1 + work::byte_work(width))?;
                }
                if !result.contains(&token) {
                    if result.len() == MAX_TOKENS {
                        ordinary = Some(Reason::NotFound);
                        return Ok(Vec::new());
                    }
                    // Reserve total measured payload before each exact token copy; scratch capacity stays live.
                    a.charge(1 + work::byte_work(token.len()))?;
                    result.push(token.clone());
                }
                token.clear();
            }
        }
        Ok(result)
    })?;
    phase(a, ordinary.map_or(Ok(tokens), |reason| Err(reason.into())))
}

/// Compare whole source tokens; a mismatched word still scans every scalar until its delimiter.
fn contains<C: WorkControl + ?Sized>(
    text: &str,
    token: &str,
    a: &mut Admission<'_, C>,
) -> WorkResult<bool> {
    let mut matched = 0;
    let mut mismatch = false;
    for character in text.chars().chain(std::iter::once(' ')) {
        a.charge(1)?;
        if character.is_alphanumeric() {
            if mismatch {
                continue;
            }
            for lowered in character.to_lowercase() {
                a.charge(1)?;
                let mut buffer = [0; 4];
                let bytes = lowered.encode_utf8(&mut buffer).as_bytes();
                let end = work::add(matched, bytes.len(), a)?;
                a.charge(1 + work::byte_work(token.len() + bytes.len()))?;
                if token.as_bytes().get(matched..end) != Some(bytes) {
                    mismatch = true;
                    break;
                }
                matched = end;
            }
        } else {
            if !mismatch && matched == token.len() {
                return Ok(true);
            }
            matched = 0;
            mismatch = false;
        }
    }
    Ok(false)
}

/// Fixed search metadata points into the complete borrowed native fact roster.
struct Hit {
    /// Actual canonical full tuple ordinal; internal data, never a native proof capability.
    index: usize,
    /// Exact-ID/all-token/some-token bucket; no adjudicated relevance score.
    bucket: usize,
    /// Actual ordered whitelist matches; no private matched text is retained.
    fields: [bool; 4],
}

/// Match actual native/control/label/source-line operands before any rank or page selection.
fn hit<C: WorkControl + ?Sized>(
    scope: &ServerNativeScopeV2<'_, C>,
    row: &Fact<'_>,
    text: &str,
    tokens: &[String],
    a: &mut Admission<'_, C>,
) -> QueryResult<Option<(usize, [bool; 4])>> {
    let source =
        scope.sources().resource(&row.tuple.source_key)?.ok_or(Reason::IncompleteClosure)?;
    let raw = source.bytes()?;
    let span = checked_span(raw, &row.tuple.source_span, a)?;
    a.charge(1 + work::byte_work(span.end_byte - span.start_byte))?;
    let line = std::str::from_utf8(&raw[span.start_byte..span.end_byte])
        .map_err(|_| Reason::SourceSpanUnavailable)?;
    let operands = [row.tuple.native_id.as_str(), row.label, row.tuple.control_id.as_str(), line];
    let mut fields = [false; 4];
    let mut hits = 0;
    for token in tokens {
        let mut any = false;
        for (i, operand) in operands.iter().enumerate() {
            let width = work::add(operand.len(), token.len(), a)?;
            a.charge(1 + work::byte_work(width))?;
            if contains(operand, token, a)? {
                fields[i] = true;
                any = true;
            }
        }
        if any {
            hits = work::add(hits, 1, a)?;
        }
    }
    let width = work::add(text.len(), row.tuple.native_id.len(), a)?;
    let width = work::add(width, row.tuple.control_id.len(), a)?;
    a.charge(1 + work::byte_work(width))?;
    let exact = text == row.tuple.native_id || text == row.tuple.control_id;
    if text == row.tuple.native_id {
        fields[0] = true;
    }
    if text == row.tuple.control_id {
        fields[2] = true;
    }
    Ok((exact || hits != 0).then_some((
        if exact {
            0
        } else if hits == tokens.len() {
            1
        } else {
            2
        },
        fields,
    )))
}

/// Reserve the full possible hit universe before filtering; ordinary refusal discards all partial hits.
fn hits<C: WorkControl + ?Sized>(
    scope: &ServerNativeScopeV2<'_, C>,
    all: &[Fact<'_>],
    text: &str,
    key: Option<&str>,
    tokens: &[String],
    a: &mut Admission<'_, C>,
) -> QueryResult<AdmittedValue<Vec<Hit>>> {
    selection(all, key, text, a)?;
    let slots = work::multiply(all.len(), std::mem::size_of::<Hit>(), a)?;
    let logical = work::add(slots, std::mem::size_of::<Vec<Hit>>(), a)?;
    let mut ordinary = None;
    let hits = a.retain(logical, |a| {
        let mut hits = Vec::with_capacity(all.len());
        for (index, row) in all.iter().enumerate() {
            a.charge(1)?;
            if key.is_some_and(|key| key != row.tuple.artifact_key) {
                continue;
            }
            match hit(scope, row, text, tokens, a) {
                Ok(Some((bucket, fields))) => hits.push(Hit { index, bucket, fields }),
                Ok(None) => (),
                Err(QueryError::Work(error)) => return Err(error),
                Err(QueryError::Unavailable(reason)) => {
                    ordinary = Some(reason);
                    return Ok(Vec::new());
                }
            }
        }
        Ok(hits)
    })?;
    phase(a, ordinary.map_or(Ok(hits), |reason| Err(reason.into())))
}

/// Walk three fixed rank buckets over complete canonical hits, then retain only the lawful page.
fn search<C: WorkControl + ?Sized>(
    scope: &ServerNativeScopeV2<'_, C>,
    all: &[Fact<'_>],
    text: &str,
    key: Option<&str>,
    page: &Page,
    projection: Projection<'_>,
    a: &mut Admission<'_, C>,
) -> QueryResult<AdmittedValue<QueryResponse>> {
    let Projection { query, generation, logical } = projection;
    let tokens = search_tokens(text, a)?;
    let hits = hits(scope, all, text, key, tokens.value(), a)?;
    let matched = hits.value().len();
    let (start, emitted) = page_plan(generation, page, matched, a)?;
    let end = work::add(start, emitted, a)?;
    response(query, logical, a, |a| {
        a.charge(1 + work::byte_work(generation.len() * 2 + 32))?;
        let mut rows = Vec::with_capacity(emitted);
        let mut ordinal = 0;
        for bucket in 0..3 {
            for hit in hits.value() {
                a.charge(1)?;
                if hit.bucket != bucket {
                    continue;
                }
                let rank = work::add(ordinal, 1, a)?;
                if ordinal >= start && ordinal < end {
                    let row = &all[hit.index];
                    let width = tuple_width(row.tuple, a)?;
                    a.charge(1 + work::byte_work(width))?;
                    let labels =
                        ["native-id", "source-label", "control-id", "captured-requirement-text"];
                    let mut fields = Vec::with_capacity(4);
                    for (label, selected) in labels.into_iter().zip(hit.fields) {
                        a.charge(1)?;
                        if selected {
                            fields.push(label);
                        }
                    }
                    rows.push(SearchRow {
                        requirement: requirement(row),
                        rank,
                        matched_fields: fields,
                        match_kind: ["exact-id", "all-tokens", "some-tokens"][bucket],
                    });
                }
                ordinal = rank;
            }
        }
        Ok(Data::Search(page_result(generation.to_owned(), matched, start, rows)))
    })
}

/// Exact query generation has the same /2 framing as App, with actual complete owner generation.
fn generation<C: WorkControl + ?Sized>(
    scope: &ServerNativeScopeV2<'_, C>,
    query: &Query,
) -> QueryResult<AdmittedValue<String>> {
    let mut a = scope.admission();
    let mut failed = false;
    let digest = a.retain(64 + std::mem::size_of::<String>(), |a| {
        let mut writer = GenerationWriter { admission: a, bytes: 0, hash: Sha256::new() };
        let ordinary = (|| {
            writer.write_all(b"forge.mcp-query-generation/2\0")?;
            writer.write_all(scope.scope_generation().as_bytes())?;
            writer.write_all(b"\0")?;
            serde_json::to_writer(&mut writer, &CanonicalQuery::from(query))
                .map_err(io::Error::other)
        })();
        failed = ordinary.is_err();
        let hash = writer.hash.finalize();
        work::fence(a, Stage::PrepareDomain)?;
        Ok(crate::hashing::lower_hex(&hash))
    })?;
    phase(&mut a, if failed { Err(Reason::OutputBoundExceeded.into()) } else { Ok(digest) })
}

/// Constant-size hash state charges each complete serializer chunk before touching it.
struct GenerationWriter<'a, 'control, C: WorkControl + ?Sized> {
    /// The genuine original admission; never an independently renewed allowance.
    admission: &'a mut Admission<'control, C>,
    /// Exact complete byte extent, including domain and actual owner generation.
    bytes: usize,
    /// Plain digest state; its resulting string is data, not currentness proof.
    hash: Sha256,
}
impl<C: WorkControl + ?Sized> Write for GenerationWriter<'_, '_, C> {
    /// Charge actual complete chunks before bounded canonical hash updates.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let next = self
            .bytes
            .checked_add(bytes.len())
            .filter(|n| *n <= MAX_ARGUMENTS)
            .ok_or_else(|| io::Error::other("complete-query-bound"))?;
        self.admission
            .charge(1 + work::byte_work(bytes.len()))
            .map_err(|_| io::Error::other("original-owner-stop"))?;
        self.hash.update(bytes);
        self.bytes = next;
        Ok(bytes.len())
    }
    /// No output bytes are buffered or published by query hashing.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
