//! Complete admitted capture geometry and native tuple projections for the real receiver.
//! This child has no owner constructor, filesystem reopening or subset-success capability.
use super::{
    AuthoringMember, AuthoringPurpose, CaptureLease, ContractError, ContractLedger, NativeModelV3,
    Path, PathBuf, PreparedAuthoringFacts, ReviewCapture, SourceKindV3, SourcePinV3, WorkControl,
    checkpoint, native, phase, sha256_hex,
};

/// Join actual review-root parent to a validated native project label using portable slash spelling.
pub(super) fn join(
    parent: &Path,
    relative: &Path,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PathBuf, ContractError> {
    phase(ledger, control, |ledger, _| {
        let left = parent.to_str().ok_or(ContractError::Invalid)?;
        let right = relative.to_str().ok_or(ContractError::Invalid)?;
        let extent = left
            .len()
            .checked_add(right.len())
            .and_then(|n| n.checked_add(1))
            .ok_or_else(|| ledger.capacity())?;
        ledger.bytes(extent)?;
        ledger.visits(parent.components().count() + relative.components().count() + 1)?;
        crate::linkage::fresh::validate_relative(relative).map_err(|_| ContractError::Invalid)?;
        ledger.derived(
            extent.checked_add(std::mem::size_of::<PathBuf>()).ok_or(ContractError::Capacity)?,
        )?;
        let joined = if left.is_empty() {
            PathBuf::from(right)
        } else {
            PathBuf::from(format!("{left}/{right}"))
        };
        crate::linkage::fresh::validate_relative(&joined).map_err(|_| ContractError::Invalid)?;
        Ok(joined)
    })
}
/// Copy one full already-admitted private string before retaining it in the receiver roster.
fn copy(value: &str, ledger: &mut ContractLedger) -> Result<String, ContractError> {
    ledger.bytes(value.len())?;
    let extent =
        value.len().checked_add(std::mem::size_of::<String>()).ok_or_else(|| ledger.capacity())?;
    ledger.derived(extent)?;
    Ok(value.to_owned())
}
/// Borrowed declarations for one complete actual native capture occurrence.
/// This inert operand carrier cannot select registrations or issue an owner.
#[derive(Clone, Copy)]
pub(super) struct SourceOccurrence<'a> {
    /// Exact actual physical route under the retained review root.
    pub(super) path: &'a Path,
    /// Complete native project-relative label, absent for stored-plan S.
    pub(super) native_path: Option<&'a Path>,
    /// Complete actual native purpose, including the declaration ordinal.
    pub(super) purpose: AuthoringPurpose,
    /// Exact native role label, absent together with the native path.
    pub(super) native_role: Option<&'a str>,
    /// Existing unchanged complete raw admission cap for this occurrence.
    pub(super) maximum: usize,
}
/// Capture a complete actual declared occurrence and its raw identity before any later native call.
pub(super) fn capture(
    capture: &mut ReviewCapture,
    occurrence: SourceOccurrence<'_>,
    members: &mut Vec<AuthoringMember>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let SourceOccurrence { path, native_path, purpose, native_role, maximum } = occurrence;
    phase(ledger, control, |ledger, control| {
        let label = native_path.map(|p| p.to_str().ok_or(ContractError::Invalid)).transpose()?;
        if label.is_some() != native_role.is_some() {
            return Err(ContractError::Binding);
        }
        if let Some(label) = label {
            ledger.bytes(label.len())?;
            ledger.visits(1)?;
            for row in &*members {
                checkpoint(ledger, control)?;
                ledger.visits(1)?;
                if let Some(prior) = &row.native_path {
                    ledger.bytes(
                        prior.len().checked_add(label.len()).ok_or(ContractError::Capacity)?,
                    )?;
                    if prior.eq_ignore_ascii_case(label) {
                        return Err(ContractError::Binding);
                    }
                }
            }
        }
        let ordinary = capture.required_authoring_source(path, purpose, maximum, ledger, control);
        checkpoint(ledger, control)?;
        let index = ordinary?;
        ledger.derived(std::mem::size_of::<AuthoringMember>())?;
        if members.len() == members.capacity() {
            ledger.bytes(
                members
                    .len()
                    .checked_mul(std::mem::size_of::<AuthoringMember>())
                    .ok_or(ContractError::Capacity)?,
            )?;
        }
        members.try_reserve(1).map_err(|_| ledger.capacity())?;
        ledger.derived(std::mem::size_of::<CaptureLease>())?;
        let lease = capture.lease(index)?;
        ledger.bytes(lease.bytes().len())?;
        ledger.derived(64)?;
        let raw_sha256 = sha256_hex(lease.bytes());
        checkpoint(ledger, control)?;
        let path_extent = path.as_os_str().len();
        ledger.bytes(path_extent)?;
        ledger.derived(
            path_extent
                .checked_add(std::mem::size_of::<PathBuf>())
                .ok_or(ContractError::Capacity)?,
        )?;
        let row = AuthoringMember {
            index,
            lease,
            purpose,
            path: path.to_path_buf(),
            native_path: label.map(|v| copy(v, ledger)).transpose()?,
            native_role: native_role.map(|v| copy(v, ledger)).transpose()?,
            raw_sha256,
        };
        members.push(row);
        Ok(())
    })
}
/// Borrow one complete actual native original by receiver-derived closed purpose.
pub(super) fn raw<'a>(
    members: &'a [AuthoringMember],
    purpose: AuthoringPurpose,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<&'a [u8], ContractError> {
    phase(ledger, control, |ledger, _| {
        ledger.visits(members.len())?;
        let row = members.iter().find(|m| m.purpose == purpose).ok_or(ContractError::Binding)?;
        Ok(row.lease.bytes())
    })
}
/// Borrow complete ordered Mapping or clause bytes after precharging the actual reference list.
pub(super) fn raws<'a>(
    members: &'a [AuthoringMember],
    mapping: bool,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<&'a [u8]>, ContractError> {
    phase(ledger, control, |ledger, control| {
        ledger.visits(members.len())?;
        let size = members
            .len()
            .checked_mul(std::mem::size_of::<&[u8]>())
            .ok_or_else(|| ledger.capacity())?;
        ledger.derived(size)?;
        let mut raws = Vec::new();
        raws.try_reserve_exact(members.len()).map_err(|_| ledger.capacity())?;
        for member in members {
            checkpoint(ledger, control)?;
            ledger.visits(1)?;
            if (mapping && matches!(member.purpose, AuthoringPurpose::Mapping(_)))
                || (!mapping && matches!(member.purpose, AuthoringPurpose::Clause(_)))
            {
                raws.push(member.lease.bytes());
            }
        }
        Ok(raws)
    })
}
/// Bind every full native fingerprint to actual N, independently of native/public sort orders.
pub(super) fn provenance(
    members: &[AuthoringMember],
    facts: &PreparedAuthoringFacts,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        if facts.loaded.inputs.len().checked_add(1).ok_or_else(|| ledger.capacity())?
            != members.len()
            || facts.plan.provenance.inputs.len() != facts.loaded.inputs.len()
        {
            return Err(ContractError::Binding);
        }
        for expected in [&facts.loaded.inputs, &facts.plan.provenance.inputs] {
            for row in expected {
                checkpoint(ledger, control)?;
                let mut matches = 0usize;
                for actual in members {
                    ledger.visits(1)?;
                    let text = row
                        .role
                        .len()
                        .checked_add(row.path.len())
                        .and_then(|n| n.checked_add(row.sha256.len()))
                        .and_then(|n| {
                            n.checked_add(actual.native_role.as_ref().map_or(0, String::len))
                        })
                        .and_then(|n| {
                            n.checked_add(actual.native_path.as_ref().map_or(0, String::len))
                        })
                        .and_then(|n| n.checked_add(actual.raw_sha256.len()))
                        .ok_or_else(|| ledger.capacity())?;
                    ledger.bytes(text)?;
                    ledger.matching(1)?;
                    if actual.native_role.as_deref() == Some(row.role.as_str())
                        && actual.native_path.as_deref() == Some(row.path.as_str())
                        && row.sha256 == actual.raw_sha256
                        && row.byte_length == actual.lease.bytes().len() as u64
                    {
                        matches += 1;
                    }
                }
                if matches != 1 {
                    return Err(ContractError::Binding);
                }
            }
        }
        Ok(())
    })
}
/// Exact intrinsic/public profile selection; native labels are verified only after real preparation.
fn profile(
    purpose: AuthoringPurpose,
) -> (&'static str, SourceKindV3, Option<&'static str>, &'static str) {
    match purpose {
        AuthoringPurpose::Project => (
            "authoring:project",
            SourceKindV3::AuthorProject,
            Some("forge.author-project/1"),
            "forge.author-project-intrinsic/1",
        ),
        AuthoringPurpose::Pack => (
            "authoring:pack",
            SourceKindV3::AuthoringPack,
            Some("forge.authoring-pack/1"),
            "forge.authoring-pack-intrinsic/1",
        ),
        AuthoringPurpose::GapReport => (
            "authoring:gap-report",
            SourceKindV3::GapReport,
            Some("forge.applicability-report/1"),
            "forge.authoring-gap-report-complete/1",
        ),
        AuthoringPurpose::Applicability => (
            "authoring:applicability",
            SourceKindV3::ApplicabilityManifest,
            Some("forge.applicability/1"),
            "forge.authoring-applicability-intrinsic/1",
        ),
        AuthoringPurpose::Framework => (
            "authoring:framework",
            SourceKindV3::Framework,
            None,
            "forge.authoring-framework-native/1",
        ),
        AuthoringPurpose::Resolved => (
            "authoring:resolved",
            SourceKindV3::ResolvedCatalog,
            None,
            "forge.authoring-resolved-catalog-native/1",
        ),
        AuthoringPurpose::Mapping(_) => (
            "authoring:mapping:",
            SourceKindV3::MappingCollection,
            None,
            "forge.authoring-mapping-native/1",
        ),
        AuthoringPurpose::Clause(_) => (
            "authoring:clause:",
            SourceKindV3::HumanClause,
            None,
            "forge.authoring-clause-validated/1",
        ),
        AuthoringPurpose::StoredPlan => (
            "authoring:plan",
            SourceKindV3::StoredPlan,
            Some("forge.authoring-plan/1"),
            "forge.authoring-plan-complete-equality/1",
        ),
    }
}
/// Build all eight exact tuple fields from genuine actual originals and full validated native data.
pub(super) fn pins(
    members: &[AuthoringMember],
    facts: &PreparedAuthoringFacts,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<SourcePinV3>, ContractError> {
    phase(ledger, control, |ledger, control| {
        let size = members
            .len()
            .checked_mul(std::mem::size_of::<SourcePinV3>())
            .ok_or_else(|| ledger.capacity())?;
        ledger.derived(size)?;
        let mut pins = Vec::new();
        pins.try_reserve_exact(members.len()).map_err(|_| ledger.capacity())?;
        for row in members {
            checkpoint(ledger, control)?;
            ledger.visits(1)?;
            let (prefix, kind, schema, validation) = profile(row.purpose);
            let key = match row.purpose {
                AuthoringPurpose::Mapping(n) | AuthoringPurpose::Clause(n) => {
                    ledger.derived(prefix.len() + 32)?;
                    ledger.bytes(prefix.len())?;
                    format!("{prefix}{n}")
                }
                _ => copy(prefix, ledger)?,
            };
            let model = match row.purpose {
                AuthoringPurpose::Framework => {
                    Some(match facts.loaded.baseline_report.framework.resource_type {
                        crate::mapping::manifest::ResourceType::Catalog => NativeModelV3::Catalog,
                        crate::mapping::manifest::ResourceType::Profile => NativeModelV3::Profile,
                    })
                }
                AuthoringPurpose::Resolved => Some(NativeModelV3::Catalog),
                AuthoringPurpose::Mapping(_) => Some(NativeModelV3::MappingCollection),
                _ => None,
            };
            let root = match model {
                Some(model) => {
                    Some(native::root(row.lease.bytes(), model.as_str(), ledger, control)?)
                }
                None => None,
            };
            if row.purpose == AuthoringPurpose::Framework {
                let expected = &facts.loaded.baseline_report.framework;
                let comparison = expected
                    .root_uuid
                    .len()
                    .checked_add(root.as_ref().map_or(0, String::len))
                    .and_then(|n| n.checked_add(expected.raw_sha256.len()))
                    .and_then(|n| n.checked_add(row.raw_sha256.len()))
                    .ok_or_else(|| ledger.capacity())?;
                ledger.bytes(comparison)?;
                if root.as_deref() != Some(expected.root_uuid.as_str())
                    || expected.raw_sha256 != row.raw_sha256
                {
                    return Err(ContractError::Binding);
                }
            }
            if row.purpose == AuthoringPurpose::Resolved {
                let expected =
                    facts.loaded.baseline_report.framework.resolved_catalog_sha256.as_deref();
                let comparison = expected
                    .map_or(0, str::len)
                    .checked_add(row.raw_sha256.len())
                    .ok_or_else(|| ledger.capacity())?;
                ledger.bytes(comparison)?;
                if expected != Some(row.raw_sha256.as_str()) {
                    return Err(ContractError::Binding);
                }
            }
            pins.push(SourcePinV3 {
                artifact_key: key,
                kind,
                raw_sha256: copy(&row.raw_sha256, ledger)?,
                byte_length: row.lease.bytes().len() as u64,
                schema_identity: schema.map(|s| copy(s, ledger)).transpose()?,
                validation_profile: copy(validation, ledger)?,
                native_model: model,
                native_root_uuid: root,
            });
        }
        sort_pins(&mut pins, ledger, control)?;
        // The real complete native preparation chooses exactly Catalog or Profile companion state.
        ledger.visits(members.len())?;
        let profile = facts.loaded.baseline_report.framework.resource_type
            == crate::mapping::manifest::ResourceType::Profile;
        if members.iter().any(|m| m.purpose == AuthoringPurpose::Resolved) != profile {
            return Err(ContractError::Binding);
        }
        Ok(pins)
    })
}

/// Sort every admitted typed pin while preserving all original comparison/move charges.
fn sort_pins(
    pins: &mut [SourcePinV3],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    // Bounded insertion sort charges each exact string comparison and moved typed row.
    for i in 1..pins.len() {
        let mut j = i;
        while j > 0 {
            checkpoint(ledger, control)?;
            ledger.visits(1)?;
            let comparison = pins[j - 1]
                .artifact_key
                .len()
                .checked_add(pins[j].artifact_key.len())
                .ok_or_else(|| ledger.capacity())?;
            ledger.bytes(comparison)?;
            if pins[j - 1].artifact_key < pins[j].artifact_key {
                break;
            }
            if pins[j - 1].artifact_key == pins[j].artifact_key {
                return Err(ContractError::Binding);
            }
            ledger.bytes(2 * std::mem::size_of::<SourcePinV3>())?;
            pins.swap(j - 1, j);
            j -= 1;
        }
    }
    Ok(())
}
