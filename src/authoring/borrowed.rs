//! Complete plain authoring facts from supplied borrowed native originals.
//!
//! No IO, paths-as-authority, currentness, capture lease or approval constructor exists.
//! The genuine future factory must bind every actual role/route/original and reject
//! physical aliases. This evaluator derives native provenance only; review-only stored
//! plan, locator and policy originals are intentionally not inputs to the native plan.

use super::admitted::Admission;
use super::manifest::{
    self, AuthorProject, AuthoringPack, MAX_CLAUSE_BYTES, MAX_MANIFEST_BYTES, MAX_TOTAL_BYTES,
};
use super::model::{AuthoringPlan, InputFingerprint, LoadedAuthorProject, LoadedClause};
use crate::applicability::legacy_borrowed::{self as applicability, LegacyApplicabilityInputs};
use crate::applicability::model::ReportFilters;
use crate::hashing::sha256_hex;
use crate::workspace::preparation::WorkControl;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

/// Existing monotonic admission vocabulary, shared with the full native App evaluator.
pub(crate) use crate::applicability::legacy_borrowed::ApplicabilityCharge as AuthoringCharge;
/// Preserve complete native diagnostics and exact original callback/cooperative failures.
pub(crate) use crate::applicability::legacy_borrowed::LegacyBorrowedError as BorrowedAuthoringError;

/// Plain complete native bytes; fields confer no role, physical alias or ownership proof.
#[derive(Clone, Copy)]
pub(crate) struct BorrowedAuthoringInputs<'a> {
    /// Actual author manifest filename relative to its containing project root.
    /// Must be one canonical portable filename, not an arbitrary route or identity assertion.
    pub(crate) project_filename: &'a Path,
    pub(crate) project: &'a [u8],
    pub(crate) pack: &'a [u8],
    pub(crate) gap_report: &'a [u8],
    pub(crate) applicability_manifest: &'a [u8],
    pub(crate) framework: &'a [u8],
    /// The exact explicit Profile Catalog original, absent for Catalog declarations.
    pub(crate) resolved_catalog: Option<&'a [u8]>,
    /// Full original App declaration order; never a selected native subset.
    pub(crate) mappings: &'a [&'a [u8]],
    /// Full original author-project `human_clauses` declaration order, including blocked clauses.
    pub(crate) clauses: &'a [&'a [u8]],
}

/// Genuine native data only; public/plain facts are never an opaque source-current capability.
pub(crate) struct PreparedAuthoringFacts {
    pub(crate) loaded: LoadedAuthorProject,
    pub(crate) plan: AuthoringPlan,
}

/// Discover exact native project declarations under the same actual caller work owner.
/// No IO routes or physical proof are issued by the returned ordinary typed fields.
pub(crate) fn discover_project<E>(
    bytes: &[u8],
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(AuthoringCharge) -> Result<(), E>,
) -> Result<AuthorProject, BorrowedAuthoringError<E>> {
    let mut admission = Admission { control, admit, _error: std::marker::PhantomData };
    admission.step()?;
    let result = decode_project(bytes, &mut admission);
    admission.finish(result)
}

/// Prepare the complete legacy native baseline and whole plan without reopening any path.
/// Every ordinary Domain result is fenced; first Admission/Work/Capacity bypasses later probes.
pub(crate) fn prepare<E>(
    inputs: BorrowedAuthoringInputs<'_>,
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(AuthoringCharge) -> Result<(), E>,
) -> Result<PreparedAuthoringFacts, BorrowedAuthoringError<E>> {
    let mut admission = Admission { control, admit, _error: std::marker::PhantomData };
    admission.step()?;
    let result = prepare_inner(inputs, &mut admission);
    admission.finish(result)
}

/// Decode and validate in native chronology, admitting regex/typed phases before growth.
fn decode_project<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    bytes: &[u8],
    a: &mut Admission<'_, E, F>,
) -> Result<AuthorProject, BorrowedAuthoringError<E>> {
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        a.reserve(256)?;
        return a.native(|| manifest::parse_closed(bytes, "author project"));
    }
    a.raw(bytes, 3)?;
    let project: AuthorProject = a.native(|| manifest::parse_closed(bytes, "author project"))?;
    admit_project(&project, a)?;
    a.native(|| manifest::validate_project(&project))?;
    Ok(project)
}

/// Preserve the native pack decoder while admitting its actual dynamic regex operands first.
fn decode_pack<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    bytes: &[u8],
    a: &mut Admission<'_, E, F>,
) -> Result<AuthoringPack, BorrowedAuthoringError<E>> {
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        a.reserve(256)?;
        return a.native(|| manifest::parse_closed(bytes, "authoring pack"));
    }
    a.raw(bytes, 3)?;
    let pack: AuthoringPack = a.native(|| manifest::parse_closed(bytes, "authoring pack"))?;
    admit_pack(&pack, a)?;
    a.native(|| manifest::validate_pack(&pack))?;
    Ok(pack)
}

/// Native JSON admits its entire lexical/parse tree; its ordinary syntax error remains native.
fn strict<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    bytes: &[u8],
    label: &str,
    a: &mut Admission<'_, E, F>,
) -> Result<Value, BorrowedAuthoringError<E>> {
    a.raw(bytes, 2)?;
    a.native(|| {
        crate::json_strict::parse_value(
            bytes,
            label,
            crate::json_strict::Limits { max_depth: 128, max_string_bytes: bytes.len() },
        )
        .map_err(|cause| super::error(cause.to_string()))
    })
}

/// Derive the full legacy native fingerprint roster from declarations, never a caller list.
fn prepare_inner<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    inputs: BorrowedAuthoringInputs<'_>,
    a: &mut Admission<'_, E, F>,
) -> Result<PreparedAuthoringFacts, BorrowedAuthoringError<E>> {
    let mut fingerprints = Vec::new();
    let mut total = 0usize;
    let headers = prepare_headers(inputs, &mut fingerprints, &mut total, a)?;
    let base = register_native_roster(inputs, &headers.project, &mut fingerprints, &mut total, a)?;
    let baseline_report = prepare_baseline(inputs, &headers, base, a)?;
    let clauses = prepare_clauses(inputs, &headers.project, &mut fingerprints, &mut total, a)?;
    let roster_nodes = a.mul(fingerprints.len(), fingerprints.len())?;
    let mut roster_text = 0usize;
    for row in &fingerprints {
        let text = a.add(row.role.len(), row.path.len())?;
        roster_text = a.add(roster_text, text)?;
    }
    let sort_work = a.mul(roster_text, fingerprints.len())?;
    a.work(fingerprints.len(), sort_work, roster_nodes)?;
    fingerprints
        .sort_unstable_by(|left, right| (&left.role, &left.path).cmp(&(&right.role, &right.path)));
    let loaded = LoadedAuthorProject {
        project: headers.project,
        pack: headers.pack,
        baseline_report,
        inputs: fingerprints,
        project_sha256: headers.project_hash,
        pack_sha256: headers.pack_hash,
        report_sha256: headers.report_hash,
        clauses,
    };
    let plan = super::plan::build_plan_admitted_inner(&loaded, a)?;
    Ok(PreparedAuthoringFacts { loaded, plan })
}

/// Plain aggregate of existing parsed header locals; no extra heap or native capability.
struct NativeHeaders {
    project: AuthorProject,
    pack: AuthoringPack,
    project_hash: String,
    pack_hash: String,
    report_hash: String,
    supplied_report: Value,
}
/// Group existing borrowed registration operands without changing their evaluation order.
#[derive(Clone, Copy)]
struct OriginalObservation<'a> {
    role: &'a str,
    path: &'a Path,
    bytes: &'a [u8],
    expected: Option<&'a str>,
    limit: u64,
}
/// Preserve project/pack/gap/manifest registration and full header decoding in native order.
fn prepare_headers<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    inputs: BorrowedAuthoringInputs<'_>,
    fingerprints: &mut Vec<InputFingerprint>,
    total: &mut usize,
    a: &mut Admission<'_, E, F>,
) -> Result<NativeHeaders, BorrowedAuthoringError<E>> {
    let project_hash = register(
        OriginalObservation {
            role: "author-project",
            path: inputs.project_filename,
            bytes: inputs.project,
            expected: None,
            limit: MAX_MANIFEST_BYTES,
        },
        fingerprints,
        total,
        a,
    )?;
    if inputs.project_filename.components().count() != 1 {
        return domain("borrowed author manifest must name one actual containing-root filename");
    }
    let project = decode_project(inputs.project, a)?;
    let pack_hash = register(
        OriginalObservation {
            role: "authoring-pack",
            path: &project.authoring_pack.path,
            bytes: inputs.pack,
            expected: Some(&project.authoring_pack.expected_sha256),
            limit: MAX_MANIFEST_BYTES,
        },
        fingerprints,
        total,
        a,
    )?;
    let pack = decode_pack(inputs.pack, a)?;
    let report_hash = register(
        OriginalObservation {
            role: "gap-report",
            path: &project.gap_report.path,
            bytes: inputs.gap_report,
            expected: Some(&project.gap_report.expected_sha256),
            limit: crate::io::MAX_FILE_SIZE,
        },
        fingerprints,
        total,
        a,
    )?;
    let supplied_report = strict(inputs.gap_report, "gap report", a)?;
    register(
        OriginalObservation {
            role: "applicability-manifest",
            path: &project.applicability_manifest.path,
            bytes: inputs.applicability_manifest,
            expected: Some(&project.applicability_manifest.expected_sha256),
            limit: crate::applicability::manifest::MAX_MANIFEST_BYTES,
        },
        fingerprints,
        total,
        a,
    )?;
    Ok(NativeHeaders { project, pack, project_hash, pack_hash, report_hash, supplied_report })
}

/// Preserve complete framework/companion/Mapping registration and exact native base resolution.
fn register_native_roster<'project, E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    inputs: BorrowedAuthoringInputs<'_>,
    project: &'project AuthorProject,
    fingerprints: &mut Vec<InputFingerprint>,
    total: &mut usize,
    a: &mut Admission<'_, E, F>,
) -> Result<&'project Path, BorrowedAuthoringError<E>> {
    // Complete potential native Domain diagnostic plus its legacy prefix are
    // admitted before the actual native phase; no error formatting buffer is hidden.
    diagnostic(inputs.applicability_manifest.len(), a)?;
    let discovery =
        applicability::discover_manifest(inputs.applicability_manifest, a.control, a.admit);
    let discovery = discovery.map_err(|error| app_error(error, "applicability manifest"));
    let declared = a.finish(discovery)?;
    let base = project.applicability_manifest.path.parent().unwrap_or_else(|| Path::new(""));
    admit_route(base, &declared.framework.artifact, a)?;
    let framework_path =
        a.native(|| super::input::contained_dependency(base, &declared.framework.artifact))?;
    register(
        OriginalObservation {
            role: "framework",
            path: &framework_path,
            bytes: inputs.framework,
            expected: Some(&project.baseline.framework_sha256),
            limit: crate::io::MAX_FILE_SIZE,
        },
        fingerprints,
        total,
        a,
    )?;
    strict(inputs.framework, "framework", a)?;
    match (&declared.framework.resolved_catalog, inputs.resolved_catalog) {
        (Some(path), Some(bytes)) => {
            let expected =
                project.baseline.resolved_catalog_sha256.as_deref().ok_or_else(|| {
                    BorrowedAuthoringError::Domain(super::error(
                        "Profile baseline requires the resolved Catalog fingerprint",
                    ))
                })?;
            admit_route(base, path, a)?;
            let path = a.native(|| super::input::contained_dependency(base, path))?;
            register(
                OriginalObservation {
                    role: "resolved-catalog",
                    path: &path,
                    bytes,
                    expected: Some(expected),
                    limit: crate::io::MAX_FILE_SIZE,
                },
                fingerprints,
                total,
                a,
            )?;
            strict(bytes, "resolved Catalog", a)?;
        }
        (None, None) if project.baseline.resolved_catalog_sha256.is_none() => {}
        (None, None) => {
            return domain("Catalog baseline must not declare a resolved Catalog fingerprint");
        }
        _ => {
            return domain("borrowed resolved Catalog does not cover the exact declared companion");
        }
    }
    if declared.mapping_collections.len() != inputs.mappings.len() {
        return domain(
            "borrowed Mapping originals do not cover the complete declared manifest roster",
        );
    }
    for (index, (path, bytes)) in
        declared.mapping_collections.iter().zip(inputs.mappings).enumerate()
    {
        admit_route(base, path, a)?;
        let path = a.native(|| super::input::contained_dependency(base, path))?;
        a.reserve(64)?;
        let role = format!("mapping-collection-{index}");
        register(
            OriginalObservation {
                role: &role,
                path: &path,
                bytes,
                expected: None,
                limit: crate::io::MAX_FILE_SIZE,
            },
            fingerprints,
            total,
            a,
        )?;
        strict(bytes, "Mapping Collection", a)?;
    }
    Ok(base)
}

/// Preserve whole native App regeneration, full private equality and cross-contract validation.
fn prepare_baseline<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    inputs: BorrowedAuthoringInputs<'_>,
    headers: &NativeHeaders,
    base: &Path,
    a: &mut Admission<'_, E, F>,
) -> Result<crate::applicability::model::ApplicabilityReport, BorrowedAuthoringError<E>> {
    let mut diagnostic_extent =
        a.add(inputs.applicability_manifest.len(), inputs.framework.len())?;
    if let Some(bytes) = inputs.resolved_catalog {
        diagnostic_extent = a.add(diagnostic_extent, bytes.len())?;
    }
    for bytes in inputs.mappings {
        a.step()?;
        diagnostic_extent = a.add(diagnostic_extent, bytes.len())?;
    }
    diagnostic(diagnostic_extent, a)?;
    let native = applicability::prepare(
        LegacyApplicabilityInputs {
            manifest_bytes: inputs.applicability_manifest,
            manifest_dir: base,
            framework_bytes: inputs.framework,
            resolved_catalog_bytes: inputs.resolved_catalog,
            mapping_bytes: inputs.mappings,
        },
        ReportFilters::default(),
        a.control,
        a.admit,
    );
    let native = native.map_err(|error| app_error(error, "baseline analysis"));
    let native = a.finish(native)?;
    let metric = a.measure(&native.report)?;
    a.materialize(metric, 1, 2)?;
    let regenerated = a.native(|| {
        serde_json::to_value(&native.report)
            .map_err(|cause| super::error(format!("cannot serialize baseline: {cause}")))
    })?;
    a.step()?;
    // Both exact operands are bounded before complete structural traversal.
    let supplied = a.measure(&headers.supplied_report)?;
    let combined_nodes = a.add(metric.nodes, supplied.nodes)?;
    let combined_bytes = a.add(metric.encoded, supplied.encoded)?;
    a.work(combined_nodes, combined_bytes, combined_nodes)?;
    let same = regenerated == headers.supplied_report;
    a.step()?;
    if !same {
        return domain(
            "gap report does not exactly represent the complete current unfiltered PRD-056 analysis",
        );
    }
    let baseline = &native.report;
    a.work(8, 1024, 8)?;
    if headers.pack.baseline != headers.project.baseline
        || headers.project.baseline.report_sha256 != headers.report_hash
        || headers.project.baseline.framework_sha256 != baseline.framework.raw_sha256
        || headers.project.baseline.resolved_catalog_sha256
            != baseline.framework.resolved_catalog_sha256
    {
        return domain(
            "pack, project, framework and report baseline fingerprints must match exactly",
        );
    }
    admit_relationships(&headers.pack, &headers.project, baseline, a)?;
    a.native(|| manifest::validate_relationships(&headers.pack, &headers.project, baseline))?;
    Ok(native.report)
}

/// Validate and retain every actual declared clause, including context-blocked clauses.
fn prepare_clauses<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    inputs: BorrowedAuthoringInputs<'_>,
    project: &AuthorProject,
    fingerprints: &mut Vec<InputFingerprint>,
    total: &mut usize,
    a: &mut Admission<'_, E, F>,
) -> Result<BTreeMap<String, LoadedClause>, BorrowedAuthoringError<E>> {
    if inputs.clauses.len() != project.human_clauses.len() {
        return domain(
            "borrowed clause originals do not cover the complete declared project roster",
        );
    }
    let mut clauses = BTreeMap::new();
    for (clause, bytes) in project.human_clauses.iter().zip(inputs.clauses) {
        a.step()?;
        let role_extent = a.add(clause.key.len(), 32)?;
        a.reserve(role_extent)?;
        let role = format!("human-clause-{}", clause.key);
        register(
            OriginalObservation {
                role: &role,
                path: &clause.source.path,
                bytes,
                expected: Some(&clause.source.expected_sha256),
                limit: MAX_CLAUSE_BYTES,
            },
            fingerprints,
            total,
            a,
        )?;
        // UTF8, nonblank, reserved-template scan, line scan, control-char scan and
        // Markdown parser are six complete passes before actual native validation.
        // A complete character-pair envelope also admits bounded link/bracket matching.
        let work = a.mul(bytes.len(), 6)?;
        let matching = a.mul(bytes.len(), bytes.len())?;
        a.work(1, work, matching)?;
        let parser_headers = a.mul(bytes.len(), 128)?;
        a.reserve(parser_headers)?;
        a.native(|| super::render::validate_clause(bytes))?;
        let metric = a.measure(clause)?;
        let payload = a.add(metric.logical, bytes.len())?;
        let payload = a.add(payload, 256)?;
        a.reserve(payload)?;
        let compared = a.mul(clause.key.len(), project.human_clauses.len())?;
        a.work(1, compared, project.human_clauses.len())?;
        clauses.insert(
            clause.key.clone(),
            LoadedClause { source: clause.clone(), bytes: bytes.to_vec() },
        );
    }
    Ok(clauses)
}

/// Reserve the complete source-bounded native diagnostic and legacy prefix before formatting.
fn diagnostic<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    extent: usize,
    a: &mut Admission<'_, E, F>,
) -> Result<(), BorrowedAuthoringError<E>> {
    let payload = a.mul(extent, 2)?;
    let payload = a.add(payload, 512)?;
    a.reserve(payload)?;
    a.work(1, payload, 0)
}

/// Preserve actual private native error prose while keeping callback/cooperative errors exact.
fn app_error<E>(error: BorrowedAuthoringError<E>, prefix: &str) -> BorrowedAuthoringError<E> {
    match error {
        BorrowedAuthoringError::Domain(cause) => {
            BorrowedAuthoringError::Domain(super::error(format!("{prefix}: {cause}")))
        }
        other => other,
    }
}
/// Produce ordinary native-domain data refusal, never a sticky capacity/currentness label.
fn domain<T, E>(message: &str) -> Result<T, BorrowedAuthoringError<E>> {
    Err(BorrowedAuthoringError::Domain(super::error(message)))
}
/// Admit retained normalized route labels and borrowed split vectors before path normalization.
fn admit_route<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    base: &Path,
    path: &Path,
    a: &mut Admission<'_, E, F>,
) -> Result<(), BorrowedAuthoringError<E>> {
    let extent = a.add(base.as_os_str().len(), path.as_os_str().len())?;
    let payload = a.mul(extent, 32)?;
    let payload = a.add(payload, 256)?;
    a.reserve(payload)?;
    let work = a.mul(extent, 8)?;
    a.work(1, work, extent)
}
/// Admit a complete source observation before hashing, cloning or installing its fingerprint.
/// The derived path roster rejects case-folded aliases; physical aliases remain factory-owned.
fn register<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    input: OriginalObservation<'_>,
    roster: &mut Vec<InputFingerprint>,
    total: &mut usize,
    a: &mut Admission<'_, E, F>,
) -> Result<String, BorrowedAuthoringError<E>> {
    let OriginalObservation { role, path, bytes, expected, limit } = input;
    a.step()?;
    let path_extent = path.as_os_str().len();
    let reserve = a.add(path_extent, role.len())?;
    let reserve = a.add(reserve, 384)?;
    a.reserve(reserve)?;
    let path_work = a.mul(path_extent, 8)?;
    a.work(1, path_work, path_extent)?;
    let label = a.native(|| super::input::portable_label(path))?;
    for prior in &*roster {
        let compared = a.add(prior.path.len(), label.len())?;
        a.work(1, compared, 1)?;
        if prior.path.eq_ignore_ascii_case(&label) {
            return Err(BorrowedAuthoringError::Domain(super::error(format!(
                "{role} aliases another input"
            ))));
        }
    }
    if bytes.len() as u64 > limit {
        return Err(BorrowedAuthoringError::Domain(super::error(format!(
            "{role} exceeds its native input byte limit"
        ))));
    }
    *total = a.add(*total, bytes.len())?;
    if *total as u64 > MAX_TOTAL_BYTES {
        return domain("captured project inputs exceed the 50 MiB total limit");
    }
    a.work(0, bytes.len(), 0)?;
    let digest = sha256_hex(bytes);
    a.step()?;
    if let Some(expected) = expected {
        if !a.equal(expected, &digest)? {
            return Err(BorrowedAuthoringError::Domain(super::error(format!(
                "{role} SHA-256 does not match its exact byte pin"
            ))));
        }
    }
    roster.push(InputFingerprint {
        role: role.to_owned(),
        path: label,
        sha256: digest.clone(),
        byte_length: bytes.len() as u64,
    });
    Ok(digest)
}

/// Admit full native pack validation, including compiled regex states and enum comparisons.
pub(super) fn admit_pack<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    pack: &AuthoringPack,
    a: &mut Admission<'_, E, F>,
) -> Result<(), BorrowedAuthoringError<E>> {
    let metric = a.measure(pack)?;
    a.materialize(metric, 1, 2)?;
    let mut slots = a.add(pack.topics.len(), pack.policy_families.len())?;
    slots = a.add(slots, pack.questions.len())?;
    slots = a.add(slots, pack.reviewers.len())?;
    slots = a.add(slots, pack.control_assignments.len())?;
    slots = a.add(slots, pack.family_assignments.len())?;
    for topic in &pack.topics {
        a.step()?;
        slots = a.add(slots, topic.question_keys.len())?;
    }
    super::plan::admitted::registry(metric, slots, 6, a)?;
    for question in &pack.questions {
        a.step()?;
        if let Some(pattern) = &question.constraints.regex {
            let values = a.measure(&question.constraints.allowed_values)?;
            super::plan::admitted::regex(pattern, values.text, 1, a)?;
        }
        for value in &question.constraints.allowed_values {
            let metric = a.measure(value)?;
            let encoded = a.mul(metric.encoded, 2)?;
            a.reserve(encoded)?;
            let visits = a.mul(metric.nodes, question.constraints.allowed_values.len())?;
            let bytes = a.mul(metric.encoded, question.constraints.allowed_values.len())?;
            a.work(visits, bytes, visits)?;
        }
    }
    Ok(())
}
/// Admit all native project key/path/review/date registries before validation.
fn admit_project<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    project: &AuthorProject,
    a: &mut Admission<'_, E, F>,
) -> Result<(), BorrowedAuthoringError<E>> {
    let metric = a.measure(project)?;
    a.materialize(metric, 1, 2)?;
    let mut slots = a.add(project.policies.len(), project.answers.len())?;
    slots = a.add(slots, project.deferrals.len())?;
    slots = a.add(slots, project.human_clauses.len())?;
    slots = a.add(slots, project.reviewers.len())?;
    slots = a.add(slots, 3)?;
    for clause in &project.human_clauses {
        a.step()?;
        slots = a.add(slots, clause.answer_refs.len())?;
        slots = a.add(slots, clause.gap_ids.len())?;
    }
    super::plan::admitted::registry(metric, slots, 8, a)
}

/// Admit the complete maintained cross-contract validator before registry/hash/regex growth.
pub(super) fn admit_relationships<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    pack: &AuthoringPack,
    project: &AuthorProject,
    report: &crate::applicability::model::ApplicabilityReport,
    a: &mut Admission<'_, E, F>,
) -> Result<(), BorrowedAuthoringError<E>> {
    admit_pack(pack, a)?;
    let metric = a.measure(&project.baseline_review)?;
    a.materialize(metric, 1, 1)?;
    admit_project(project, a)?;
    let metric = a.measure(project)?;
    let mut slots = a.add(project.policies.len(), project.answers.len())?;
    slots = a.add(slots, project.deferrals.len())?;
    slots = a.add(slots, project.human_clauses.len())?;
    slots = a.add(slots, project.reviewers.len())?;
    for clause in &project.human_clauses {
        a.step()?;
        slots = a.add(slots, clause.answer_refs.len())?;
        slots = a.add(slots, clause.gap_ids.len())?;
    }
    super::plan::admitted::registry(metric, slots, 8, a)?;
    let mut report_metric = a.measure(report)?;
    a.materialize(report_metric, 1, 2)?;
    let gap_key_text = a.mul(report.controls.len(), 64)?;
    report_metric.text = a.add(report_metric.text, gap_key_text)?;
    let relations = a.add(slots, pack.topics.len())?;
    let relations = a.add(relations, pack.questions.len())?;
    let relations = a.add(relations, pack.control_assignments.len())?;
    let relations = a.add(relations, pack.family_assignments.len())?;
    let relations = a.add(relations, report.controls.len())?;
    super::plan::admitted::registry(report_metric, relations, 8, a)?;
    for control in &report.controls {
        a.step()?;
        let extent = a.add(project.baseline.report_sha256.len(), control.control_id.len())?;
        let payload = a.add(extent, 512)?;
        a.reserve(payload)?;
        let work_extent = a.add(extent, 128)?;
        let work = a.mul(work_extent, 3)?;
        a.work(1, work, 1)?;
    }
    for answer in &project.answers {
        let metric = a.measure(answer)?;
        // Canonical Value, object-sorting registry and framed/hash buffers cover actual full answer.
        super::plan::admitted::canonical(metric, a)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "borrowed_tests.rs"]
mod tests;

/// Plain review-only native route and full-plan equality utilities; no owner is issued.
#[path = "borrowed_review.rs"]
mod review;
pub(crate) use review::{compare_review_stored_plan, resolve_review_dependency, review_root_text};
