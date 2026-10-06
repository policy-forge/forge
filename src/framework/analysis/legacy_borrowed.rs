//! Plain complete legacy Impact facts from explicit borrowed input bytes.
//!
//! No IO, staging, fingerprint reopens, lease objects or proof constructors live
//! here. Paths describe declarations and preserve private report hrefs; they do
//! not select filesystem originals. This primitive cannot establish capture,
//! physical-alias, same-owner, currentness, complete cohort or union authority.
//! The stored current-report oracle and genuine S4 comparison issuer remain absent.
//!
//! The caller supplies one actual admission callback and `WorkControl` throughout.
//! Checked conservative descriptors request admission before native phases and
//! downstream collection growth; accepting them confers no evidence authority.
//! Logical byte/work ceilings are conservative profile accounting, not a total
//! heap, exact timing, schema complexity or synchronous preemption theorem.

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;

use crate::ForgeError;
use crate::applicability::legacy_borrowed::{
    self as applicability, ApplicabilityCharge, LegacyApplicabilityInputs,
};
use crate::mapping::inventory::{self, CapturedInventoryError, LoadedResource};
use crate::mapping::manifest::SubjectType;
use crate::workspace::preparation::{ProgressUpdate, Stage, WorkControl, WorkError};

use super::super::manifest::{FrameworkResource, ImpactManifest};
use super::super::model::{ChangeClass, ImpactFilters, ImpactReport, ReasonCode};
use super::{ApplicabilityFactsRef, MappingPortfolio};

/// Conservative per supplied original data-profile bound, without capture authority.
const MAX_ORIGINAL_BYTES: usize = 10 * 1024 * 1024;
/// Conservative complete supplied-byte profile; actual physical union remains absent.
const MAX_BORROWED_BYTES: usize = 50 * 1024 * 1024;
/// Conservative supplied occurrence count; actual attempted registration accounting is separate.
const MAX_BORROWED_ORIGINALS: usize = 100;

/// Artifact and optional explicitly declared Profile companion bytes, without provenance.
#[derive(Clone, Copy)]
pub(crate) struct FrameworkBytes<'a> {
    /// Complete artifact bytes supplied by the caller; no copying or path lookup.
    pub(crate) artifact: &'a [u8],
    /// Complete explicit resolved Catalog bytes, if the manifest declares one.
    pub(crate) resolved_catalog: Option<&'a [u8]>,
}

/// Explicit optional applicability originals using that manifest's own declared base.
#[derive(Clone, Copy)]
pub(crate) struct ApplicabilityBytes<'a> {
    /// Complete applicability manifest bytes.
    pub(crate) manifest: &'a [u8],
    /// Complete framework artifact declared by that applicability manifest.
    pub(crate) framework: &'a [u8],
    /// Complete explicit Profile companion, if configured.
    pub(crate) resolved_catalog: Option<&'a [u8]>,
    /// Complete Mapping originals in applicability manifest order.
    pub(crate) mappings: &'a [&'a [u8]],
}

/// Complete borrowed data roster for maintained Impact semantics, without owner evidence.
#[derive(Clone, Copy)]
pub(crate) struct ImpactInputs<'a> {
    /// Declared base retained for exact private basename href/path semantics only.
    pub(crate) manifest_dir: &'a Path,
    /// Complete duplicate-key-safe Impact manifest input.
    pub(crate) manifest: &'a [u8],
    /// Complete declared old framework artifact and explicit companion.
    pub(crate) old: FrameworkBytes<'a>,
    /// Complete declared new framework artifact and explicit companion.
    pub(crate) new: FrameworkBytes<'a>,
    /// Complete declared Mapping originals in Impact manifest order.
    pub(crate) mappings: &'a [&'a [u8]],
    /// Configured legacy applicability originals; policy hrefs remain embedded data.
    pub(crate) applicability: Option<ApplicabilityBytes<'a>>,
    /// Complete declared successor-map bytes, if configured.
    pub(crate) successor_map: Option<&'a [u8]>,
    /// Complete declared prior-report bytes, paired with dispositions.
    pub(crate) prior_report: Option<&'a [u8]>,
    /// Complete declared disposition bytes, paired with the prior report.
    pub(crate) dispositions: Option<&'a [u8]>,
}

/// Complete private semantic data; this is not a pending cohort or native approval.
pub(crate) struct LegacyImpactFacts {
    /// Fully validated declaration, preserving exact private resource versions and paths.
    #[cfg(test)]
    pub(crate) manifest: ImpactManifest,
    /// Full maintained report including all private evidence, findings and hidden filter rows.
    pub(crate) report: ImpactReport,
    /// Complete configured declaration retained for native parity controls only.
    #[cfg(test)]
    pub(crate) applicability_manifest:
        Option<crate::applicability::manifest::ApplicabilityManifest>,
}

/// Plain requests to the actual caller's monotonic ledger; no ownership assertion.
pub(crate) enum ImpactCharge {
    /// Conservative logical owned-byte growth requested before allocation.
    Bytes(usize),
    /// Complete row growth requested before collection construction.
    Rows(usize),
    /// Repeated node, string and matching/sorting work, including conservative bounds.
    Work {
        /// Complete admitted node inspection visits, including repeated phases.
        nodes: usize,
        /// Complete admitted private string/canonical/comparison byte work.
        string_bytes: usize,
        /// Complete admitted matching and ordering probes.
        comparisons: usize,
    },
    /// Complete repeated native Mapping reference/facts payload before registry growth.
    MappingProjection {
        /// Whole repeated reference/facts logical payload admitted before native copying.
        logical_bytes: usize,
        /// Every actual role-selected framework Control occurrence, without deduplication credit.
        controls: usize,
    },
    /// Shared maintained borrowed applicability/inventory admission, without conversion.
    Applicability(ApplicabilityCharge),
    /// Checked/profile arithmetic refused; accepting this notification cannot permit success.
    Capacity,
}

/// Preserve complete legacy domain diagnostics and original caller failures privately.
#[derive(Debug)]
pub(crate) enum LegacyImpactError<E> {
    /// Full maintained FrameworkImpact/native detail; public callers may minimize it later.
    Domain(ForgeError),
    /// Original actual admission failure, without replacing or clearing its sticky state.
    Admission(E),
    /// Actual supplied `WorkControl` stop/failure; no local replacement control exists.
    Work(WorkError),
    /// Conservative profile bound refused even if the caller accepted its notification.
    Capacity,
}

/// Discover only maintained Impact declarations from admitted supplied bytes, without IO.
///
/// This shared native decoder lets a genuine captured reader discover the complete
/// declared roster. Its plain result carries no role, ownership, currentness or cohort
/// authority. Full preparation repeats this exact decode under the same caller budget.
///
/// # Errors
///
/// Preserve actual first admission/work failures. Native success and domain refusal
/// both pass the same maintained phase's original post-parse cooperative fence.
pub(crate) fn discover_manifest<E>(
    bytes: &[u8],
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<ImpactManifest, LegacyImpactError<E>> {
    phase(control, admit, |control, admit| {
        preparse(bytes, control, admit)?;
        super::super::manifest::parse(bytes).map_err(LegacyImpactError::Domain)
    })
}

/// Compute complete ordinary legacy facts from supplied bytes through shared native semantics.
///
/// Cardinality and option checks establish data shape only. The caller's absent
/// capture factory must independently establish every declared original, physical
/// alias rule and same-owner complete read cohort. The supplied current stored
/// report is intentionally outside this primitive's data input; its strict complete
/// equality oracle and private comparison sealer remain absent.
///
/// # Errors
///
/// Retains full maintained manifest/resource/Mapping/applicability/successor/prior
/// disposition errors within the conservative profile, original admission failures
/// and actual work-control stops. Every guarded native phase checks `WorkControl`
/// after success and domain failure before returning its native result. A first
/// actual callback admission/work failure propagates without a replacing checkpoint.
#[expect(
    clippy::too_many_lines,
    reason = "One ordered native phase pipeline preserves admission and post-phase first-error fences"
)]
pub(crate) fn prepare<E>(
    inputs: ImpactInputs<'_>,
    filters: ImpactFilters,
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<LegacyImpactFacts, LegacyImpactError<E>> {
    checkpoint(control)?;
    let original_extent = admit_roster(&inputs, control, admit)?;
    let original_extent =
        add(original_extent, mul(inputs.manifest_dir.as_os_str().len(), 4, admit)?, admit)?;
    let parsed = discover_manifest(inputs.manifest, control, admit)?;
    phase(control, admit, |_, admit| {
        let filter_extent = filter_extent(&filters, admit)?;
        work(5, mul(filter_extent, 32, admit)?, 5, admit)?;
        super::validate_filters(&filters).map_err(LegacyImpactError::Domain)?;
        validate_shape(&parsed, &inputs).map_err(LegacyImpactError::Domain)
    })?;
    let old =
        load_framework(inputs.manifest_dir, "$.old", &parsed.old, &inputs.old, control, admit)?;
    let new =
        load_framework(inputs.manifest_dir, "$.new", &parsed.new, &inputs.new, control, admit)?;
    phase(control, admit, |_, admit| {
        let controls = add(
            old.inventory.count(SubjectType::Control),
            new.inventory.count(SubjectType::Control),
            admit,
        )?;
        work(controls, mul(original_extent, 2, admit)?, controls, admit)?;
        if let Some(group_id) = filters.group.as_deref() {
            super::validate_group_ids("$.old", &old.inventory)
                .map_err(LegacyImpactError::Domain)?;
            super::validate_group_ids("$.new", &new.inventory)
                .map_err(LegacyImpactError::Domain)?;
            super::validate_requested_group(group_id, &old.inventory, &new.inventory)
                .map_err(LegacyImpactError::Domain)?;
        }
        Ok(())
    })?;
    let mut portfolio = MappingPortfolio::default();
    let mut collection_ids = BTreeSet::new();
    for (index, (dependency, bytes)) in
        parsed.mapping_collections.iter().zip(inputs.mappings).enumerate()
    {
        phase(control, admit, |control, admit| {
            // Strict parsing, schema/version validation, typed decoding, UUID sets,
            // embedded endpoint/old-subject checks and all reference/fact growth
            // happen only after these complete conservative input/work admissions.
            let slots = preparse(bytes, control, admit)?;
            rows(slots, admit)?;
            work(
                mul(slots, 32, admit)?,
                mul(bytes.len(), 32, admit)?,
                mul(slots, 32, admit)?,
                admit,
            )?;
            reserve(128, admit)?;
            let label = format!("$.mapping_collections[{index}]");
            let raw_sha256 = crate::hashing::sha256_hex(bytes);
            let collection =
                super::decode_mapping_bytes(&label, bytes).map_err(LegacyImpactError::Domain)?;
            admit_mapping_projection(
                &collection,
                dependency.framework_role,
                old.inventory.count(SubjectType::Control),
                portfolio.references.len(),
                control,
                admit,
            )?;
            super::record_mapping_collection(
                &label,
                &collection,
                raw_sha256,
                dependency.framework_role,
                &old,
                &mut collection_ids,
                &mut portfolio,
            )
            .map_err(LegacyImpactError::Domain)
        })?;
    }
    phase(control, admit, |_, admit| {
        let references = reference_count(&portfolio, admit)?;
        work(
            references,
            mul(original_extent, references, admit)?,
            mul(references, references, admit)?,
            admit,
        )?;
        super::sort_mapping_references(&mut portfolio);
        Ok(())
    })?;
    let applicability_facts = if let Some(bytes) = inputs.applicability.as_ref() {
        phase(control, admit, |control, admit| {
            let declaration = parsed.applicability_manifest.as_ref().ok_or_else(|| {
                LegacyImpactError::Domain(super::impact_error(
                    "borrowed applicability bytes have no declaration",
                ))
            })?;
            reserve(
                add(inputs.manifest_dir.as_os_str().len(), declaration.as_os_str().len(), admit)?,
                admit,
            )?;
            let path = inputs.manifest_dir.join(declaration);
            let base = path.parent().unwrap_or(inputs.manifest_dir);
            let facts = applicability::prepare(
                LegacyApplicabilityInputs {
                    manifest_bytes: bytes.manifest,
                    manifest_dir: base,
                    framework_bytes: bytes.framework,
                    resolved_catalog_bytes: bytes.resolved_catalog,
                    mapping_bytes: bytes.mappings,
                },
                crate::applicability::model::ReportFilters::default(),
                control,
                &mut |charge| admit(ImpactCharge::Applicability(charge)),
            )
            .map_err(map_applicability_error)?;
            let controls = facts.report.controls.len();
            // Existing consistency validation clones complete policy-source sets.
            reserve(mul(original_extent, add(controls, 1, admit)?, admit)?, admit)?;
            work(
                controls,
                mul(original_extent, add(controls, 1, admit)?, admit)?,
                controls,
                admit,
            )?;
            super::validate_applicability_facts(
                &ApplicabilityFactsRef { manifest: &facts.manifest, report: &facts.report },
                &old,
                &portfolio,
            )
            .map_err(LegacyImpactError::Domain)?;
            Ok(facts)
        })
        .map(Some)?
    } else {
        None
    };
    let successor_map = inputs
        .successor_map
        .map(|bytes| {
            phase(control, admit, |control, admit| {
                let slots = preparse(bytes, control, admit)?;
                work(
                    mul(slots, 64, admit)?,
                    mul(bytes.len(), 64, admit)?,
                    mul(slots, slots, admit)?,
                    admit,
                )?;
                crate::migration::parse_successor(bytes)
                    .map_err(super::map_migration_error)
                    .map_err(LegacyImpactError::Domain)
            })
        })
        .transpose()?;
    let borrowed_app = applicability_facts
        .as_ref()
        .map(|facts| ApplicabilityFactsRef { manifest: &facts.manifest, report: &facts.report });
    let mut report = phase(control, admit, |control, admit| {
        super::build_report_admitted(
            old,
            new,
            &portfolio,
            borrowed_app.as_ref(),
            successor_map.as_ref(),
            filters,
            &mut |stage| admit_report_growth(stage, control, admit),
        )
        .map_err(|error| match error {
            super::ReportBuildError::Native(error) => LegacyImpactError::Domain(error),
            super::ReportBuildError::Admission(error) => error,
        })
    })?;
    if let (Some(prior), Some(dispositions)) = (inputs.prior_report, inputs.dispositions) {
        phase(control, admit, |control, admit| {
            let prior_slots = preparse(prior, control, admit)?;
            let disposition_slots = preparse(dispositions, control, admit)?;
            let slots = add(prior_slots, disposition_slots, admit)?;
            rows(add(report.findings.len(), disposition_slots, admit)?, admit)?;
            let extent = add(prior.len(), dispositions.len(), admit)?;
            work(
                mul(slots, 32, admit)?,
                mul(extent, 32, admit)?,
                mul(slots, slots, admit)?,
                admit,
            )?;
            // Retain the file wrapper's order: disposition contract first, then
            // raw prior hash, strict prior parse, header/pair, IDs and application.
            let parsed_dispositions = super::super::disposition::parse(dispositions)
                .map_err(LegacyImpactError::Domain)?;
            admit_history_report(
                &report,
                &parsed_dispositions.dispositions,
                prior.len(),
                prior_slots,
                control,
                admit,
            )?;
            super::apply_disposition_records(prior, parsed_dispositions, &mut report)
                .map_err(LegacyImpactError::Domain)
        })?;
    }
    phase(control, admit, |_, admit| {
        let count = report.findings.len();
        reserve(
            mul(count, std::mem::size_of::<super::super::model::ImpactFinding>(), admit)?,
            admit,
        )?;
        rows(count, admit)?;
        work(
            count,
            mul(original_extent, add(count, 1, admit)?, admit)?,
            mul(count, 5, admit)?,
            admit,
        )?;
        super::finish_report(&mut report);
        Ok(())
    })?;
    Ok(LegacyImpactFacts {
        #[cfg(test)]
        manifest: parsed,
        report,
        #[cfg(test)]
        applicability_manifest: applicability_facts.map(|facts| facts.manifest),
    })
}

/// Preserve declared option/cardinality data shape; this is not captured read membership.
fn validate_shape(manifest: &ImpactManifest, inputs: &ImpactInputs<'_>) -> Result<(), ForgeError> {
    for (label, declared, supplied) in [
        (
            "$.old.resolved_catalog",
            manifest.old.resolved_catalog.is_some(),
            inputs.old.resolved_catalog.is_some(),
        ),
        (
            "$.new.resolved_catalog",
            manifest.new.resolved_catalog.is_some(),
            inputs.new.resolved_catalog.is_some(),
        ),
        (
            "$.applicability_manifest",
            manifest.applicability_manifest.is_some(),
            inputs.applicability.is_some(),
        ),
        ("$.successor_map", manifest.successor_map.is_some(), inputs.successor_map.is_some()),
        ("$.prior_report", manifest.prior_report.is_some(), inputs.prior_report.is_some()),
        ("$.disposition_file", manifest.disposition_file.is_some(), inputs.dispositions.is_some()),
    ] {
        if declared != supplied {
            return Err(super::impact_error(format!(
                "{label} borrowed bytes do not match the declaration"
            )));
        }
    }
    if manifest.mapping_collections.len() != inputs.mappings.len() {
        return Err(super::impact_error(
            "borrowed Mapping originals do not cover the complete declared Impact roster",
        ));
    }
    Ok(())
}

/// Load genuine native inventory semantics from bytes, preserving the exact full evidence tuple.
fn load_framework<E>(
    base: &Path,
    label: &str,
    resource: &FrameworkResource,
    bytes: &FrameworkBytes<'_>,
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<LoadedResource, LegacyImpactError<E>> {
    phase(control, admit, |control, admit| {
        preparse(bytes.artifact, control, admit)?;
        if let Some(companion) = bytes.resolved_catalog {
            preparse(companion, control, admit)?;
        }
        let path_bytes = add(base.as_os_str().len(), resource.artifact.as_os_str().len(), admit)?;
        let companion_path_bytes =
            resource.resolved_catalog.as_ref().map_or(0, |path| path.as_os_str().len());
        let descriptor_extent = add(add(path_bytes, companion_path_bytes, admit)?, 1024, admit)?;
        reserve(mul(descriptor_extent, 4, admit)?, admit)?;
        work(8, mul(descriptor_extent, 4, admit)?, 8, admit)?;
        let path = base.join(&resource.artifact);
        let descriptor = super::framework_descriptor(&path, resource);
        let result = inventory::load_captured_admitted(
            base,
            label,
            &descriptor,
            bytes.artifact,
            bytes.resolved_catalog,
            &mut |value| {
                applicability::admit_legacy_inventory(
                    value,
                    &mut |charge| admit(ImpactCharge::Applicability(charge)),
                    control,
                )
                .map_err(map_applicability_error)
            },
        );
        let loaded = match result {
            Ok(loaded) => loaded,
            Err(CapturedInventoryError::Domain(error)) => {
                return Err(LegacyImpactError::Domain(super::map_mapping_error(error)));
            }
            Err(CapturedInventoryError::Admission(error)) => return Err(error),
        };
        super::validate_loaded_framework(label, resource, &loaded)
            .map_err(LegacyImpactError::Domain)?;
        Ok(loaded)
    })
}

/// Admit complete actual framework-side Control multiplicity before native policy strings clone.
///
/// Private policy hrefs/root-UUID properties are borrowed data only. Duplicated
/// candidate properties/items are conservatively counted before legacy validators
/// report their full domain errors. No endpoint read or membership proof is issued.
fn admit_mapping_projection<E>(
    collection: &crate::mapping::model::MappingCollectionEnvelope,
    role: super::super::manifest::FrameworkRole,
    old_controls: usize,
    existing_reference_controls: usize,
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<(), LegacyImpactError<E>> {
    let mut controls = 0usize;
    let mut items = 0usize;
    let mut logical_bytes = 0usize;
    let mut projected_bytes = 0usize;
    work(collection.mapping_collection.mappings.len(), 0, 0, admit)?;
    for mapping in &collection.mapping_collection.mappings {
        checkpoint(control)?;
        let policy = match role {
            super::super::manifest::FrameworkRole::Source => &mapping.target_resource,
            super::super::manifest::FrameworkRole::Target => &mapping.source_resource,
        };
        let mut identity_bytes = 0usize;
        for prop in &policy.props {
            let spelling = add(prop.name.len(), prop.ns.as_deref().map_or(0, str::len), admit)?;
            work(1, spelling, 2, admit)?;
            if prop.name == "root-uuid"
                && prop.ns.as_deref() == Some(crate::mapping::inventory::FORGE_MAPPING_NS)
            {
                identity_bytes = add(identity_bytes, prop.value.len(), admit)?;
            }
        }
        // require_forge_prop copies the matching private identity once per mapping.
        logical_bytes = add(logical_bytes, identity_bytes, admit)?;
        projected_bytes = add(projected_bytes, identity_bytes, admit)?;
        work(mapping.maps.len(), 0, 0, admit)?;
        for map in &mapping.maps {
            checkpoint(control)?;
            let framework_items = match role {
                super::super::manifest::FrameworkRole::Source => &map.sources,
                super::super::manifest::FrameworkRole::Target => &map.targets,
            };
            work(framework_items.len(), 0, framework_items.len(), admit)?;
            items = add(items, framework_items.len(), admit)?;
            for item in framework_items {
                if item.subject_type != SubjectType::Control {
                    continue;
                }
                controls = add(controls, 1, admit)?;
                let target_copies =
                    usize::from(role == super::super::manifest::FrameworkRole::Target);
                let id_bytes = mul(item.id_ref.len(), add(1, target_copies, admit)?, admit)?;
                let href_bytes = mul(policy.href.len(), add(1, target_copies, admit)?, admit)?;
                let strings = add(add(id_bytes, href_bytes, admit)?, identity_bytes, admit)?;
                // Fixed logical row allowance covers native UUID strings, String/Vec
                // headers and attempted registry nodes. It is not allocator heap proof.
                logical_bytes = add(logical_bytes, add(strings, 1024, admit)?, admit)?;
                projected_bytes = add(projected_bytes, strings, admit)?;
            }
        }
    }
    // This complete multiplier is admitted before inventory_mapping clones any
    // projected reference/facts strings, including repeated target set attempts.
    admit(ImpactCharge::MappingProjection { logical_bytes, controls })
        .map_err(LegacyImpactError::Admission)?;
    let width = add(
        add(old_controls, existing_reference_controls, admit)?,
        add(controls, 8, admit)?,
        admit,
    )?;
    work(
        add(items, controls, admit)?,
        mul(projected_bytes, add(width, 1, admit)?, admit)?,
        mul(items, width, admit)?,
        admit,
    )
}

/// Charge every supplied original/duplicate occurrence before roster scanning or native decoding.
fn admit_roster<E>(
    inputs: &ImpactInputs<'_>,
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<usize, LegacyImpactError<E>> {
    phase(control, admit, |_, admit| {
        let mut count = 0;
        let mut extent = 0;
        let mut include = |bytes: &[u8]| -> Result<(), LegacyImpactError<E>> {
            count = add(count, 1, admit)?;
            extent = add(extent, bytes.len(), admit)?;
            work(1, bytes.len(), 0, admit)?;
            if count > MAX_BORROWED_ORIGINALS
                || extent > MAX_BORROWED_BYTES
                || bytes.len() > MAX_ORIGINAL_BYTES
            {
                return capacity(admit);
            }
            Ok(())
        };
        include(inputs.manifest)?;
        include(inputs.old.artifact)?;
        include(inputs.new.artifact)?;
        for companion in
            [inputs.old.resolved_catalog, inputs.new.resolved_catalog].into_iter().flatten()
        {
            include(companion)?;
        }
        for bytes in inputs.mappings {
            include(bytes)?;
        }
        if let Some(app) = inputs.applicability.as_ref() {
            include(app.manifest)?;
            include(app.framework)?;
            if let Some(companion) = app.resolved_catalog {
                include(companion)?;
            }
            for bytes in app.mappings {
                include(bytes)?;
            }
        }
        for bytes in
            [inputs.successor_map, inputs.prior_report, inputs.dispositions].into_iter().flatten()
        {
            include(bytes)?;
        }
        Ok(extent)
    })
}

/// Pre-admit complete logical decoded forms before native decoding/schema/typed growth.
/// The scan counts a conservative slot ceiling on valid JSON but admits no syntax.
/// Native duplicate-key, domain/schema and version diagnostics stay downstream.
fn preparse<E>(
    bytes: &[u8],
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<usize, LegacyImpactError<E>> {
    work(0, mul(bytes.len(), 10, admit)?, 0, admit)?;
    let slots = preparse_slots(bytes, control, admit)?;
    let decoded = add(add(mul(slots, 128, admit)?, bytes.len(), admit)?, 512, admit)?;
    reserve(mul(decoded, 4, admit)?, admit)?;
    work(mul(slots, 32, admit)?, mul(bytes.len(), 32, admit)?, mul(slots, 32, admit)?, admit)?;
    Ok(slots)
}

/// Literal bounded lexical scan adapted from the pinned applicability proposal.
/// On valid JSON each Value/key starts at a container, quote or primitive token;
/// raw bytes bound decoded string/key payload. No tree or parser result is retained.
fn preparse_slots<E>(
    bytes: &[u8],
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<usize, LegacyImpactError<E>> {
    let mut slots = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    let mut primitive = false;
    for chunk in bytes.chunks(32 * 1024) {
        checkpoint(control)?;
        for &byte in chunk {
            if quoted {
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    quoted = false;
                }
                continue;
            }
            match byte {
                b'"' => {
                    slots = add(slots, 1, admit)?;
                    quoted = true;
                    primitive = false;
                }
                b'{' | b'[' => {
                    slots = add(slots, 1, admit)?;
                    primitive = false;
                }
                b'}' | b']' | b',' | b':' | b' ' | b'\r' | b'\n' | b'\t' => primitive = false,
                _ if !primitive => {
                    slots = add(slots, 1, admit)?;
                    primitive = true;
                }
                _ => {}
            }
        }
    }
    work(slots, 0, 0, admit)?;
    Ok(slots)
}

/// Fixed logical registry-slot allowance already used by this borrowed profile.
/// This is descriptor accounting, not a claim about allocator or total heap bytes.
const REPORT_REGISTRY_SLOT: usize = 128;
/// An i32 year (at most eleven signed digits), date/time punctuation, nine
/// fractional digits and a full six-byte offset fit 42 UTF-8 bytes. Maintained
/// successor parsing requires UTC; this bound also includes the longer offset.
const REPORT_TIMESTAMP_BYTES: usize = 42;

/// Stack-only arithmetic; no report allocation or evidence authority is retained.
#[derive(Default)]
struct ReportBudget {
    bytes: usize,
    string_bytes: usize,
    visits: usize,
    comparisons: usize,
}

impl ReportBudget {
    /// Count actual or conservative logical owned growth before its constructor.
    fn owned<E>(
        &mut self,
        amount: usize,
        admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
    ) -> Result<(), LegacyImpactError<E>> {
        self.bytes = add(self.bytes, amount, admit)?;
        self.string_bytes = add(self.string_bytes, amount, admit)?;
        Ok(())
    }

    /// Record admitted native lookup/sort/clone work without constructing rows.
    fn probes<E>(
        &mut self,
        count: usize,
        width: usize,
        admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
    ) -> Result<(), LegacyImpactError<E>> {
        self.comparisons = add(self.comparisons, count, admit)?;
        self.string_bytes = add(self.string_bytes, mul(count, width, admit)?, admit)?;
        Ok(())
    }

    /// Admit the entire stage before the shared native builder resumes growth.
    fn emit<E>(
        self,
        admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
    ) -> Result<(), LegacyImpactError<E>> {
        rows(self.visits, admit)?;
        reserve(self.bytes, admit)?;
        work(self.visits, self.string_bytes, self.comparisons, admit)
    }
}

/// Share one actual callback/control through every native report construction stage.
fn admit_report_growth<E>(
    stage: super::ReportGrowth<'_>,
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<(), LegacyImpactError<E>> {
    checkpoint(control)?;
    match stage {
        super::ReportGrowth::Classification { old, new, successor_map } => {
            admit_classification(old, new, successor_map, control, admit)
        }
        super::ReportGrowth::Findings(inputs) => admit_findings(inputs, control, admit),
        super::ReportGrowth::FrameworkGroups { old, new, findings } => {
            admit_groups(old, new, findings, control, admit)
        }
        super::ReportGrowth::FindingSort { changes, findings } => {
            admit_finding_sort(changes.len(), findings, control, admit)
        }
    }
}

/// Measure cloned IDs/hashes, declared migration arrays and native sort storage.
/// All descriptors borrow original native objects; this creates no second classifier.
fn admit_classification<E>(
    old: &inventory::Inventory,
    new: &inventory::Inventory,
    successor_map: Option<&crate::migration::SuccessorMap>,
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<(), LegacyImpactError<E>> {
    let controls = add(old.count(SubjectType::Control), new.count(SubjectType::Control), admit)?;
    let all_subjects = add(
        controls,
        add(old.count(SubjectType::Statement), new.count(SubjectType::Statement), admit)?,
        admit,
    )?;
    work(all_subjects, 0, 0, admit)?;
    let mut budget = ReportBudget::default();
    let mut id_bytes = [0usize; 2];
    let mut hash_bytes = 0usize;
    let mut max_id = 0usize;
    let mut max_hash = 0usize;
    for (side, inventory) in [old, new].into_iter().enumerate() {
        for (kind, id, hash) in inventory.subject_entries_refs() {
            checkpoint(control)?;
            if kind != SubjectType::Control {
                continue;
            }
            id_bytes[side] = add(id_bytes[side], id.len(), admit)?;
            hash_bytes = add(hash_bytes, hash.len(), admit)?;
            max_id = max_id.max(id.len());
            max_hash = max_hash.max(hash.len());
        }
    }
    let ids = add(id_bytes[0], id_bytes[1], admit)?;
    // Old/new ID sets, plus the stable intersection; migrated sets are below.
    budget.owned(add(mul(controls, REPORT_REGISTRY_SLOT, admit)?, ids, admit)?, admit)?;
    let stable = old.count(SubjectType::Control).min(new.count(SubjectType::Control));
    budget.owned(
        add(mul(stable, REPORT_REGISTRY_SLOT, admit)?, id_bytes[0].min(id_bytes[1]), admit)?,
        admit,
    )?;
    // At most old+new change rows; the second row footprint covers stable sort scratch.
    budget.owned(
        mul(
            mul(controls, 2, admit)?,
            std::mem::size_of::<super::super::model::ControlChange>(),
            admit,
        )?,
        admit,
    )?;
    budget.owned(
        mul(controls, std::mem::size_of::<super::super::model::SubjectFingerprint>(), admit)?,
        admit,
    )?;
    // Every ordinary side contributes one fingerprint ID and two hash strings;
    // the union subject IDs add at most one additional complete side-ID payload.
    budget.owned(add(mul(ids, 2, admit)?, mul(hash_bytes, 2, admit)?, admit)?, admit)?;
    budget.owned(
        add(
            std::mem::size_of::<ImpactReport>(),
            super::super::model::REPORT_SCHEMA_VERSION.len(),
            admit,
        )?,
        admit,
    )?;
    let (declared_ids, migration_rows) = admit_declared_migrations(
        successor_map,
        max_hash,
        &mut max_id,
        &mut budget,
        control,
        admit,
    )?;
    // ID/stable-set insertion, union traversal, ordinary hash comparison and
    // change sorting each fit a Control-square bound. Fingerprint maps also hold
    // Statements: retain that complete map extent for ordinary AND migrated IDs.
    // Three checks per declared migration ID cover baseline/stable/migrated sets.
    let square = mul(controls, controls, admit)?;
    let fingerprints =
        mul(add(mul(controls, 2, admit)?, declared_ids, admit)?, all_subjects, admit)?;
    let classification_probes = add(
        add(mul(square, 5, admit)?, fingerprints, admit)?,
        mul(mul(declared_ids, add(controls, declared_ids, admit)?, admit)?, 3, admit)?,
        admit,
    )?;
    budget.probes(
        add(classification_probes, controls, admit)?,
        mul(max_id.max(max_hash), 2, admit)?,
        admit,
    )?;
    budget.visits = add(add(controls, declared_ids, admit)?, migration_rows, admit)?;
    budget.emit(admit)
}

/// Admit declared migration copies before native classification without allocating facts.
fn admit_declared_migrations<E>(
    successor_map: Option<&crate::migration::SuccessorMap>,
    max_hash: usize,
    max_id: &mut usize,
    budget: &mut ReportBudget,
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<(usize, usize), LegacyImpactError<E>> {
    let mut declared_ids = 0usize;
    let mut migration_rows = 0usize;
    if let Some(successor_map) = successor_map {
        work(successor_map.relationships.len(), 0, 0, admit)?;
        for relationship in &successor_map.relationships {
            checkpoint(control)?;
            migration_rows = add(migration_rows, 1, admit)?;
            let count = add(relationship.old_ids.len(), relationship.new_ids.len(), admit)?;
            declared_ids = add(declared_ids, count, admit)?;
            work(count, 0, 0, admit)?;
            let mut names = 0usize;
            for id in relationship.old_ids.iter().chain(&relationship.new_ids) {
                checkpoint(control)?;
                names = add(names, id.len(), admit)?;
                *max_id = (*max_id).max(id.len());
            }
            let commas = add(
                relationship.old_ids.len().saturating_sub(1),
                relationship.new_ids.len().saturating_sub(1),
                admit,
            )?;
            let joined = add(names, commas, admit)?;
            let subject = add(joined, 2, admit)?;
            *max_id = (*max_id).max(subject);
            // Declared migrated registries retain their actual IDs; complete native
            // subject arrays retain all ID/hash copies, including split/merge arrays.
            budget.owned(add(mul(count, REPORT_REGISTRY_SLOT, admit)?, names, admit)?, admit)?;
            budget.owned(
                add(
                    mul(
                        count,
                        std::mem::size_of::<super::super::model::SubjectFingerprint>(),
                        admit,
                    )?,
                    add(names, mul(count, max_hash, admit)?, admit)?,
                    admit,
                )?,
                admit,
            )?;
            budget.owned(
                add(mul(max_hash, 2, admit)?, add(subject, joined, admit)?, admit)?,
                admit,
            )?;
            budget.owned(
                add(
                    add(relationship.approved_by.len(), relationship.rationale.len(), admit)?,
                    REPORT_TIMESTAMP_BYTES,
                    admit,
                )?,
                admit,
            )?;
        }
    }
    Ok((declared_ids, migration_rows))
}

/// Complete heap payload cloned from the genuine classified change into one finding.
fn change_payload<E>(
    change: &super::super::model::ControlChange,
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<usize, LegacyImpactError<E>> {
    work(add(change.old_subjects.len(), change.new_subjects.len(), admit)?, 0, 0, admit)?;
    let mut amount = 0usize;
    for hash in [change.old_sha256.as_deref(), change.new_sha256.as_deref()].into_iter().flatten() {
        amount = add(amount, hash.len(), admit)?;
    }
    for subject in change.old_subjects.iter().chain(&change.new_subjects) {
        checkpoint(control)?;
        amount = add(
            amount,
            add(
                std::mem::size_of::<super::super::model::SubjectFingerprint>(),
                add(subject.id.len(), subject.sha256.len(), admit)?,
                admit,
            )?,
            admit,
        )?;
    }
    if let Some(migration) = &change.migration {
        for value in [&migration.approved_by, &migration.approved_at, &migration.rationale] {
            amount = add(amount, value.len(), admit)?;
        }
    }
    Ok(amount)
}

/// Exact full private migration identity seed and generated hash, without hashing it.
fn migration_scratch<E>(
    change: &super::super::model::ControlChange,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<usize, LegacyImpactError<E>> {
    let Some(migration) = &change.migration else { return Ok(0) };
    let mut amount = add(change.subject_id.len(), migration.relationship.as_str().len(), admit)?;
    for value in [&migration.approved_by, &migration.approved_at, &migration.rationale] {
        amount = add(amount, value.len(), admit)?;
    }
    add(add(amount, 4, admit)?, 64, admit)
}

/// Data-only exact context extents; strings remain borrowed until native allocation.
#[derive(Default)]
struct ContextBudget {
    retained: usize,
    identity: usize,
    temporary: usize,
}

/// Actual borrowed native arguments for one planned finding; no constructed row.
struct FindingMeasure<'a> {
    old: &'a LoadedResource,
    new: &'a LoadedResource,
    change: &'a super::super::model::ControlChange,
    subject: &'a str,
    context: ContextBudget,
}

/// Admit one complete prospective native finding, including its transient identity/UUID seed.
fn finding_budget<E>(
    budget: &mut ReportBudget,
    inputs: FindingMeasure<'_>,
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<(), LegacyImpactError<E>> {
    let FindingMeasure { old, new, change, subject, context } = inputs;
    let copied =
        add(change_payload(change, control, admit)?, add(subject.len(), 36, admit)?, admit)?;
    budget.owned(
        add(
            mul(2, std::mem::size_of::<super::super::model::ImpactFinding>(), admit)?,
            std::mem::size_of::<super::FindingContext>(),
            admit,
        )?,
        admit,
    )?;
    budget.owned(
        add(
            add(copied, context.retained, admit)?,
            add(context.identity, context.temporary, admit)?,
            admit,
        )?,
        admit,
    )?;
    let mut seed = add(super::super::model::REPORT_SCHEMA_VERSION.len(), subject.len(), admit)?;
    for evidence in [&old.evidence, &new.evidence] {
        seed = add(seed, evidence.raw_sha256.len(), admit)?;
        if let Some(hash) = &evidence.resolved_catalog_sha256 {
            seed = add(seed, hash.len(), admit)?;
        }
    }
    // These arrays are the complete current native vocabularies used in the seed.
    let class = [
        ChangeClass::Added,
        ChangeClass::Removed,
        ChangeClass::ContentChanged,
        ChangeClass::IdentityMigrated,
        ChangeClass::Unchanged,
    ]
    .into_iter()
    .map(|value| value.as_str().len())
    .max()
    .unwrap_or(0);
    let reason = [
        ReasonCode::ControlAdded,
        ReasonCode::ControlRemoved,
        ReasonCode::ControlContentChanged,
        ReasonCode::MappingReferenceRemoved,
        ReasonCode::MappingSubjectChanged,
        ReasonCode::ApplicabilityDecisionRemoved,
        ReasonCode::ApplicabilityDecisionChanged,
        ReasonCode::ApplicabilityDecisionMigrated,
        ReasonCode::IdentityMigrationDeclared,
        ReasonCode::MappingSubjectMigrated,
        ReasonCode::ResourceMetadataChanged,
        ReasonCode::MigrationEvidenceMissing,
    ]
    .into_iter()
    .map(|value| value.as_str().len())
    .max()
    .unwrap_or(0);
    budget.owned(
        add(
            add(seed, add(class, reason, admit)?, admit)?,
            add(context.identity, 8, admit)?,
            admit,
        )?,
        admit,
    )?;
    budget.visits = add(budget.visits, 1, admit)?;
    Ok(())
}

/// Sum lengths/headers of borrowed string-vector values without creating a vector.
fn string_vector<E>(
    values: &[String],
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<usize, LegacyImpactError<E>> {
    let mut amount = mul(values.len(), std::mem::size_of::<String>(), admit)?;
    for value in values {
        checkpoint(control)?;
        amount = add(amount, value.len(), admit)?;
    }
    Ok(amount)
}

/// Admit complete finding/context occurrences from actual classified changes.
fn admit_findings<E>(
    inputs: super::ReportFindingInputs<'_>,
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<(), LegacyImpactError<E>> {
    let super::ReportFindingInputs { old, new, changes, portfolio, applicability, .. } = inputs;
    let mut budget = ReportBudget::default();
    let mut app_ids = 0usize;
    if let Some(facts) = applicability {
        work(
            add(facts.report.controls.len(), facts.manifest.decisions.len(), admit)?,
            0,
            0,
            admit,
        )?;
        for row in &facts.report.controls {
            checkpoint(control)?;
            app_ids = add(app_ids, row.control_id.len(), admit)?;
        }
        for row in &facts.manifest.decisions {
            checkpoint(control)?;
            app_ids = add(app_ids, row.control_id.len(), admit)?;
        }
    }
    work(portfolio.references.len(), 0, 0, admit)?;
    let mut mapping_ids = 0usize;
    for id in portfolio.references.keys() {
        checkpoint(control)?;
        mapping_ids = add(mapping_ids, id.len(), admit)?;
    }
    work(changes.len(), 0, 0, admit)?;
    work(1, add(old.evidence.raw_sha256.len(), new.evidence.raw_sha256.len(), admit)?, 1, admit)?;
    let mut metadata_notice = old.evidence.raw_sha256 != new.evidence.raw_sha256;
    let mut missing_notice = false;
    for change in changes {
        checkpoint(control)?;
        let subjects = add(change.old_subjects.len(), change.new_subjects.len(), admit)?;
        work(subjects, 0, 0, admit)?;
        if change.change_class != ChangeClass::Unchanged {
            metadata_notice = false;
            admit_change_finding(&mut budget, inputs, change, control, admit)?;
        }
        missing_notice |= matches!(change.change_class, ChangeClass::Added | ChangeClass::Removed);
        for subject in &change.old_subjects {
            checkpoint(control)?;
            let keys = portfolio.references.len();
            work(keys, add(mul(subject.id.len(), keys, admit)?, mapping_ids, admit)?, keys, admit)?;
            let references: &[super::MappingReference] =
                portfolio.references.get(&subject.id).map_or(&[], Vec::as_slice);
            // Native build_findings looks up every old subject before the Mapping
            // helper's eligibility guard, including unchanged Controls.
            if !matches!(
                change.change_class,
                ChangeClass::Removed | ChangeClass::ContentChanged | ChangeClass::IdentityMigrated
            ) {
                continue;
            }
            work(references.len(), 0, 0, admit)?;
            for reference in references {
                checkpoint(control)?;
                admit_mapping_finding(
                    &mut budget,
                    inputs,
                    change,
                    subject,
                    reference,
                    control,
                    admit,
                )?;
            }
            admit_applicability_finding(
                &mut budget,
                inputs,
                change,
                subject,
                app_ids,
                control,
                admit,
            )?;
        }
    }
    admit_fixed_notices(&mut budget, inputs, metadata_notice, missing_notice, admit)?;
    budget.emit(admit)
}

/// Admit one actual base finding, retaining the complete native change payload.
fn admit_change_finding<E>(
    budget: &mut ReportBudget,
    inputs: super::ReportFindingInputs<'_>,
    change: &super::super::model::ControlChange,
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<(), LegacyImpactError<E>> {
    let old = inputs.old;
    let new = inputs.new;
    let context = if change.change_class == ChangeClass::IdentityMigrated {
        let owner = change.migration.as_ref().map_or(0, |value| value.approved_by.len());
        ContextBudget {
            retained: add(
                add(
                    add("identity-migration:".len(), "migration:".len(), admit)?,
                    mul(change.subject_id.len(), 2, admit)?,
                    admit,
                )?,
                add(owner, std::mem::size_of::<String>(), admit)?,
                admit,
            )?,
            identity: 64,
            temporary: migration_scratch(change, admit)?.saturating_sub(64),
        }
    } else {
        ContextBudget {
            retained: add(
                add("control:".len(), change.subject_id.len(), admit)?,
                std::mem::size_of::<String>(),
                admit,
            )?,
            identity: "subject".len(),
            temporary: 0,
        }
    };
    finding_budget(
        budget,
        FindingMeasure { old, new, change, subject: &change.subject_id, context },
        control,
        admit,
    )?;
    Ok(())
}

/// Admit one real Mapping dependency occurrence with every private context string.
fn admit_mapping_finding<E>(
    budget: &mut ReportBudget,
    inputs: super::ReportFindingInputs<'_>,
    change: &super::super::model::ControlChange,
    subject: &super::super::model::SubjectFingerprint,
    reference: &super::MappingReference,
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<(), LegacyImpactError<E>> {
    let old = inputs.old;
    let new = inputs.new;
    let mut identity = 3usize;
    for value in [
        &reference.artifact_id,
        &reference.mapping_id,
        &reference.map_id,
        &reference.policy_resource_identity,
    ] {
        identity = add(identity, value.len(), admit)?;
    }
    let mut retained = "control:".len()
        + "mapping-collection:".len()
        + "mapping:".len()
        + "map:".len()
        + "policy-resource:".len();
    retained = add(retained, subject.id.len(), admit)?;
    for value in [
        &reference.artifact_id,
        &reference.mapping_id,
        &reference.map_id,
        &reference.policy_resource_identity,
        &reference.artifact_id,
        &reference.map_id,
        &reference.policy_resource_identity,
        &reference.policy_resource_label,
    ] {
        retained = add(retained, value.len(), admit)?;
    }
    retained = add(retained, mul(6, std::mem::size_of::<String>(), admit)?, admit)?;
    let migrated = change.change_class == ChangeClass::IdentityMigrated;
    let context = ContextBudget {
        retained,
        identity: if migrated { add(identity, 65, admit)? } else { identity },
        temporary: if migrated {
            add(identity, migration_scratch(change, admit)?, admit)?
        } else {
            0
        },
    };
    finding_budget(
        budget,
        FindingMeasure { old, new, change, subject: &subject.id, context },
        control,
        admit,
    )?;
    Ok(())
}

/// Admit the actual optional applicability dependency lookup and complete copied context.
fn admit_applicability_finding<E>(
    budget: &mut ReportBudget,
    inputs: super::ReportFindingInputs<'_>,
    change: &super::super::model::ControlChange,
    subject: &super::super::model::SubjectFingerprint,
    app_ids: usize,
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<(), LegacyImpactError<E>> {
    let old = inputs.old;
    let new = inputs.new;
    let applicability = inputs.applicability;
    if let Some(facts) = applicability {
        let scans = add(facts.report.controls.len(), facts.manifest.decisions.len(), admit)?;
        work(scans, add(mul(subject.id.len(), scans, admit)?, app_ids, admit)?, scans, admit)?;
        if let Some(control_row) =
            facts.report.controls.iter().find(|row| row.control_id == subject.id)
        {
            work(control_row.policy_sources.len(), 0, 0, admit)?;
            let classification = control_row.classification.as_str().len();
            let dependency = add("applicability:".len(), subject.id.len(), admit)?;
            let identity = add(add(dependency, classification, admit)?, 1, admit)?;
            let mut retained =
                "control:".len() + "applicability-manifest:".len() + "gap-state:".len();
            retained = add(retained, subject.id.len(), admit)?;
            retained = add(
                retained,
                mul(
                    add(
                        facts.report.manifest_sha256.len(),
                        add(dependency, classification, admit)?,
                        admit,
                    )?,
                    2,
                    admit,
                )?,
                admit,
            )?;
            retained =
                add(retained, control_row.reviewer_key.as_ref().map_or(0, String::len), admit)?;
            retained = add(
                retained,
                add(
                    mul(4, std::mem::size_of::<String>(), admit)?,
                    string_vector(&control_row.policy_sources, control, admit)?,
                    admit,
                )?,
                admit,
            )?;
            let migrated = change.change_class == ChangeClass::IdentityMigrated;
            let context = ContextBudget {
                retained,
                identity: if migrated { add(identity, 65, admit)? } else { identity },
                temporary: if migrated {
                    add(identity, migration_scratch(change, admit)?, admit)?
                } else {
                    0
                },
            };
            finding_budget(
                budget,
                FindingMeasure { old, new, change, subject: &subject.id, context },
                control,
                admit,
            )?;
        }
    }
    Ok(())
}

/// Admit the same complete fixed native notices after every dependency occurrence.
fn admit_fixed_notices<E>(
    budget: &mut ReportBudget,
    inputs: super::ReportFindingInputs<'_>,
    metadata_notice: bool,
    missing_notice: bool,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<(), LegacyImpactError<E>> {
    let old = inputs.old;
    let new = inputs.new;
    let successor_map = inputs.successor_map;
    // Small complete fixed notices use the same real finding constructor.
    for (enabled, subject, identity, path) in [
        (metadata_notice, "$resource", "catalog-metadata", "catalog-metadata"),
        (
            missing_notice && successor_map.is_none(),
            "$migration-evidence",
            "migration-evidence-missing",
            "successor-map approval trail",
        ),
    ] {
        if !enabled {
            continue;
        }
        // Do not construct even the temporary native ControlChange during planning.
        budget.owned(
            add(std::mem::size_of::<super::super::model::ControlChange>(), subject.len(), admit)?,
            admit,
        )?;
        let seed = add(
            add(super::super::model::REPORT_SCHEMA_VERSION.len(), subject.len(), admit)?,
            add(identity.len(), 8 + "unchanged".len() + "migration_evidence_missing".len(), admit)?,
            admit,
        )?;
        let mut hashes = 0usize;
        for evidence in [&old.evidence, &new.evidence] {
            hashes = add(hashes, evidence.raw_sha256.len(), admit)?;
            if let Some(hash) = &evidence.resolved_catalog_sha256 {
                hashes = add(hashes, hash.len(), admit)?;
            }
        }
        let fixed = add(
            mul(2, std::mem::size_of::<super::super::model::ImpactFinding>(), admit)?,
            std::mem::size_of::<super::FindingContext>(),
            admit,
        )?;
        let strings = add(
            add(subject.len(), 36, admit)?,
            add(add(path.len(), identity.len(), admit)?, std::mem::size_of::<String>(), admit)?,
            admit,
        )?;
        budget.owned(add(add(fixed, strings, admit)?, add(seed, hashes, admit)?, admit)?, admit)?;
        budget.visits = add(budget.visits, 1, admit)?;
    }
    Ok(())
}

/// Admit every native group clone attempt, including identical repeated ancestry.
fn admit_groups<E>(
    old: &inventory::Inventory,
    new: &inventory::Inventory,
    findings: &[super::super::model::ImpactFinding],
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<(), LegacyImpactError<E>> {
    let controls = add(old.count(SubjectType::Control), new.count(SubjectType::Control), admit)?;
    work(controls, 0, 0, admit)?;
    let mut id_bytes = 0usize;
    for id in
        old.ids_of_type_refs(SubjectType::Control).chain(new.ids_of_type_refs(SubjectType::Control))
    {
        checkpoint(control)?;
        id_bytes = add(id_bytes, id.len(), admit)?;
    }
    let mut budget = ReportBudget::default();
    for finding in findings {
        checkpoint(control)?;
        work(add(finding.old_subjects.len(), finding.new_subjects.len(), admit)?, 0, 0, admit)?;
        budget.owned(std::mem::size_of::<BTreeSet<String>>(), admit)?;
        let mut attempts = 0usize;
        let mut width = 0usize;
        for subject in finding.old_subjects.iter().chain(&finding.new_subjects) {
            work(
                controls,
                add(mul(subject.id.len(), controls, admit)?, id_bytes, admit)?,
                controls,
                admit,
            )?;
            for inventory in [old, new] {
                let groups = inventory.groups_for_control(&subject.id);
                work(groups.len(), 0, 0, admit)?;
                for group in groups {
                    checkpoint(control)?;
                    attempts = add(attempts, 1, admit)?;
                    width = width.max(group.len());
                    // The set string moves into the Vec; no payload dedup credit.
                    budget.owned(
                        add(
                            add(REPORT_REGISTRY_SLOT, std::mem::size_of::<String>(), admit)?,
                            group.len(),
                            admit,
                        )?,
                        admit,
                    )?;
                }
            }
        }
        budget.probes(mul(attempts, attempts, admit)?, mul(width, 2, admit)?, admit)?;
        budget.visits = add(budget.visits, attempts, admit)?;
    }
    budget.emit(admit)
}

/// Admit native stable-sort scratch and all five exact tuple comparison positions.
fn admit_finding_sort<E>(
    changes: usize,
    findings: &[super::super::model::ImpactFinding],
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<(), LegacyImpactError<E>> {
    work(findings.len(), 0, 0, admit)?;
    let mut width = 0usize;
    for finding in findings {
        checkpoint(control)?;
        width = width.max(add(
            add(finding.subject_id.len(), finding.finding_id.len(), admit)?,
            finding.dependency_id.as_ref().map_or(0, String::len),
            admit,
        )?);
    }
    let comparisons = add(mul(findings.len(), findings.len(), admit)?, findings.len(), admit)?;
    reserve(
        mul(findings.len(), std::mem::size_of::<super::super::model::ImpactFinding>(), admit)?,
        admit,
    )?;
    // Sorting reads findings; complete summary reads every change and finding again.
    work(
        add(changes, mul(findings.len(), 2, admit)?, admit)?,
        mul(mul(comparisons, width, admit)?, 2, admit)?,
        mul(comparisons, 5, admit)?,
        admit,
    )
}

/// Admit current finding-ID copies, full evidence Values and prior-only row moves.
/// Original strict prior/disposition decoding charges remain separate and unchanged.
fn admit_history_report<E>(
    report: &ImpactReport,
    dispositions: &[super::super::disposition::DispositionRecord],
    prior_extent: usize,
    prior_slots: usize,
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<(), LegacyImpactError<E>> {
    work(add(report.findings.len(), dispositions.len(), admit)?, 0, 0, admit)?;
    let mut budget = ReportBudget::default();
    let mut width = 0usize;
    let mut current_id_bytes = 0usize;
    for finding in &report.findings {
        checkpoint(control)?;
        width = width.max(finding.finding_id.len());
        current_id_bytes = add(current_id_bytes, finding.finding_id.len(), admit)?;
        budget.owned(
            add(
                add(REPORT_REGISTRY_SLOT, std::mem::size_of::<(String, usize)>(), admit)?,
                mul(finding.finding_id.len(), 2, admit)?,
                admit,
            )?,
            admit,
        )?;
    }
    let mut disposition_id_bytes = 0usize;
    for disposition in dispositions {
        checkpoint(control)?;
        width = width.max(disposition.finding_id.len());
        disposition_id_bytes = add(disposition_id_bytes, disposition.finding_id.len(), admit)?;
    }
    for evidence in [&report.old, &report.new] {
        checkpoint(control)?;
        let keys = [
            "resource_type",
            "href",
            "raw_sha256",
            "root_uuid",
            "document_version",
            "oscal_version",
        ];
        let values = [
            "catalog",
            evidence.href.as_str(),
            evidence.raw_sha256.as_str(),
            evidence.root_uuid.as_str(),
            evidence.document_version.as_str(),
            evidence.oscal_version.as_str(),
        ];
        budget.owned(std::mem::size_of::<Value>(), admit)?;
        for (key, value) in keys.into_iter().zip(values) {
            budget.owned(
                add(
                    add(REPORT_REGISTRY_SLOT, std::mem::size_of::<(String, Value)>(), admit)?,
                    add(key.len(), value.len(), admit)?,
                    admit,
                )?,
                admit,
            )?;
        }
        if let Some(hash) = &evidence.resolved_catalog_sha256 {
            budget.owned(
                add(
                    add(REPORT_REGISTRY_SLOT, std::mem::size_of::<(String, Value)>(), admit)?,
                    add("resolved_catalog_sha256".len(), hash.len(), admit)?,
                    admit,
                )?,
                admit,
            )?;
        }
    }
    // Asserted strings move unchanged. Two row footprints cover prior-only Vec
    // growth and a third covers its native stable-sort scratch; no row is dropped.
    budget.owned(
        mul(
            mul(dispositions.len(), 3, admit)?,
            std::mem::size_of::<super::super::disposition::DispositionRecord>(),
            admit,
        )?,
        admit,
    )?;
    // Prior IDs remain borrowed, but their set has one logical registry slot per
    // lexical upper-bound occurrence. The original strict Value parse is unchanged.
    budget.owned(mul(prior_slots, REPORT_REGISTRY_SLOT, admit)?, admit)?;
    let current = mul(report.findings.len(), report.findings.len(), admit)?;
    let lookup = mul(dispositions.len(), add(report.findings.len(), prior_slots, admit)?, admit)?;
    let prior = mul(prior_slots, prior_slots, admit)?;
    let sorting = mul(dispositions.len(), dispositions.len(), admit)?;
    budget.probes(add(current, sorting, admit)?, mul(width, 2, admit)?, admit)?;
    budget.probes(add(lookup, prior, admit)?, 0, admit)?;
    // Each BTree insertion/search examines each retained candidate at most once.
    // Across all prior ID insertions, complete decoded ID bytes are bounded by
    // raw prior bytes: 2*n*sum(widths), rather than n*n*whole-input-width.
    let prior_work = mul(mul(prior_slots, prior_extent, admit)?, 2, admit)?;
    let lookup_prior = add(
        mul(prior_slots, disposition_id_bytes, admit)?,
        mul(dispositions.len(), prior_extent, admit)?,
        admit,
    )?;
    let lookup_current = add(
        mul(report.findings.len(), disposition_id_bytes, admit)?,
        mul(dispositions.len(), current_id_bytes, admit)?,
        admit,
    )?;
    budget.string_bytes = add(
        budget.string_bytes,
        add(prior_work, add(lookup_prior, lookup_current, admit)?, admit)?,
        admit,
    )?;
    // Full private old/new Value equality may inspect the complete prior evidence.
    budget.string_bytes = add(budget.string_bytes, mul(prior_extent, 2, admit)?, admit)?;
    budget.visits =
        add(add(report.findings.len(), dispositions.len(), admit)?, prior_slots, admit)?;
    budget.emit(admit)
}

/// Count the complete dependency roster; admission precedes repeated borrowed traversal.
fn reference_count<E>(
    portfolio: &MappingPortfolio,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<usize, LegacyImpactError<E>> {
    work(portfolio.references.len(), 0, portfolio.references.len(), admit)?;
    let mut count = 0;
    for values in portfolio.references.values() {
        count = add(count, values.len(), admit)?;
    }
    Ok(count)
}

/// Preserve all five filters; string work uses complete private values before validation.
fn filter_extent<E>(
    filters: &ImpactFilters,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<usize, LegacyImpactError<E>> {
    let mut extent = 0;
    for value in
        [filters.group.as_deref(), filters.policy_source.as_deref(), filters.owner.as_deref()]
            .into_iter()
            .flatten()
    {
        extent = add(extent, value.len(), admit)?;
    }
    Ok(extent)
}

/// Preserve full legacy applicability errors while keeping admission/work/capacity distinct.
fn map_applicability_error<E>(
    error: applicability::LegacyBorrowedError<E>,
) -> LegacyImpactError<E> {
    match error {
        applicability::LegacyBorrowedError::Domain(error) => {
            LegacyImpactError::Domain(super::map_applicability_error(error))
        }
        applicability::LegacyBorrowedError::Admission(error) => LegacyImpactError::Admission(error),
        applicability::LegacyBorrowedError::Work(error) => LegacyImpactError::Work(error),
        applicability::LegacyBorrowedError::Capacity => LegacyImpactError::Capacity,
    }
}

/// Preserve first caller failures; fence both native success and native domain failure.
fn phase<E, T, A>(
    control: &mut dyn WorkControl,
    admit: &mut A,
    operation: impl FnOnce(&mut dyn WorkControl, &mut A) -> Result<T, LegacyImpactError<E>>,
) -> Result<T, LegacyImpactError<E>>
where
    A: FnMut(ImpactCharge) -> Result<(), E>,
{
    checkpoint(control)?;
    let result = operation(control, admit);
    // Admission/Work already came from the actual caller during this phase.
    // Return that first failure directly; a later checkpoint cannot replace it.
    // Native domain failures and successes retain their required post-phase fence.
    if matches!(&result, Err(LegacyImpactError::Admission(_) | LegacyImpactError::Work(_))) {
        return result;
    }
    checkpoint(control)?;
    result
}

/// Consult the actual original `WorkControl`; no replacement deadline/state or numeric progress.
fn checkpoint<E>(control: &mut dyn WorkControl) -> Result<(), LegacyImpactError<E>> {
    control
        .checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)
        .map_err(LegacyImpactError::Work)
}

/// Preserve the actual admission error without clearing its caller's sticky state.
fn reserve<E>(
    amount: usize,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<(), LegacyImpactError<E>> {
    admit(ImpactCharge::Bytes(amount)).map_err(LegacyImpactError::Admission)
}
/// Request complete row visits from the actual caller before collection growth.
fn rows<E>(
    amount: usize,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<(), LegacyImpactError<E>> {
    admit(ImpactCharge::Rows(amount)).map_err(LegacyImpactError::Admission)
}
/// Request full repeated node/string/comparison work from the original admission callback.
fn work<E>(
    nodes: usize,
    string_bytes: usize,
    comparisons: usize,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<(), LegacyImpactError<E>> {
    admit(ImpactCharge::Work { nodes, string_bytes, comparisons })
        .map_err(LegacyImpactError::Admission)
}
/// Refuse overflow through actual Capacity notification before bounded arithmetic succeeds.
fn add<E>(
    left: usize,
    right: usize,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<usize, LegacyImpactError<E>> {
    match left.checked_add(right) {
        Some(value) => Ok(value),
        None => capacity(admit),
    }
}
/// Refuse product overflow through actual Capacity notification before downstream growth.
fn mul<E>(
    left: usize,
    right: usize,
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<usize, LegacyImpactError<E>> {
    match left.checked_mul(right) {
        Some(value) => Ok(value),
        None => capacity(admit),
    }
}
/// Return actual first admission failure or Capacity even if its notification is accepted.
fn capacity<E, T>(
    admit: &mut impl FnMut(ImpactCharge) -> Result<(), E>,
) -> Result<T, LegacyImpactError<E>> {
    admit(ImpactCharge::Capacity).map_err(LegacyImpactError::Admission)?;
    Err(LegacyImpactError::Capacity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashing::sha256_hex;
    use crate::workspace::preparation::{Interruption, NoopControl, test_support::Recorder};
    use serde_json::{Value, json};

    const FRAMEWORK: &str = "77777777-7777-4777-8777-777777777777";
    const POLICY: &str = "88888888-8888-4888-8888-888888888888";

    /// Genuine file fixtures; all raw hashes and native Mapping fingerprints are producer-derived.
    struct Fixture {
        /// Held actual temporary root throughout native producers and comparison.
        directory: tempfile::TempDir,
        /// Complete ordinary Impact declaration bytes.
        manifest: Vec<u8>,
        /// Actual native old framework artifact bytes.
        old: Vec<u8>,
        /// Actual native new framework artifact bytes.
        new: Vec<u8>,
        /// Explicit complete old resolved Catalog bytes, if configured.
        old_companion: Option<Vec<u8>>,
        /// Explicit complete new resolved Catalog bytes, if configured.
        new_companion: Option<Vec<u8>>,
        /// Genuine native Mapping artifact originals in declared order.
        mappings: Vec<Vec<u8>>,
        /// Genuine scaffold plus explicitly edited applicability declarations.
        applicability: Option<Vec<u8>>,
        /// Complete configured applicability Mapping originals.
        applicability_mappings: Vec<Vec<u8>>,
        /// Complete declared native successor/split/merge input.
        successor: Option<Vec<u8>>,
        /// Complete prior report bytes from the maintained producer.
        prior: Option<Vec<u8>>,
        /// Complete declared assertions bound to actual prior report bytes.
        dispositions: Option<Vec<u8>>,
    }

    /// Serialize source-proposed genuine fixture values without inventing native facts.
    fn bytes(value: &Value) -> Vec<u8> {
        serde_json::to_vec_pretty(value).expect("serialize genuine fixture")
    }
    /// Declare a redistributable native Catalog fixture with control statement content.
    fn catalog(uuid: &str, version: &str, controls: &[(&str, &str)]) -> Value {
        json!({"catalog":{"uuid":uuid,"metadata":{"title":"Synthetic redistributable framework","last-modified":"2026-08-25T12:00:00Z","version":version,"oscal-version":"1.2.3"},"groups":[{"id":"group-1","title":"Synthetic group","controls":controls.iter().map(|(id, prose)| json!({"id":id,"title":format!("Control {id}"),"parts":[{"id":format!("{id}_smt"),"name":"statement","prose":prose}]})).collect::<Vec<_>>()}]}})
    }
    /// Declare exact raw native resource hashes and private tuple expectations.
    fn resource(artifact: &str, raw: &[u8], version: &str) -> Value {
        json!({"type":"catalog","artifact":artifact,"expected_sha256":sha256_hex(raw),"root_uuid":FRAMEWORK,"document_version":version,"oscal_version":"1.2.3"})
    }
    /// Hold complete old/new native Catalog originals for all ordinary parity checks.
    fn basic() -> Fixture {
        let directory = tempfile::tempdir().expect("genuine fixture directory");
        let old = bytes(&catalog(
            FRAMEWORK,
            "1.0.0",
            &[("unchanged", "Same"), ("changed", "Old"), ("removed", "Removed")],
        ));
        let new = bytes(&catalog(
            FRAMEWORK,
            "2.0.0",
            &[("unchanged", "Same"), ("changed", "New"), ("added", "Added")],
        ));
        let manifest = bytes(
            &json!({"schema_version":"forge.framework-impact/1","old":resource("old.json",&old,"1.0.0"),"new":resource("new.json",&new,"2.0.0"),"mapping_collections":[]}),
        );
        Fixture {
            directory,
            manifest,
            old,
            new,
            old_companion: None,
            new_companion: None,
            mappings: Vec::new(),
            applicability: None,
            applicability_mappings: Vec::new(),
            successor: None,
            prior: None,
            dispositions: None,
        }
    }
    impl Fixture {
        /// Install only explicit legacy read originals; no generated policy route is reconstructed.
        fn install(&self) {
            for (name, raw) in [
                ("old.json", self.old.as_slice()),
                ("new.json", self.new.as_slice()),
                ("impact.json", self.manifest.as_slice()),
            ] {
                std::fs::write(self.directory.path().join(name), raw)
                    .expect("write fixture original");
            }
            for (name, raw) in [
                ("old-resolved.json", self.old_companion.as_deref()),
                ("new-resolved.json", self.new_companion.as_deref()),
                ("applicability.json", self.applicability.as_deref()),
                ("successor.json", self.successor.as_deref()),
                ("prior.json", self.prior.as_deref()),
                ("dispositions.json", self.dispositions.as_deref()),
            ] {
                if let Some(raw) = raw {
                    std::fs::write(self.directory.path().join(name), raw)
                        .expect("write declared fixture original");
                }
            }
            for (index, raw) in self.mappings.iter().enumerate() {
                std::fs::write(self.directory.path().join(format!("mapping-{index}.json")), raw)
                    .expect("write genuine Mapping");
            }
        }
        /// Compare every private native report field, hidden row and all fifteen counters.
        fn compare(&self, filters: ImpactFilters) -> ImpactReport {
            self.install();
            let manifest =
                crate::framework::manifest::parse(&self.manifest).expect("parse genuine manifest");
            let (legacy, _) = crate::framework::analysis::analyze(
                self.directory.path(),
                &manifest,
                filters.clone(),
            )
            .expect("maintained file producer");
            let mapping_refs: Vec<_> = self.mappings.iter().map(Vec::as_slice).collect();
            let app_mapping_refs: Vec<_> =
                self.applicability_mappings.iter().map(Vec::as_slice).collect();
            let borrowed = prepare(
                self.inputs(&mapping_refs, &app_mapping_refs),
                filters,
                &mut NoopControl,
                &mut |_| Ok::<(), ()>(()),
            )
            .expect("ordinary borrowed data");
            assert_eq!(
                serde_json::to_value(&legacy).unwrap(),
                serde_json::to_value(&borrowed.report).unwrap(),
                "complete private report differs"
            );
            assert_eq!(
                serde_json::to_value(&legacy.filtered_out_findings).unwrap(),
                serde_json::to_value(&borrowed.report.filtered_out_findings).unwrap(),
                "hidden full filter rows differ"
            );
            assert_eq!(
                legacy.summary.rows(),
                borrowed.report.summary.rows(),
                "all fifteen counters differ"
            );
            assert_eq!(legacy.old, borrowed.report.old);
            assert_eq!(legacy.new, borrowed.report.new);
            borrowed.report
        }
        /// Borrow complete ordinary data originals without producing capture or source proofs.
        fn inputs<'a>(
            &'a self,
            mappings: &'a [&'a [u8]],
            app_mappings: &'a [&'a [u8]],
        ) -> ImpactInputs<'a> {
            ImpactInputs {
                manifest_dir: self.directory.path(),
                manifest: &self.manifest,
                old: FrameworkBytes {
                    artifact: &self.old,
                    resolved_catalog: self.old_companion.as_deref(),
                },
                new: FrameworkBytes {
                    artifact: &self.new,
                    resolved_catalog: self.new_companion.as_deref(),
                },
                mappings,
                applicability: self.applicability.as_deref().map(|manifest| ApplicabilityBytes {
                    manifest,
                    framework: &self.old,
                    resolved_catalog: self.old_companion.as_deref(),
                    mappings: app_mappings,
                }),
                successor_map: self.successor.as_deref(),
                prior_report: self.prior.as_deref(),
                dispositions: self.dispositions.as_deref(),
            }
        }
    }

    /// Obtain the complete mandatory native inventory snapshot through `execute_init`.
    fn native_applicability_scaffold(framework: &Path, output: &Path) -> Value {
        crate::applicability::execute_init(framework, None, Some(output))
            .expect("genuine maintained applicability scaffold producer");
        serde_json::from_slice(&std::fs::read(output).expect("read genuine scaffold"))
            .expect("decode genuine scaffold data")
    }

    /// Build an authentic Mapping via the maintained native producer, then retain exact artifact bytes.
    fn native_mapping(fixture: &Fixture, key: &str, framework_is_target: bool) -> Vec<u8> {
        native_mapping_controls(
            fixture,
            key,
            framework_is_target,
            "policy.json",
            &["changed", "removed"],
        )
    }

    /// Produce complete native Mapping bytes for every supplied framework Control and private label.
    fn native_mapping_controls(
        fixture: &Fixture,
        key: &str,
        framework_is_target: bool,
        policy_href: &str,
        controls: &[&str],
    ) -> Vec<u8> {
        fixture.install();
        std::fs::write(
            fixture.directory.path().join("policy.json"),
            bytes(&catalog(POLICY, "1.0.0", &[("policy-1", "Policy")])),
        )
        .unwrap();
        let framework_resource = json!({"type":"catalog","artifact":"old.json","href":"old.json"});
        let policy_resource = json!({"type":"catalog","artifact":"policy.json","href":policy_href});
        let (source, target) = if framework_is_target {
            (policy_resource, framework_resource)
        } else {
            (framework_resource, policy_resource)
        };
        let maps: Vec<_> = controls.iter().enumerate().map(|(index,id)| {
            let framework_item = json!({"type":"control","id_ref":id});
            let policy_item = json!({"type":"control","id_ref":"policy-1"});
            let (source_item,target_item) = if framework_is_target { (policy_item,framework_item) } else { (framework_item,policy_item) };
            json!({"key":format!("{key}-{index}"),"relationship":"intersects-with","sources":[source_item],"targets":[target_item],"reviewer_key":"reviewer","reviewed_at":"2026-08-25T12:00:00Z","rationale":"Synthetic native Mapping relationship."})
        }).collect();
        let declaration = bytes(
            &json!({"schema_version":"forge.mapping-manifest/1","collection":{"key":key,"title":"Synthetic native Mapping","version":"1.0.0","last_modified":"2026-08-25T12:00:00Z"},"reviewers":[{"key":"reviewer","type":"person","name":"Synthetic Reviewer"}],"provenance":{"method":"human","matching_rationale":"semantic","status":"complete","mapping_description":"Synthetic test declarations; no project approval.","reviewer_keys":["reviewer"],"reviewed_at":"2026-08-25T12:00:00Z"},"mapping":{"key":key,"scope":"control-only","source":source,"target":target,"maps":maps}}),
        );
        let path = fixture.directory.path().join("mapping-build.json");
        std::fs::write(&path, declaration).unwrap();
        let prepared = crate::mapping::prepare(&path, None, false)
            .expect("genuine maintained Mapping producer");
        prepared.artifact_json.into_bytes()
    }

    #[test]
    /// Compare all native classes/private tuples and metadata findings against the maintained producer.
    fn full_native_classes_private_tuples_and_metadata_findings_match_legacy() {
        let fixture = basic();
        let report = fixture.compare(ImpactFilters::default());
        assert_eq!(
            (
                report.summary.added,
                report.summary.removed,
                report.summary.content_changed,
                report.summary.unchanged
            ),
            (1, 1, 1, 1)
        );
        assert!(report.changes.iter().any(|row| row.subject_id == "unchanged"));
        let mut metadata = basic();
        metadata.new = bytes(&catalog(
            FRAMEWORK,
            "2.0.0 private-name",
            &[("unchanged", "Same"), ("changed", "Old"), ("removed", "Removed")],
        ));
        let mut declaration: Value = serde_json::from_slice(&metadata.manifest).unwrap();
        declaration["new"] = resource("new.json", &metadata.new, "2.0.0 private-name");
        metadata.manifest = bytes(&declaration);
        let report = metadata.compare(ImpactFilters::default());
        assert_eq!(
            (
                report.summary.added,
                report.summary.removed,
                report.summary.content_changed,
                report.summary.identity_migrated
            ),
            (0, 0, 0, 0)
        );
        assert_eq!(report.new.document_version, "2.0.0 private-name");
        assert_eq!(report.new.href, "new.json");
        assert_eq!(report.findings.len(), 1);
        assert_eq!(
            report.findings[0].reason_code,
            crate::framework::model::ReasonCode::ResourceMetadataChanged
        );
    }

    #[test]
    /// Compare full Profile-companion native reports while leaving imported hrefs unopened.
    fn profile_companion_native_change_report_matches_full_legacy_result() {
        let mut fixture = basic();
        fixture.old_companion = Some(fixture.old.clone());
        fixture.new_companion = Some(fixture.new.clone());
        let profile = |version: &str| {
            bytes(
                &json!({"profile":{"uuid":FRAMEWORK,"metadata":{"title":"Synthetic profile","last-modified":"2026-08-25T12:00:00Z","version":version,"oscal-version":"1.2.3"},"imports":[{"href":"unopened-import.json","include-all":{}}]}}),
            )
        };
        fixture.old = profile("1.0.0");
        fixture.new = profile("2.0.0");
        let pair = |artifact: &str,
                    raw: &[u8],
                    companion: &str,
                    companion_raw: &[u8],
                    version: &str| json!({"type":"profile","artifact":artifact,"resolved_catalog":companion,"resolved_catalog_attestation":true,"expected_sha256":sha256_hex(raw),"expected_resolved_catalog_sha256":sha256_hex(companion_raw),"root_uuid":FRAMEWORK,"document_version":version,"oscal_version":"1.2.3"});
        fixture.manifest = bytes(
            &json!({"schema_version":"forge.framework-impact/1","old":pair("old.json",&fixture.old,"old-resolved.json",fixture.old_companion.as_deref().unwrap(),"1.0.0"),"new":pair("new.json",&fixture.new,"new-resolved.json",fixture.new_companion.as_deref().unwrap(),"2.0.0"),"mapping_collections":[]}),
        );
        let report = fixture.compare(ImpactFilters::default());
        assert!(report.old.resolved_catalog_sha256.is_some());
        assert_eq!(report.summary.content_changed, 1);
        assert!(!fixture.directory.path().join("unopened-import.json").exists());
    }

    #[test]
    /// Compare genuine scaffolded applicability, both roles, all filters and historical dispositions.
    fn both_mapping_roles_applicability_all_filters_and_prior_dispositions_match_without_policy_reads()
     {
        let mut fixture = basic();
        let target = native_mapping(&fixture, "target-native", true);
        let source = native_mapping(&fixture, "source-native", false);
        fixture.mappings = vec![target.clone(), source];
        fixture.applicability_mappings = vec![target];
        let mut app = native_applicability_scaffold(
            &fixture.directory.path().join("old.json"),
            &fixture.directory.path().join("applicability.json"),
        );
        app["reviewers"] = json!([{"key":"owner","type":"person","name":"Synthetic Owner"}]);
        app["decisions"] = json!([{"control_id":"changed","state":"applicable","reviewer_key":"owner","reviewed_at":"2026-08-25T12:00:00Z"},{"control_id":"removed","state":"not-applicable","reviewer_key":"owner","reviewed_at":"2026-08-25T12:00:00Z","rationale":"Synthetic scope assertion."}]);
        app["mapping_collections"] = json!(["mapping-0.json"]);
        fixture.applicability = Some(bytes(&app));
        let mut declaration: Value = serde_json::from_slice(&fixture.manifest).unwrap();
        declaration["mapping_collections"] = json!([{"artifact":"mapping-0.json","framework_role":"target"},{"artifact":"mapping-1.json","framework_role":"source"}]);
        declaration["applicability_manifest"] = json!("applicability.json");
        fixture.manifest = bytes(&declaration);
        // Remove producer-only inputs before both legacy Impact and borrowed comparison.
        std::fs::remove_file(fixture.directory.path().join("policy.json")).unwrap();
        std::fs::remove_file(fixture.directory.path().join("mapping-build.json")).unwrap();
        let prior = fixture.compare(ImpactFilters::default());
        let source_artifact: Value = serde_json::from_slice(&fixture.mappings[1]).unwrap();
        let source_uuid = source_artifact["mapping-collection"]["uuid"].as_str().unwrap();
        let prior_only = prior
            .findings
            .iter()
            .find(|row| row.affected_artifact_id.as_deref() == Some(source_uuid))
            .expect("genuine source-role dependency finding");
        let current = prior
            .findings
            .iter()
            .find(|row| row.prior_decision_state.is_some())
            .expect("genuine applicability finding");
        fixture.prior = Some(serde_json::to_vec_pretty(&prior).unwrap());
        fixture.dispositions = Some(bytes(
            &json!({"schema_version":"forge.framework-impact-dispositions/1","prior_report_sha256":sha256_hex(fixture.prior.as_deref().unwrap()),"dispositions":[{"finding_id":prior_only.finding_id,"status":"accepted-risk","decided_by":"Synthetic Owner","decided_at":"2026-08-25T12:00:00Z","rationale":"Synthetic historical disposition."},{"finding_id":current.finding_id,"status":"resolved","decided_by":"Synthetic Owner","decided_at":"2026-08-25T12:00:00Z","rationale":"Synthetic exact-pair disposition."}]}),
        ));
        fixture.mappings.truncate(1);
        declaration["mapping_collections"] =
            json!([{"artifact":"mapping-0.json","framework_role":"target"}]);
        declaration["prior_report"] = json!("prior.json");
        declaration["disposition_file"] = json!("dispositions.json");
        fixture.manifest = bytes(&declaration);
        let full = fixture.compare(ImpactFilters::default());
        assert_eq!(full.prior_only_dispositions.len(), 1);
        for filters in [
            ImpactFilters { group: Some("group-1".into()), ..ImpactFilters::default() },
            ImpactFilters {
                decision_state: Some(crate::applicability::manifest::DecisionState::Applicable),
                ..ImpactFilters::default()
            },
            ImpactFilters { policy_source: Some("policy.json".into()), ..ImpactFilters::default() },
            ImpactFilters {
                priority: Some(crate::framework::model::FindingPriority::ReviewRequired),
                ..ImpactFilters::default()
            },
            ImpactFilters { owner: Some("owner".into()), ..ImpactFilters::default() },
            ImpactFilters {
                group: Some("group-1".into()),
                decision_state: Some(crate::applicability::manifest::DecisionState::Applicable),
                policy_source: Some("policy.json".into()),
                priority: Some(crate::framework::model::FindingPriority::ReviewRequired),
                owner: Some("owner".into()),
            },
        ] {
            fixture.compare(filters);
        }
        assert!(!fixture.directory.path().join("policy.json").exists());
        assert!(!fixture.directory.path().join("mapping-build.json").exists());
    }

    #[test]
    /// Compare all complete native migration declarations and evidence with legacy semantics.
    fn genuine_successor_split_merge_full_reports_match_legacy() {
        let mut fixture = basic();
        fixture.old = bytes(&catalog(
            FRAMEWORK,
            "1.0.0",
            &[
                ("successor-old", "A"),
                ("split-old", "B"),
                ("merge-old-a", "C"),
                ("merge-old-b", "D"),
            ],
        ));
        fixture.new = bytes(&catalog(
            FRAMEWORK,
            "2.0.0",
            &[
                ("successor-new", "A"),
                ("split-new-a", "B"),
                ("split-new-b", "B"),
                ("merge-new", "CD"),
            ],
        ));
        fixture.successor = Some(bytes(
            &json!({"schema_version":"forge.successor-map/1","relationships":[{"relationship":"successor","old_ids":["successor-old"],"new_ids":["successor-new"],"approved_by":"Synthetic Reviewer","approved_at":"2026-08-25T12:00:00Z","rationale":"Synthetic successor declaration."},{"relationship":"split","old_ids":["split-old"],"new_ids":["split-new-b","split-new-a"],"approved_by":"Synthetic Reviewer","approved_at":"2026-08-25T12:00:00Z","rationale":"Synthetic split declaration."},{"relationship":"merge","old_ids":["merge-old-b","merge-old-a"],"new_ids":["merge-new"],"approved_by":"Synthetic Reviewer","approved_at":"2026-08-25T12:00:00Z","rationale":"Synthetic merge declaration."}]}),
        ));
        fixture.manifest = bytes(
            &json!({"schema_version":"forge.framework-impact/1","old":resource("old.json",&fixture.old,"1.0.0"),"new":resource("new.json",&fixture.new,"2.0.0"),"mapping_collections":[],"successor_map":"successor.json"}),
        );
        let report = fixture.compare(ImpactFilters::default());
        assert_eq!(report.summary.identity_migrated, 3);
        assert!(report.changes.iter().all(|row| row.migration.is_some()));
    }

    #[test]
    /// Use a genuine nested scaffold and compare the full native own-base applicability result.
    fn applicability_uses_its_own_declared_nested_base_with_full_legacy_parity() {
        let mut fixture = basic();
        let nested = fixture.directory.path().join("nested");
        std::fs::create_dir(&nested).unwrap();
        let mut declaration: Value = serde_json::from_slice(&fixture.manifest).unwrap();
        declaration["applicability_manifest"] = json!("nested/applicability.json");
        fixture.manifest = bytes(&declaration);
        fixture.install();
        std::fs::write(nested.join("baseline.json"), &fixture.old).unwrap();
        let mut app = native_applicability_scaffold(
            &nested.join("baseline.json"),
            &nested.join("applicability.json"),
        );
        app["framework"]["href"] = json!("private-framework-label.json");
        let app = bytes(&app);
        std::fs::write(nested.join("applicability.json"), &app).unwrap();
        let parsed = crate::framework::manifest::parse(&fixture.manifest).unwrap();
        let (legacy, paths) = crate::framework::analysis::analyze(
            fixture.directory.path(),
            &parsed,
            ImpactFilters::default(),
        )
        .expect("genuine maintained nested legacy reads");
        assert!(paths.contains(&nested.join("baseline.json")));
        let mut inputs = fixture.inputs(&[], &[]);
        inputs.applicability = Some(ApplicabilityBytes {
            manifest: &app,
            framework: &fixture.old,
            resolved_catalog: None,
            mappings: &[],
        });
        let borrowed =
            prepare(inputs, ImpactFilters::default(), &mut NoopControl, &mut |_| Ok::<(), ()>(()))
                .expect("nested borrowed legacy facts");
        assert_eq!(
            serde_json::to_value(&legacy).unwrap(),
            serde_json::to_value(&borrowed.report).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&legacy.filtered_out_findings).unwrap(),
            serde_json::to_value(&borrowed.report.filtered_out_findings).unwrap()
        );
        assert_eq!(legacy.summary.rows(), borrowed.report.summary.rows());
        assert_eq!(
            borrowed.applicability_manifest.unwrap().framework.artifact,
            Path::new("baseline.json")
        );
    }

    /// Use genuine long-label native rows to require actual ledger refusal before projection growth.
    #[test]
    fn complete_mapping_projection_denial_precedes_native_registry_growth() {
        use crate::review::decode::{ContractError, ContractLedger};
        let mut fixture = basic();
        let ids: Vec<String> = (0..272).map(|index| format!("control-{index}")).collect();
        let old_controls: Vec<_> =
            ids.iter().map(|id| (id.as_str(), "Old native requirement")).collect();
        let new_controls: Vec<_> =
            ids.iter().map(|id| (id.as_str(), "New native requirement")).collect();
        fixture.old = bytes(&catalog(FRAMEWORK, "1.0.0", &old_controls));
        fixture.new = bytes(&catalog(FRAMEWORK, "2.0.0", &new_controls));
        fixture.manifest = bytes(
            &json!({"schema_version":"forge.framework-impact/1","old":resource("old.json",&fixture.old,"1.0.0"),"new":resource("new.json",&fixture.new,"2.0.0"),"mapping_collections":[]}),
        );
        let href = format!("{}.json", "p".repeat(64_000));
        let references: Vec<_> = ids.iter().map(String::as_str).collect();
        let mapping =
            native_mapping_controls(&fixture, "projection-native", true, &href, &references);
        let collection = super::super::decode_mapping_bytes("$.mapping_collections[0]", &mapping)
            .expect("genuine complete maintained native Mapping decode");
        let mut ledger = ContractLedger::default();
        let mut projection_request = None;
        let denied = admit_mapping_projection(
            &collection,
            crate::framework::manifest::FrameworkRole::Target,
            ids.len(),
            0,
            &mut NoopControl,
            &mut |charge| {
                ledger.bound(|ledger| match charge {
                    ImpactCharge::Work { nodes, string_bytes, comparisons } => {
                        ledger.visits(nodes)?;
                        ledger.bytes(string_bytes)?;
                        ledger.matching(comparisons)
                    }
                    ImpactCharge::MappingProjection { logical_bytes, controls } => {
                        projection_request = Some((logical_bytes, controls));
                        ledger.derived(logical_bytes)?;
                        ledger.visits(controls)
                    }
                    ImpactCharge::Capacity => Err(ContractError::Capacity),
                    _ => Err(ContractError::Invalid),
                })
            },
        );
        assert!(matches!(denied, Err(LegacyImpactError::Admission(ContractError::Capacity))));
        let (payload, controls) =
            projection_request.expect("complete pre-growth multiplier reached");
        assert_eq!(controls, ids.len());
        assert!(payload >= 2 * href.len() * ids.len());
        assert!(payload > 33_554_432);
        assert!(matches!(ledger.bound(|_| Ok(())), Err(ContractError::Capacity)));
        // On execution this targets only the projection admission boundary. Actual
        // command-wide ledger/capture ownership and sticky outer fence stay absent.
        fixture.mappings = vec![mapping];
        let mut declaration: Value = serde_json::from_slice(&fixture.manifest).unwrap();
        declaration["mapping_collections"] =
            json!([{"artifact":"mapping-0.json","framework_role":"target"}]);
        fixture.manifest = bytes(&declaration);
        std::fs::remove_file(fixture.directory.path().join("policy.json")).unwrap();
        std::fs::remove_file(fixture.directory.path().join("mapping-build.json")).unwrap();
        let full = fixture.compare(ImpactFilters::default());
        assert_eq!(full.summary.content_changed, ids.len());
        assert_eq!(full.summary.findings, 2 * ids.len());
        assert!(!fixture.directory.path().join("policy.json").exists());
    }

    #[test]
    /// Preserve genuine native error details and actual success/failure post-phase interruption.
    fn native_failure_details_and_success_failure_post_phase_stops_are_preserved() {
        let mut fixture = basic();
        let mut declaration: Value = serde_json::from_slice(&fixture.manifest).unwrap();
        declaration["old"]["expected_sha256"] = json!("0".repeat(64));
        fixture.manifest = bytes(&declaration);
        fixture.install();
        let parsed = crate::framework::manifest::parse(&fixture.manifest).unwrap();
        let legacy = crate::framework::analysis::analyze(
            fixture.directory.path(),
            &parsed,
            ImpactFilters::default(),
        )
        .unwrap_err();
        let borrowed = prepare(
            fixture.inputs(&[], &[]),
            ImpactFilters::default(),
            &mut NoopControl,
            &mut |_| Ok::<(), ()>(()),
        );
        assert!(
            matches!(borrowed,Err(LegacyImpactError::Domain(error)) if error.to_string()==legacy.to_string())
        );
        // Observe the actual malformed native phase's post-failure boundary,
        // then select the same boundary for success/failure originals of one chunk.
        let mut malformed = basic();
        malformed.manifest = b"{".to_vec();
        let mut observed = Recorder::default();
        let failed = prepare(
            malformed.inputs(&[], &[]),
            ImpactFilters::default(),
            &mut observed,
            &mut |_| Ok::<(), ()>(()),
        );
        assert!(matches!(failed, Err(LegacyImpactError::Domain(_))));
        let post_visit = observed.events.len();
        for invalid in [false, true] {
            let mut fixture = basic();
            if invalid {
                fixture.manifest = b"{".to_vec();
            }
            let mut control = Recorder::at(Stage::PrepareDomain, post_visit);
            let result = prepare(
                fixture.inputs(&[], &[]),
                ImpactFilters::default(),
                &mut control,
                &mut |_| Ok::<(), ()>(()),
            );
            assert!(matches!(
                result,
                Err(LegacyImpactError::Work(WorkError::Interrupted(Interruption::CancelRequested)))
            ));
            assert_eq!(control.interruption(), Some(Interruption::CancelRequested));
        }
        let fixture = basic();
        let admission = prepare(
            fixture.inputs(&[], &[]),
            ImpactFilters::default(),
            &mut NoopControl,
            &mut |_| Err::<(), _>("original admission stop"),
        );
        assert!(matches!(admission, Err(LegacyImpactError::Admission("original admission stop"))));
    }
    /// Controls exercise the genuine shared native builder; no capture proof is minted.
    mod staged_report_admission_controls {
        use super::*;
        use crate::framework::analysis::{ReportBuildError, ReportGrowth};

        /// Load actual original bytes through the same maintained native resource helper.
        fn originals(fixture: &Fixture) -> (LoadedResource, LoadedResource) {
            let manifest = crate::framework::manifest::parse(&fixture.manifest).unwrap();
            let old = load_framework(
                fixture.directory.path(),
                "$.old",
                &manifest.old,
                &FrameworkBytes {
                    artifact: &fixture.old,
                    resolved_catalog: fixture.old_companion.as_deref(),
                },
                &mut NoopControl,
                &mut |_| Ok::<(), ()>(()),
            )
            .unwrap();
            let new = load_framework(
                fixture.directory.path(),
                "$.new",
                &manifest.new,
                &FrameworkBytes {
                    artifact: &fixture.new,
                    resolved_catalog: fixture.new_companion.as_deref(),
                },
                &mut NoopControl,
                &mut |_| Ok::<(), ()>(()),
            )
            .unwrap();
            (old, new)
        }

        /// Build real declared Mapping references and successor data before the staged builder.
        fn native_report<E>(
            fixture: &Fixture,
            admit: &mut impl FnMut(ReportGrowth<'_>) -> Result<(), E>,
        ) -> Result<ImpactReport, ReportBuildError<E>> {
            let manifest = crate::framework::manifest::parse(&fixture.manifest).unwrap();
            let (old, new) = originals(fixture);
            let mut portfolio = MappingPortfolio::default();
            let mut collections = BTreeSet::new();
            for (index, (dependency, raw)) in
                manifest.mapping_collections.iter().zip(&fixture.mappings).enumerate()
            {
                let label = format!("$.mapping_collections[{index}]");
                let collection =
                    crate::framework::analysis::decode_mapping_bytes(&label, raw).unwrap();
                crate::framework::analysis::record_mapping_collection(
                    &label,
                    &collection,
                    sha256_hex(raw),
                    dependency.framework_role,
                    &old,
                    &mut collections,
                    &mut portfolio,
                )
                .unwrap();
            }
            let successor = fixture
                .successor
                .as_deref()
                .map(|raw| crate::migration::parse_successor(raw).unwrap());
            let mut report = crate::framework::analysis::build_report_admitted(
                old,
                new,
                &portfolio,
                None,
                successor.as_ref(),
                ImpactFilters::default(),
                admit,
            )?;
            crate::framework::analysis::finish_report(&mut report);
            Ok(report)
        }

        /// Observe only actual private stage reach, without implying source completeness.
        fn stage_number(stage: &ReportGrowth<'_>) -> usize {
            match stage {
                ReportGrowth::Classification { .. } => 0,
                ReportGrowth::Findings(_) => 1,
                ReportGrowth::FrameworkGroups { .. } => 2,
                ReportGrowth::FindingSort { .. } => 3,
            }
        }

        #[test]
        /// Preserve all complete native fields/counts and stop before each subsequent constructor.
        fn ordinary_parity_and_first_original_admission_stop_at_every_native_stage() {
            let fixture = basic();
            let expected = fixture.compare(ImpactFilters::default());
            let mut reached = Vec::new();
            let actual = native_report(&fixture, &mut |stage| {
                reached.push(stage_number(&stage));
                admit_report_growth(stage, &mut NoopControl, &mut |_| Ok::<(), &'static str>(()))
            })
            .unwrap_or_else(|_| panic!("genuine admitted native report"));
            assert_eq!(reached, vec![0, 1, 2, 3]);
            assert_eq!(
                serde_json::to_value(&actual).unwrap(),
                serde_json::to_value(&expected).unwrap()
            );
            assert_eq!(actual.summary.rows(), expected.summary.rows());
            for selected in 0..4 {
                let mut reached = Vec::new();
                let result = native_report(&fixture, &mut |stage| {
                    let number = stage_number(&stage);
                    reached.push(number);
                    if number == selected {
                        Err("original stage admission refusal")
                    } else {
                        Ok(())
                    }
                });
                assert!(matches!(
                    result,
                    Err(ReportBuildError::Admission("original stage admission refusal"))
                ));
                assert_eq!(reached, (0..=selected).collect::<Vec<_>>());
            }
        }

        #[test]
        /// Native invalid migration refuses after classification admission and before findings admission.
        fn genuine_native_domain_failure_keeps_full_detail_and_later_stages_absent() {
            let mut fixture = basic();
            fixture.successor = Some(bytes(
                &json!({"schema_version":"forge.successor-map/1","relationships":[{
                "relationship":"successor","old_ids":["absent-old"],"new_ids":["added"],
                "approved_by":"Synthetic Reviewer","approved_at":"2026-08-25T12:00:00Z","rationale":"Synthetic invalid old subject."}]}),
            ));
            let mut manifest: Value = serde_json::from_slice(&fixture.manifest).unwrap();
            manifest["successor_map"] = json!("successor.json");
            fixture.manifest = bytes(&manifest);
            fixture.install();
            let declaration = crate::framework::manifest::parse(&fixture.manifest).unwrap();
            let expected = crate::framework::analysis::analyze(
                fixture.directory.path(),
                &declaration,
                ImpactFilters::default(),
            )
            .unwrap_err();
            let mut reached = Vec::new();
            let actual = native_report(&fixture, &mut |stage| {
                reached.push(stage_number(&stage));
                admit_report_growth(stage, &mut NoopControl, &mut |_| Ok::<(), ()>(()))
            });
            assert!(
                matches!(actual, Err(ReportBuildError::Native(error)) if error.to_string() == expected.to_string())
            );
            assert_eq!(reached, vec![0]);
        }

        /// Genuine native Mapping with full policy href preserved in each dependency finding.
        fn mapping_fixture(href: &str) -> Fixture {
            let mut fixture = basic();
            let raw = native_mapping_controls(
                &fixture,
                "staged-policy-native",
                true,
                href,
                &["changed", "removed"],
            );
            fixture.mappings.push(raw);
            let mut manifest: Value = serde_json::from_slice(&fixture.manifest).unwrap();
            manifest["mapping_collections"] =
                json!([{"artifact":"mapping-0.json","framework_role":"target"}]);
            fixture.manifest = bytes(&manifest);
            fixture
        }

        /// Collect the complete real findings-stage reservation while retaining native output.
        fn finding_extent(fixture: &Fixture) -> (ImpactReport, usize) {
            let mut extent = 0usize;
            let report = native_report(fixture, &mut |stage| {
                let selected = stage_number(&stage) == 1;
                admit_report_growth(stage, &mut NoopControl, &mut |charge| {
                    if selected && let ImpactCharge::Bytes(amount) = charge {
                        extent += amount;
                    }
                    Ok::<(), ()>(())
                })
            })
            .unwrap_or_else(|_| panic!("genuine complete native report"));
            (report, extent)
        }

        #[test]
        /// Increasing full private hrefs increases complete admission and denial precedes row allocation.
        fn private_mapping_href_payload_is_never_clipped_or_omitted_from_admission() {
            let short = mapping_fixture("policy.json");
            let long_href = format!("{}.json", "p".repeat(8192));
            let long = mapping_fixture(&long_href);
            let (short_report, short_extent) = finding_extent(&short);
            let (long_report, long_extent) = finding_extent(&long);
            let dependencies = long_report
                .findings
                .iter()
                .filter(|row| row.affected_artifact_id.is_some())
                .count();
            assert_eq!(dependencies, 2);
            assert_eq!(short_report.findings.len(), long_report.findings.len());
            assert!(
                long_report
                    .findings
                    .iter()
                    .filter(|row| row.affected_artifact_id.is_some())
                    .all(|row| row.policy_sources == vec![long_href.clone()])
            );
            assert!(
                long_extent - short_extent
                    >= dependencies * (long_href.len() - "policy.json".len())
            );
            let mut reached = Vec::new();
            let denied = native_report(&long, &mut |stage| {
                let selected = stage_number(&stage) == 1;
                reached.push(stage_number(&stage));
                admit_report_growth(stage, &mut NoopControl, &mut |charge| {
                    if selected && matches!(charge, ImpactCharge::Bytes(_)) {
                        Err("full private finding bytes refused")
                    } else {
                        Ok(())
                    }
                })
            });
            assert!(matches!(
                denied,
                Err(ReportBuildError::Admission(LegacyImpactError::Admission(
                    "full private finding bytes refused"
                )))
            ));
            assert_eq!(reached, vec![0, 1]);
        }

        /// Real split declaration and one actual native Mapping retain whole subject vectors.
        fn split_fixture(rationale: &str) -> Fixture {
            let mut fixture = basic();
            fixture.old =
                bytes(&catalog(FRAMEWORK, "1.0.0", &[("parent", "Original native requirement")]));
            fixture.new = bytes(&catalog(
                FRAMEWORK,
                "2.0.0",
                &[
                    ("child-a", "First native requirement"),
                    ("child-b", "Second native requirement"),
                ],
            ));
            fixture.manifest = bytes(
                &json!({"schema_version":"forge.framework-impact/1", "old":resource("old.json", &fixture.old,"1.0.0"), "new":resource("new.json",&fixture.new,"2.0.0"),"mapping_collections":[]}),
            );
            let mapping = native_mapping_controls(
                &fixture,
                "staged-split-native",
                true,
                "policy.json",
                &["parent"],
            );
            fixture.mappings.push(mapping);
            fixture.successor = Some(bytes(
                &json!({"schema_version":"forge.successor-map/1","relationships":[{
                "relationship":"split","old_ids":["parent"],"new_ids":["child-a","child-b"],
                "approved_by":"Synthetic Reviewer","approved_at":"2026-08-25T12:00:00Z","rationale":rationale}]}),
            ));
            let mut manifest: Value = serde_json::from_slice(&fixture.manifest).unwrap();
            manifest["mapping_collections"] =
                json!([{"artifact":"mapping-0.json","framework_role":"target"}]);
            manifest["successor_map"] = json!("successor.json");
            fixture.manifest = bytes(&manifest);
            fixture
        }

        #[test]
        /// Each native split finding admits the entire relationship and full rationale, including seeds.
        fn complete_split_arrays_and_repeated_private_migration_rationale_are_admitted() {
            let short = split_fixture("Synthetic split declaration.");
            let long_rationale = "Synthetic private migration rationale. ".repeat(256);
            let long = split_fixture(&long_rationale);
            let (_, short_extent) = finding_extent(&short);
            let (report, long_extent) = finding_extent(&long);
            assert_eq!(report.summary.identity_migrated, 1);
            assert_eq!(report.findings.len(), 2);
            for finding in &report.findings {
                assert_eq!(finding.old_subjects.len(), 1);
                assert_eq!(finding.new_subjects.len(), 2);
                assert_eq!(finding.migration.as_ref().unwrap().rationale, long_rationale);
            }
            let delta = long_rationale.len() - "Synthetic split declaration.".len();
            assert!(long_extent - short_extent >= report.findings.len() * delta);
            long.compare(ImpactFilters::default());
        }

        #[test]
        /// Repeated old/new group attempts cannot use final set deduplication as admission credit.
        fn all_duplicate_ancestry_attempts_are_charged_before_native_group_growth() {
            let fixture = basic();
            let (old, new) = originals(&fixture);
            let mut reservation = 0usize;
            let mut attempted = 0usize;
            let report = native_report(&fixture, &mut |stage| {
                if let ReportGrowth::FrameworkGroups { findings, .. } = &stage {
                    assert!(findings.iter().all(|row| row.framework_groups.is_empty()));
                    for row in *findings {
                        for subject in row.old_subjects.iter().chain(&row.new_subjects) {
                            attempted += old.inventory.groups_for_control(&subject.id).len();
                            attempted += new.inventory.groups_for_control(&subject.id).len();
                        }
                    }
                }
                let selected = stage_number(&stage) == 2;
                admit_report_growth(stage, &mut NoopControl, &mut |charge| {
                    if selected && let ImpactCharge::Bytes(amount) = charge {
                        reservation += amount;
                    }
                    Ok::<(), ()>(())
                })
            })
            .unwrap_or_else(|_| panic!("genuine group producer"));
            let final_groups: usize =
                report.findings.iter().map(|row| row.framework_groups.len()).sum();
            assert!(attempted > final_groups);
            assert!(reservation >= attempted * "group-1".len());
            assert!(
                report
                    .findings
                    .iter()
                    .filter(|row| !row.old_subjects.is_empty() || !row.new_subjects.is_empty())
                    .all(|row| row.framework_groups == vec!["group-1"])
            );
        }

        #[test]
        /// Actual native private evidence strings remain fully charged in the history comparison.
        fn history_evidence_serialization_is_complete_and_first_admission_refusal_is_retained() {
            let mut fixture = basic();
            let private_version = format!("1.0.0 {}", "private-version".repeat(128));
            let mut old: Value = serde_json::from_slice(&fixture.old).unwrap();
            old["catalog"]["metadata"]["version"] = json!(private_version);
            fixture.old = bytes(&old);
            let mut manifest: Value = serde_json::from_slice(&fixture.manifest).unwrap();
            manifest["old"] = resource("old.json", &fixture.old, &private_version);
            fixture.manifest = bytes(&manifest);
            let report = fixture.compare(ImpactFilters::default());
            let original = serde_json::to_value(&report).unwrap();
            let prior = serde_json::to_vec(&report).unwrap();
            let records = bytes(&json!({"schema_version":"forge.framework-impact-dispositions/1",
                "prior_report_sha256":sha256_hex(&prior),"dispositions":report.findings.iter().take(2).map(|finding|
                    json!({"finding_id":finding.finding_id,"status":"resolved","decided_by":"Synthetic Reviewer",
                        "decided_at":"2026-08-25T12:00:00Z","rationale":"Synthetic exact-pair history assertion."})).collect::<Vec<_>>()}));
            let parsed = crate::framework::disposition::parse(&records).unwrap();
            assert_eq!(parsed.dispositions.len(), 2);
            let prior_slots =
                preparse_slots(&prior, &mut NoopControl, &mut |_| Ok::<(), ()>(())).unwrap();
            let mut extent = 0usize;
            admit_history_report(
                &report,
                &parsed.dispositions,
                prior.len(),
                prior_slots,
                &mut NoopControl,
                &mut |charge| {
                    if let ImpactCharge::Bytes(amount) = charge {
                        extent += amount;
                    }
                    Ok::<(), ()>(())
                },
            )
            .unwrap();
            assert!(
                extent >= report.old.document_version.len() + report.new.document_version.len()
            );
            let denied = admit_history_report(
                &report,
                &parsed.dispositions,
                prior.len(),
                prior_slots,
                &mut NoopControl,
                &mut |charge| {
                    if matches!(charge, ImpactCharge::Bytes(_)) {
                        Err("original history reservation stop")
                    } else {
                        Ok(())
                    }
                },
            );
            assert!(matches!(
                denied,
                Err(LegacyImpactError::Admission("original history reservation stop"))
            ));
            assert_eq!(serde_json::to_value(&report).unwrap(), original);
        }

        /// Retain the builder's native and original admission error variants for the real phase.
        fn phase_error<E>(error: ReportBuildError<LegacyImpactError<E>>) -> LegacyImpactError<E> {
            match error {
                ReportBuildError::Native(error) => LegacyImpactError::Domain(error),
                ReportBuildError::Admission(error) => error,
            }
        }

        #[test]
        /// Fence real native success/Domain failure, retaining a first caller refusal without a new fence.
        fn staged_native_post_phase_fences_and_original_first_stop_are_preserved() {
            let good = basic();
            let mut bad = basic();
            bad.successor = Some(bytes(
                &json!({"schema_version":"forge.successor-map/1","relationships":[{
                "relationship":"successor","old_ids":["absent-old"],"new_ids":["added"],
                "approved_by":"Synthetic Reviewer","approved_at":"2026-08-25T12:00:00Z","rationale":"Synthetic invalid old subject."}]}),
            ));
            let mut manifest: Value = serde_json::from_slice(&bad.manifest).unwrap();
            manifest["successor_map"] = json!("successor.json");
            bad.manifest = bytes(&manifest);
            for (fixture, domain) in [(&good, false), (&bad, true)] {
                let mut observed = Recorder::default();
                let result =
                    phase(&mut observed, &mut |_| Ok::<(), &'static str>(()), |control, admit| {
                        native_report(fixture, &mut |stage| {
                            admit_report_growth(stage, control, admit)
                        })
                        .map_err(phase_error)
                    });
                assert_eq!(matches!(result, Err(LegacyImpactError::Domain(_))), domain);
                let last_actual_checkpoint = observed.events.len();
                let mut stopped = Recorder::at(Stage::PrepareDomain, last_actual_checkpoint);
                let result =
                    phase(&mut stopped, &mut |_| Ok::<(), &'static str>(()), |control, admit| {
                        native_report(fixture, &mut |stage| {
                            admit_report_growth(stage, control, admit)
                        })
                        .map_err(phase_error)
                    });
                assert!(matches!(
                    result,
                    Err(LegacyImpactError::Work(WorkError::Interrupted(
                        Interruption::CancelRequested
                    )))
                ));
                assert_eq!(stopped.events.len(), last_actual_checkpoint);
            }
            let mut observed = Recorder::default();
            let first = phase(
                &mut observed,
                &mut |_| Err::<(), _>("original caller admission stop"),
                |control, admit| {
                    native_report(&good, &mut |stage| admit_report_growth(stage, control, admit))
                        .map_err(phase_error)
                },
            );
            assert!(matches!(
                first,
                Err(LegacyImpactError::Admission("original caller admission stop"))
            ));
            let reached = observed.events.len();
            let mut stopped = Recorder::at(Stage::PrepareDomain, reached + 1);
            let first = phase(
                &mut stopped,
                &mut |_| Err::<(), _>("original caller admission stop"),
                |control, admit| {
                    native_report(&good, &mut |stage| admit_report_growth(stage, control, admit))
                        .map_err(phase_error)
                },
            );
            assert!(matches!(
                first,
                Err(LegacyImpactError::Admission("original caller admission stop"))
            ));
            assert_eq!(stopped.events.len(), reached);
            assert_eq!(stopped.interruption(), None);
        }

        #[test]
        /// Actual unchanged native old subjects still perform Mapping lookups without producing rows.
        fn unchanged_old_subject_mapping_lookups_keep_repeated_work_without_finding_growth() {
            let mut fixture = mapping_fixture("policy.json");
            fixture.new = bytes(&catalog(
                FRAMEWORK,
                "2.0.0",
                &[("unchanged", "Same"), ("changed", "Old"), ("removed", "Removed")],
            ));
            let mut manifest: Value = serde_json::from_slice(&fixture.manifest).unwrap();
            manifest["new"] = resource("new.json", &fixture.new, "2.0.0");
            fixture.manifest = bytes(&manifest);
            let mut expected_lookups = 0usize;
            let mut admitted_matching = 0usize;
            let report = native_report(&fixture, &mut |stage| {
                if let ReportGrowth::Findings(inputs) = &stage {
                    assert!(inputs.changes.iter().all(|change| change.change_class
                        == crate::framework::model::ChangeClass::Unchanged));
                    assert_eq!(inputs.portfolio.references.len(), 2);
                    expected_lookups = inputs.old.inventory.count(SubjectType::Control)
                        * inputs.portfolio.references.len();
                }
                let selected = stage_number(&stage) == 1;
                admit_report_growth(stage, &mut NoopControl, &mut |charge| {
                    if selected && let ImpactCharge::Work { comparisons, .. } = charge {
                        admitted_matching += comparisons;
                    }
                    Ok::<(), ()>(())
                })
            })
            .unwrap_or_else(|_| panic!("genuine unchanged native controls"));
            assert_eq!(report.summary.content_changed, 0);
            assert_eq!(report.summary.findings, 1);
            assert_eq!(
                report.findings[0].reason_code,
                crate::framework::model::ReasonCode::ResourceMetadataChanged
            );
            assert!(report.findings[0].affected_artifact_id.is_none());
            assert!(admitted_matching >= expected_lookups);
            assert_eq!(expected_lookups, 6);
            fixture.compare(ImpactFilters::default());
        }
    }
}
