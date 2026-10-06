//! Strict full stored Impact equality as non-authorizing borrowed data.
//!
//! The caller passes its existing ledger/control and complete legacy facts. Equality
//! does not establish original ownership, capture, currentness, native approval,
//! complete read-cohort correspondence or a Source union. No comparison sealer is
//! defined here. The genuine captured factory must separately retain those facts.
//!
//! Native Serialize is replayed directly against a duplicate-safe bounded Value:
//! object key order/JSON whitespace are presentation, but every emitted field,
//! private string, integer and ordered array is exact. No native Value clone,
//! canonical byte buffer, Deserialize-only success or summary-only comparison exists.
//! Logical parser reservations are conservative; they are not allocator confinement.

use std::collections::BTreeSet;
use std::fmt;

use serde::Serialize;
use serde::ser::{Impossible, SerializeSeq, SerializeStruct, SerializeTuple, SerializeTupleStruct};
use serde_json::{Map, Value};

use crate::framework::analysis::legacy_borrowed::LegacyImpactFacts;
use crate::framework::disposition::DispositionStatus;
use crate::framework::model::{
    ChangeClass, ChangeSummary, FindingPriority, ImpactReport, REPORT_SCHEMA_VERSION, ReportStatus,
};
use crate::mapping::inventory::ResourceEvidence;
use crate::mapping::manifest::ResourceType;
use crate::workspace::preparation::WorkControl;

use super::decode::{ContractError, ContractLedger};

/// Same per-Source original ceiling as the proposed genuine S4 capture profile.
const MAX_STORED_BYTES: usize = 10 * 1024 * 1024;
/// Maintained decoded native string limit, including private versions/hrefs/prose.
const MAX_STRING_BYTES: usize = 64 * 1024;
/// Maintained complete native report/finding/disposition ceiling.
const MAX_FINDINGS: usize = 100_000;

/// Plain observation of the complete data result; no identity, lineage or vote authority.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg(test)]
pub(crate) enum NativeChangeObservation {
    /// Four full change counters are zero and complete findings are empty.
    NoDetectedNativeChange,
    /// The exact zero predicate does not hold after complete equality/correlations.
    DetectedNativeChange,
}

#[cfg(test)]
impl NativeChangeObservation {
    /// Return one fixed minimized token; it cannot carry private input prose.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::NoDetectedNativeChange => "no-detected-native-change",
            Self::DetectedNativeChange => "detected-native-change",
        }
    }
}

/// Minimized borrowed native resource data, without any authority or original owner.
#[derive(Serialize)]
#[cfg(test)]
pub(crate) struct ImpactResource<'a> {
    /// Native Catalog/Profile kind; no schema identity is synthesized from prose.
    pub(crate) resource_type: ResourceType,
    /// Exact private native raw digest; this remains potentially sensitive metadata.
    pub(crate) raw_sha256: &'a str,
    /// Exact unnormalized native root UUID under maintained identity semantics.
    pub(crate) root_uuid: &'a str,
    /// Actual fixed native release token, checked before creating this plain projection.
    pub(crate) oscal_version: &'static str,
    /// Exact native Profile companion digest, or explicit null for Catalog.
    pub(crate) resolved_catalog_sha256: Option<&'a str>,
}

/// Full equal native data borrowed from its caller; never a captured comparison capability.
pub(crate) struct ImpactData<'a> {
    /// Complete private data remains borrowed, with no Deserialize/Clone proof constructor.
    #[cfg(test)]
    report: &'a ImpactReport,
    /// Retain the borrowed-input lifetime without a production data/proof accessor.
    _borrow: std::marker::PhantomData<&'a ImpactReport>,
}

#[cfg(test)]
impl ImpactData<'_> {
    /// Borrow complete native rows for a separate genuine factory/lineage consumer.
    pub(crate) fn report(&self) -> &ImpactReport {
        self.report
    }
    /// Borrow all fifteen fully correlated summary cells without minimizing private equality.
    pub(crate) fn summary(&self) -> &ChangeSummary {
        &self.report.summary
    }
    /// Preserve every native change row, including nonempty Unchanged rows.
    pub(crate) fn change_count(&self) -> usize {
        self.report.changes.len()
    }
    /// Preserve the complete unfiltered finding denominator.
    pub(crate) fn finding_count(&self) -> usize {
        self.report.findings.len()
    }
    /// Preserve historical-only dispositions without successor settlement/vote credit.
    pub(crate) fn prior_only_disposition_count(&self) -> usize {
        self.report.prior_only_dispositions.len()
    }
    /// Derive the exact four-counter/complete-findings predicate from equal complete data.
    pub(crate) fn observation(&self) -> NativeChangeObservation {
        let summary = &self.report.summary;
        if summary.added == 0
            && summary.removed == 0
            && summary.content_changed == 0
            && summary.identity_migrated == 0
            && self.report.findings.is_empty()
        {
            NativeChangeObservation::NoDetectedNativeChange
        } else {
            NativeChangeObservation::DetectedNativeChange
        }
    }
    /// Borrow the whitelisted old tuple fields; private version/href stays in full equality only.
    pub(crate) fn old(&self) -> ImpactResource<'_> {
        minimize(&self.report.old)
    }
    /// Borrow the whitelisted new tuple fields after identical private equality requirements.
    pub(crate) fn new_resource(&self) -> ImpactResource<'_> {
        minimize(&self.report.new)
    }
}

/// Compare a supplied complete stored report to complete genuine native legacy data.
///
/// All five native filters must be None and hidden rows empty. The stored native
/// serializer shape therefore contains exactly `filters: {}`; companion filters
/// with five explicit nulls belong to its separate encoder and are not accepted here.
/// Optional fields/empty prior-only rows are omitted exactly as native Serialize
/// emits them. Every object rejects unknown/missing/replacement keys, all private
/// values and ordered arrays compare exactly, and all fifteen summary cells are
/// independently correlated before equality. No href is followed or made identity.
///
/// # Errors
///
/// Syntax/duplicate/BOM/trailing data returns fixed Invalid. Complete shape/data
/// mismatch returns Binding. Actual original-ledger capacity/interruption/failure
/// keeps first-stop precedence through both failure and success post-phase fences.
/// Ordinary report serde, native errors and file behavior are unchanged.
pub(crate) fn compare_stored<'a>(
    stored_raw: &[u8],
    facts: &'a LegacyImpactFacts,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<ImpactData<'a>, ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        let stored = phase(ledger, control, |ledger, control| {
            preparse(stored_raw, ledger, control)?;
            ledger.bytes(stored_raw.len())?;
            crate::json_strict::parse_value(
                stored_raw,
                "stored Impact report",
                crate::json_strict::Limits { max_depth: 64, max_string_bytes: MAX_STRING_BYTES },
            )
            .map_err(|_| ContractError::Invalid)
        })?;
        phase(ledger, control, |ledger, control| correlate(&facts.report, ledger, control))?;
        phase(ledger, control, |ledger, control| {
            let mut state = ComparisonState { ledger, control };
            facts
                .report
                .serialize(CompareSerializer { value: &stored, state: &mut state })
                .map_err(|error| error.0)
        })?;
        ledger.checkpoint(control)?;
        Ok(ImpactData {
            #[cfg(test)]
            report: &facts.report,
            _borrow: std::marker::PhantomData,
        })
    })
}

/// Minimize only fixed native fields; all returned references remain ordinary data.
#[cfg(test)]
fn minimize(evidence: &ResourceEvidence) -> ImpactResource<'_> {
    ImpactResource {
        resource_type: evidence.resource_type,
        raw_sha256: &evidence.raw_sha256,
        root_uuid: &evidence.root_uuid,
        oscal_version: crate::oscal::OSCAL_VERSION,
        resolved_catalog_sha256: evidence.resolved_catalog_sha256.as_deref(),
    }
}

/// Fence native/ordinary success and refusal; a latched first stop prevents another probe.
fn phase<T>(
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
    operation: impl FnOnce(&mut ContractLedger, &mut dyn WorkControl) -> Result<T, ContractError>,
) -> Result<T, ContractError> {
    ledger.checkpoint(control)?;
    let result = operation(ledger, control);
    ledger.checkpoint(control)?;
    result
}

/// Count complete valid-JSON slot/payload ceilings before strict decoder growth.
/// Invalid input gets no syntax credit; the maintained decoder still decides syntax.
fn preparse(
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    if raw.len() > MAX_STORED_BYTES {
        return Err(ledger.capacity());
    }
    if raw.is_empty() || raw.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(ContractError::Invalid);
    }
    let raw_work = mul(raw.len(), 8, ledger)?;
    ledger.bytes(raw_work)?;
    let (mut slots, mut quoted, mut escaped, mut primitive) = (0usize, false, false, false);
    for chunk in raw.chunks(32 * 1024) {
        ledger.checkpoint(control)?;
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
                    slots = add(slots, 1, ledger)?;
                    quoted = true;
                    primitive = false;
                }
                b'{' | b'[' => {
                    slots = add(slots, 1, ledger)?;
                    primitive = false;
                }
                b'}' | b']' | b',' | b':' | b' ' | b'\r' | b'\n' | b'\t' => primitive = false,
                _ if !primitive => {
                    slots = add(slots, 1, ledger)?;
                    primitive = true;
                }
                _ => {}
            }
        }
    }
    let node_bytes = mul(slots, 128, ledger)?;
    let logical_bytes = add(add(node_bytes, raw.len(), ledger)?, 512, ledger)?;
    ledger.derived(logical_bytes)?;
    let tree_work = mul(slots, 4, ledger)?;
    ledger.visits(tree_work)?;
    ledger.matching(tree_work)?;
    Ok(())
}

/// Check every native count from complete ordered rows, with no caller completeness boolean.
fn correlate(
    report: &ImpactReport,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.visits(8)?;
    let schema_extent = add(report.schema_version.len(), REPORT_SCHEMA_VERSION.len(), ledger)?;
    ledger.bytes(schema_extent)?;
    if report.schema_version != REPORT_SCHEMA_VERSION
        || report.status != ReportStatus::Complete
        || !report.filters.is_empty()
        || !report.filtered_out_findings.is_empty()
        || report.matched_findings != report.findings.len()
        || report.findings.len() > MAX_FINDINGS
        || report.prior_only_dispositions.len() > MAX_FINDINGS
    {
        return Err(ContractError::Binding);
    }
    validate_resource(&report.old, ledger)?;
    validate_resource(&report.new, ledger)?;
    if report.old.resource_type != report.new.resource_type {
        return Err(ContractError::Binding);
    }
    let mut expected = ChangeSummary::default();
    ledger.visits(report.changes.len())?;
    for change in &report.changes {
        ledger.checkpoint(control)?;
        expected.old_controls = add(expected.old_controls, change.old_subjects.len(), ledger)?;
        expected.new_controls = add(expected.new_controls, change.new_subjects.len(), ledger)?;
        let count = match change.change_class {
            ChangeClass::Added => &mut expected.added,
            ChangeClass::Removed => &mut expected.removed,
            ChangeClass::ContentChanged => &mut expected.content_changed,
            ChangeClass::IdentityMigrated => &mut expected.identity_migrated,
            ChangeClass::Unchanged => &mut expected.unchanged,
        };
        *count = add(*count, 1, ledger)?;
    }
    if expected.old_controls > crate::mapping::inventory::MAX_INVENTORY_SUBJECTS
        || expected.new_controls > crate::mapping::inventory::MAX_INVENTORY_SUBJECTS
    {
        return Err(ContractError::Binding);
    }
    expected.findings = report.findings.len();
    ledger.visits(report.findings.len())?;
    for finding in &report.findings {
        ledger.checkpoint(control)?;
        let priority_count = match finding.priority {
            FindingPriority::Blocking => &mut expected.blocking,
            FindingPriority::ReviewRequired => &mut expected.review_required,
            FindingPriority::Informational => &mut expected.informational,
        };
        *priority_count = add(*priority_count, 1, ledger)?;
        let disposition_count = match finding.disposition.as_ref().map(|row| row.status) {
            Some(DispositionStatus::Resolved) => &mut expected.dispositioned_resolved,
            Some(DispositionStatus::AcceptedRisk) => &mut expected.dispositioned_accepted_risk,
            Some(DispositionStatus::StillOpen) => &mut expected.dispositioned_still_open,
            None => &mut expected.undispositioned,
        };
        *disposition_count = add(*disposition_count, 1, ledger)?;
        if let Some(row) = &finding.disposition {
            let extent = add(row.finding_id.len(), finding.finding_id.len(), ledger)?;
            ledger.bytes(extent)?;
            ledger.matching(1)?;
            if row.finding_id != finding.finding_id {
                return Err(ContractError::Binding);
            }
        }
    }
    ledger.visits(15)?;
    ledger.bytes(15 * 16)?;
    if report.summary.rows() != expected.rows() {
        return Err(ContractError::Binding);
    }
    unique_ids(report, ledger, control)
}

/// Preserve maintained unnormalized private tuple predicates and finite public OSCAL token.
fn validate_resource(
    evidence: &ResourceEvidence,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    let lengths = [
        &evidence.href,
        &evidence.raw_sha256,
        &evidence.root_uuid,
        &evidence.document_version,
        &evidence.oscal_version,
    ];
    ledger.visits(lengths.len())?;
    for text in lengths {
        let repeated_extent = mul(text.len(), 4, ledger)?;
        ledger.bytes(repeated_extent)?;
    }
    ledger.bytes(crate::oscal::OSCAL_VERSION.len())?;
    if !matches!(evidence.resource_type, ResourceType::Catalog | ResourceType::Profile)
        || evidence.href.trim().is_empty()
        || evidence.document_version.trim().is_empty()
        || evidence.oscal_version != crate::oscal::OSCAL_VERSION
        || uuid::Uuid::parse_str(&evidence.root_uuid).is_err()
        || !lower_hash(&evidence.raw_sha256)
    {
        return Err(ContractError::Binding);
    }
    match (evidence.resource_type, evidence.resolved_catalog_sha256.as_deref()) {
        (ResourceType::Catalog, None) => Ok(()),
        (ResourceType::Profile, Some(hash)) => {
            ledger.bytes(hash.len())?;
            if lower_hash(hash) { Ok(()) } else { Err(ContractError::Binding) }
        }
        _ => Err(ContractError::Binding),
    }
}

/// Validate complete native finding/prior-only identity separation before borrowed set growth.
fn unique_ids(
    report: &ImpactReport,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let total = add(report.findings.len(), report.prior_only_dispositions.len(), ledger)?;
    let registry_bytes = mul(total, 96, ledger)?;
    ledger.derived(registry_bytes)?;
    ledger.visits(total)?;
    let max_id_bytes = report
        .findings
        .iter()
        .map(|row| row.finding_id.len())
        .chain(report.prior_only_dispositions.iter().map(|row| row.finding_id.len()))
        .max()
        .unwrap_or(0);
    let mut ids = BTreeSet::new();
    for id in report
        .findings
        .iter()
        .map(|row| row.finding_id.as_str())
        .chain(report.prior_only_dispositions.iter().map(|row| row.finding_id.as_str()))
    {
        ledger.checkpoint(control)?;
        // Fixed conservative B-tree work ceiling under this finite row/UUID profile;
        // it is neither a measured comparison count nor an allocator heap theorem.
        ledger.matching(128)?;
        let compared_extent = add(id.len(), max_id_bytes, ledger)?;
        let identity_work = mul(compared_extent, 128, ledger)?;
        ledger.bytes(identity_work)?;
        if uuid::Uuid::parse_str(id).is_err() || !ids.insert(id) {
            return Err(ContractError::Binding);
        }
    }
    Ok(())
}

/// Match exact lowercase SHA-256 spelling without normalization or private diagnostics.
fn lower_hash(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Preserve shared sticky capacity on complete checked additions.
fn add(left: usize, right: usize, ledger: &mut ContractLedger) -> Result<usize, ContractError> {
    left.checked_add(right).ok_or_else(|| ledger.capacity())
}
/// Preserve shared sticky capacity on conservative checked multipliers.
fn mul(left: usize, right: usize, ledger: &mut ContractLedger) -> Result<usize, ContractError> {
    left.checked_mul(right).ok_or_else(|| ledger.capacity())
}

/// Fixed serializer failure wrapper; native serializer prose cannot escape this new boundary.
#[derive(Debug)]
struct CompareError(ContractError);
impl fmt::Display for CompareError {
    /// Reuse the fixed existing review error display without raw fields/paths/prose.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}
impl std::error::Error for CompareError {}
impl serde::ser::Error for CompareError {
    /// Unsupported future serialization cannot silently become a partial equal report.
    fn custom<T: fmt::Display>(_message: T) -> Self {
        Self(ContractError::Binding)
    }
}
impl From<ContractError> for CompareError {
    /// Retain the exact actual shared admission/control failure.
    fn from(error: ContractError) -> Self {
        Self(error)
    }
}

/// Borrow the same original ledger/control throughout complete native Serialize replay.
struct ComparisonState<'a> {
    /// Existing invocation work accounting, never a fresh ledger.
    ledger: &'a mut ContractLedger,
    /// Existing accepted cooperative control, never reset or replaced.
    control: &'a mut dyn WorkControl,
}
impl ComparisonState<'_> {
    /// Charge one actual serializer/tree visit before inspecting its corresponding Value.
    fn node(&mut self) -> Result<(), CompareError> {
        self.ledger.checkpoint(self.control)?;
        self.ledger.visits(1)?;
        self.ledger.matching(1)?;
        Ok(())
    }
    /// Charge both complete string extents before exact unnormalized comparison.
    fn string(&mut self, expected: &str, actual: &str) -> Result<(), CompareError> {
        let extent = add(expected.len(), actual.len(), self.ledger)?;
        self.ledger.bytes(extent)?;
        if expected == actual { Ok(()) } else { Err(CompareError(ContractError::Binding)) }
    }
}

/// Replay native serialization against one borrowed stored node without copying native data.
struct CompareSerializer<'v, 's, 'c> {
    /// The already duplicate-safe bounded stored node to match completely.
    value: &'v Value,
    /// Same original invocation state, reborrowed sequentially for descendants.
    state: &'s mut ComparisonState<'c>,
}

impl<'v, 's, 'c> serde::Serializer for CompareSerializer<'v, 's, 'c> {
    type Ok = ();
    type Error = CompareError;
    type SerializeSeq = CompareSequence<'v, 's, 'c>;
    type SerializeTuple = CompareSequence<'v, 's, 'c>;
    type SerializeTupleStruct = CompareSequence<'v, 's, 'c>;
    type SerializeTupleVariant = Impossible<(), CompareError>;
    type SerializeMap = Impossible<(), CompareError>;
    type SerializeStruct = CompareStruct<'v, 's, 'c>;
    type SerializeStructVariant = Impossible<(), CompareError>;

    /// Preserve native boolean type/value if a future admitted field uses it.
    fn serialize_bool(self, value: bool) -> Result<(), CompareError> {
        self.state.node()?;
        self.state.ledger.bytes(10)?;
        if self.value.as_bool() == Some(value) {
            Ok(())
        } else {
            Err(CompareError(ContractError::Binding))
        }
    }
    /// Preserve the native signed integer representation without accepting floats or text.
    fn serialize_i64(self, value: i64) -> Result<(), CompareError> {
        self.state.node()?;
        self.state.ledger.bytes(40)?;
        if self.value.as_i64() == Some(value) {
            Ok(())
        } else {
            Err(CompareError(ContractError::Binding))
        }
    }
    /// Preserve every native usize/unsigned summary integer, including exact all-fifteen counts.
    fn serialize_u64(self, value: u64) -> Result<(), CompareError> {
        self.state.node()?;
        self.state.ledger.bytes(40)?;
        if self.value.as_u64() == Some(value) {
            Ok(())
        } else {
            Err(CompareError(ContractError::Binding))
        }
    }
    /// Preserve exact native private UTF-8 strings with no trim/case/time/href normalization.
    fn serialize_str(self, value: &str) -> Result<(), CompareError> {
        self.state.node()?;
        let actual = self.value.as_str().ok_or(CompareError(ContractError::Binding))?;
        self.state.string(value, actual)
    }
    /// Use a stack UTF-8 buffer rather than allocate an uncharged scalar string.
    fn serialize_char(self, value: char) -> Result<(), CompareError> {
        self.serialize_str(value.encode_utf8(&mut [0; 4]))
    }
    /// Present Options serialize as the exact native inner node.
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), CompareError> {
        value.serialize(self)
    }
    /// Explicit native None/unit is null only; skipped fields remain absent through struct length.
    fn serialize_none(self) -> Result<(), CompareError> {
        self.serialize_unit()
    }
    /// Preserve explicit native null without inferring it from a missing field.
    fn serialize_unit(self) -> Result<(), CompareError> {
        self.state.node()?;
        self.state.ledger.bytes(8)?;
        if self.value.is_null() { Ok(()) } else { Err(CompareError(ContractError::Binding)) }
    }
    /// Preserve native unit-struct null representation if ever encountered.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), CompareError> {
        self.serialize_unit()
    }
    /// Native enum spelling is supplied by its actual serde rename contract.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<(), CompareError> {
        self.serialize_str(variant)
    }
    /// Preserve transparent native wrappers without inventing an extra stored shape.
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), CompareError> {
        value.serialize(self)
    }
    /// Native Vec lengths are complete and every ordered element is consumed exactly once.
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, CompareError> {
        self.state.node()?;
        let rows = self.value.as_array().ok_or(CompareError(ContractError::Binding))?;
        if len != Some(rows.len()) {
            return Err(CompareError(ContractError::Binding));
        }
        Ok(CompareSequence { rows, next: 0, state: self.state })
    }
    /// Preserve tuple order through the same complete array matcher.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, CompareError> {
        self.serialize_seq(Some(len))
    }
    /// Preserve tuple-struct order through the same complete array matcher.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, CompareError> {
        self.serialize_seq(Some(len))
    }
    /// Native derived struct field count includes actual skip rules, closing every object.
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, CompareError> {
        self.state.node()?;
        let fields = self.value.as_object().ok_or(CompareError(ContractError::Binding))?;
        if fields.len() != len {
            return Err(CompareError(ContractError::Binding));
        }
        Ok(CompareStruct { fields, consumed: 0, state: self.state })
    }
    /// Preserve the native report's human-readable serde contract.
    fn is_human_readable(&self) -> bool {
        true
    }
    /// No native report f32 field exists; an added unreviewed shape refuses completely.
    fn serialize_f32(self, _value: f32) -> Result<(), CompareError> {
        Err(CompareError(ContractError::Binding))
    }
    /// No native report f64 field exists; float summary substitutions cannot match integers.
    fn serialize_f64(self, _value: f64) -> Result<(), CompareError> {
        Err(CompareError(ContractError::Binding))
    }
    /// No byte-array field exists in the native report; no coercion is invented.
    fn serialize_bytes(self, _value: &[u8]) -> Result<(), CompareError> {
        Err(CompareError(ContractError::Binding))
    }
    /// No tagged newtype variant exists in the native report; refuse unsupported future shapes.
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<(), CompareError> {
        Err(CompareError(ContractError::Binding))
    }
    /// No tagged tuple variant exists in the native report; refuse unsupported future shapes.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, CompareError> {
        Err(CompareError(ContractError::Binding))
    }
    /// No dynamic map exists in the native report; closed derived structs remain the full shape.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, CompareError> {
        Err(CompareError(ContractError::Binding))
    }
    /// No tagged struct variant exists in the native report; refuse unsupported future shapes.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, CompareError> {
        Err(CompareError(ContractError::Binding))
    }
    /// Prevent serde's default uncharged formatting allocation for unsupported future fields.
    fn collect_str<T: ?Sized + fmt::Display>(self, _value: &T) -> Result<(), CompareError> {
        Err(CompareError(ContractError::Binding))
    }
    /// Route each native signed width to exact signed scalar equality.
    fn serialize_i8(self, value: i8) -> Result<(), CompareError> {
        self.serialize_i64(i64::from(value))
    }
    /// Route each native signed width to exact signed scalar equality.
    fn serialize_i16(self, value: i16) -> Result<(), CompareError> {
        self.serialize_i64(i64::from(value))
    }
    /// Route each native signed width to exact signed scalar equality.
    fn serialize_i32(self, value: i32) -> Result<(), CompareError> {
        self.serialize_i64(i64::from(value))
    }
    /// Route each native unsigned width to exact unsigned scalar equality.
    fn serialize_u8(self, value: u8) -> Result<(), CompareError> {
        self.serialize_u64(u64::from(value))
    }
    /// Route each native unsigned width to exact unsigned scalar equality.
    fn serialize_u16(self, value: u16) -> Result<(), CompareError> {
        self.serialize_u64(u64::from(value))
    }
    /// Route each native unsigned width to exact unsigned scalar equality.
    fn serialize_u32(self, value: u32) -> Result<(), CompareError> {
        self.serialize_u64(u64::from(value))
    }
}

/// Consume all ordered stored array elements against actual native Serialize elements.
struct CompareSequence<'v, 's, 'c> {
    /// Complete stored rows, already structurally/raw bounded.
    rows: &'v [Value],
    /// Actual ordered element consumption count, not a caller complete flag.
    next: usize,
    /// Existing original invocation ledger/control.
    state: &'s mut ComparisonState<'c>,
}
impl SerializeSeq for CompareSequence<'_, '_, '_> {
    type Ok = ();
    type Error = CompareError;
    /// Compare one exact next native element; no truncation, sorting or deduplication occurs.
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), CompareError> {
        self.state.ledger.visits(1)?;
        self.state.ledger.matching(1)?;
        let row = self.rows.get(self.next).ok_or(CompareError(ContractError::Binding))?;
        value.serialize(CompareSerializer { value: row, state: self.state })?;
        self.next = add(self.next, 1, self.state.ledger)?;
        Ok(())
    }
    /// A native sequence cannot leave any hidden trailing stored row.
    fn end(self) -> Result<(), CompareError> {
        if self.next == self.rows.len() {
            Ok(())
        } else {
            Err(CompareError(ContractError::Binding))
        }
    }
}
impl SerializeTuple for CompareSequence<'_, '_, '_> {
    type Ok = ();
    type Error = CompareError;
    /// Preserve exact tuple element order through complete sequence matching.
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), CompareError> {
        SerializeSeq::serialize_element(self, value)
    }
    /// Require complete tuple consumption with no missing or extra rows.
    fn end(self) -> Result<(), CompareError> {
        SerializeSeq::end(self)
    }
}
impl SerializeTupleStruct for CompareSequence<'_, '_, '_> {
    type Ok = ();
    type Error = CompareError;
    /// Preserve exact tuple-struct element order through complete sequence matching.
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), CompareError> {
        SerializeSeq::serialize_element(self, value)
    }
    /// Require complete tuple-struct consumption with no missing or extra rows.
    fn end(self) -> Result<(), CompareError> {
        SerializeSeq::end(self)
    }
}

/// Match all native emitted struct keys and values against a closed stored object.
struct CompareStruct<'v, 's, 'c> {
    /// Complete duplicate-safe stored fields; JSON object ordering is presentation.
    fields: &'v Map<String, Value>,
    /// Native emitted key count, compared to exact actual stored key count.
    consumed: usize,
    /// Existing original invocation ledger/control.
    state: &'s mut ComparisonState<'c>,
}
impl SerializeStruct for CompareStruct<'_, '_, '_> {
    type Ok = ();
    type Error = CompareError;
    /// Precharge every complete candidate key extent before lookup, then match its full value.
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), CompareError> {
        self.state.ledger.visits(self.fields.len())?;
        let mut extent = mul(key.len(), self.fields.len(), self.state.ledger)?;
        for actual in self.fields.keys() {
            extent = add(extent, actual.len(), self.state.ledger)?;
        }
        self.state.ledger.bytes(extent)?;
        self.state.ledger.matching(self.fields.len())?;
        let stored = self.fields.get(key).ok_or(CompareError(ContractError::Binding))?;
        value.serialize(CompareSerializer { value: stored, state: self.state })?;
        self.consumed = add(self.consumed, 1, self.state.ledger)?;
        Ok(())
    }
    /// Equality requires every emitted native key and no stored unknown/replacement key.
    fn end(self) -> Result<(), CompareError> {
        if self.consumed == self.fields.len() {
            Ok(())
        } else {
            Err(CompareError(ContractError::Binding))
        }
    }
}

#[cfg(test)]
/// Prospective genuine producer/data controls; no capture or acceptance evidence is issued.
mod tests {
    use super::*;
    use crate::framework::analysis::legacy_borrowed::{FrameworkBytes, ImpactInputs};
    use crate::framework::model::{ImpactFilters, ReasonCode};
    use crate::workspace::preparation::{
        Interruption, NoopControl, ProgressUpdate, Stage, WorkError, WorkResult,
        test_support::Recorder,
    };
    use serde_json::json;

    /// Redistributable fixed synthetic framework identity, never a real project owner assertion.
    const ROOT_UUID: &str = "11111111-1111-4111-8111-111111111111";
    /// Complete declared fixture originals; native facts always come from maintained producers.
    struct Fixture {
        /// Real temporary directory for ordinary file-producer parity only.
        directory: tempfile::TempDir,
        /// Actual duplicate-safe native Impact manifest input.
        manifest: Vec<u8>,
        /// Complete old native Catalog bytes.
        old: Vec<u8>,
        /// Complete new native Catalog bytes.
        new: Vec<u8>,
        /// Complete explicit old Profile companion, if configured.
        old_companion: Option<Vec<u8>>,
        /// Complete explicit new Profile companion, if configured.
        new_companion: Option<Vec<u8>>,
        /// Complete declared native successor input, if configured.
        successor: Option<Vec<u8>>,
        /// Optional genuine complete prior report from maintained file analysis.
        prior: Option<Vec<u8>>,
        /// Optional actual private assertions bound to that prior original.
        dispositions: Option<Vec<u8>>,
    }
    /// Render genuine test declarations through the maintained JSON dependency.
    fn bytes(value: &Value) -> Vec<u8> {
        serde_json::to_vec_pretty(value).unwrap()
    }
    /// Declare an OSCAL Catalog with real statement contents and exact private version spelling.
    fn catalog(version: &str, controls: &[(&str, &str)]) -> Vec<u8> {
        bytes(
            &json!({"catalog":{"uuid":ROOT_UUID,"metadata":{"title":"Synthetic framework","last-modified":"2026-08-25T12:00:00Z","version":version,"oscal-version":crate::oscal::OSCAL_VERSION},"groups":[{"id":"group-1","title":"Synthetic group","controls":controls.iter().map(|(id,prose)| json!({"id":id,"title":format!("Control {id}"),"parts":[{"id":format!("{id}_smt"),"name":"statement","prose":prose}]})).collect::<Vec<_>>()}]}}),
        )
    }
    /// Bind each exact actual native artifact digest and private tuple without normalizing it.
    fn resource(path: &str, raw: &[u8], version: &str) -> Value {
        json!({"type":"catalog","artifact":path,"expected_sha256":crate::hashing::sha256_hex(raw),"root_uuid":ROOT_UUID,"document_version":version,"oscal_version":crate::oscal::OSCAL_VERSION})
    }
    /// Create complete genuine changed, unchanged or metadata-only native fixture inputs.
    fn fixture(mode: &str) -> Fixture {
        let directory = tempfile::tempdir().unwrap();
        let old = catalog("1.0.0 private baseline", &[("same", "Same"), ("changed", "Old")]);
        let new = match mode {
            "unchanged" => old.clone(),
            "metadata" => {
                catalog("2.0.0 private successor", &[("same", "Same"), ("changed", "Old")])
            }
            _ => catalog(
                "2.0.0 private successor",
                &[("same", "Same"), ("changed", "New"), ("added", "Added")],
            ),
        };
        let new_version =
            if mode == "unchanged" { "1.0.0 private baseline" } else { "2.0.0 private successor" };
        let manifest = bytes(
            &json!({"schema_version":"forge.framework-impact/1","old":resource("old.json",&old,"1.0.0 private baseline"),"new":resource("new.json",&new,new_version),"mapping_collections":[]}),
        );
        Fixture {
            directory,
            manifest,
            old,
            new,
            old_companion: None,
            new_companion: None,
            successor: None,
            prior: None,
            dispositions: None,
        }
    }
    impl Fixture {
        /// Install explicit ordinary producer originals only; this is not captured-reader staging.
        fn install(&self) {
            for (name, raw) in [
                ("old.json", self.old.as_slice()),
                ("new.json", self.new.as_slice()),
                ("impact.json", self.manifest.as_slice()),
            ] {
                std::fs::write(self.directory.path().join(name), raw).unwrap();
            }
            for (name, raw) in [
                ("old-resolved.json", self.old_companion.as_deref()),
                ("new-resolved.json", self.new_companion.as_deref()),
                ("successor.json", self.successor.as_deref()),
                ("prior.json", self.prior.as_deref()),
                ("dispositions.json", self.dispositions.as_deref()),
            ] {
                if let Some(raw) = raw {
                    std::fs::write(self.directory.path().join(name), raw).unwrap();
                }
            }
        }
        /// Obtain full maintained file output and coupled plain borrowed facts on the given ledger.
        fn native(
            &self,
            filters: ImpactFilters,
            ledger: &mut ContractLedger,
        ) -> (LegacyImpactFacts, Vec<u8>) {
            self.native_control(filters, ledger, &mut NoopControl)
        }
        /// Forward one actual caller control through C before D in the coupled boundary control.
        fn native_control(
            &self,
            filters: ImpactFilters,
            ledger: &mut ContractLedger,
            control: &mut dyn WorkControl,
        ) -> (LegacyImpactFacts, Vec<u8>) {
            self.install();
            let manifest = crate::framework::manifest::parse(&self.manifest).unwrap();
            let (ordinary, _) = crate::framework::analysis::analyze(
                self.directory.path(),
                &manifest,
                filters.clone(),
            )
            .unwrap();
            let facts = super::super::legacy_work::prepare_impact(
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
                    mappings: &[],
                    applicability: None,
                    successor_map: self.successor.as_deref(),
                    prior_report: self.prior.as_deref(),
                    dispositions: self.dispositions.as_deref(),
                },
                filters,
                ledger,
                control,
            )
            .unwrap();
            assert_eq!(
                serde_json::to_value(&ordinary).unwrap(),
                serde_json::to_value(&facts.report).unwrap()
            );
            assert_eq!(
                serde_json::to_value(&ordinary.filtered_out_findings).unwrap(),
                serde_json::to_value(&facts.report.filtered_out_findings).unwrap()
            );
            (facts, serde_json::to_vec_pretty(&ordinary).unwrap())
        }
        /// Add genuine current/prior-only native disposition rows through the maintained Mapping producer.
        fn with_history(mut self) -> Self {
            self.install();
            let mut policy: Value = serde_json::from_slice(&catalog(
                "private policy version",
                &[("policy-1", "Policy")],
            ))
            .unwrap();
            policy["catalog"]["uuid"] = json!("22222222-2222-4222-8222-222222222222");
            std::fs::write(self.directory.path().join("policy.json"), bytes(&policy)).unwrap();
            let mapping_manifest = bytes(
                &json!({"schema_version":"forge.mapping-manifest/1","collection":{"key":"native-history","title":"Synthetic mapping","version":"1.0.0","last_modified":"2026-08-25T12:00:00Z"},"reviewers":[{"key":"reviewer","type":"person","name":"Synthetic Reviewer"}],"provenance":{"method":"human","matching_rationale":"semantic","status":"complete","mapping_description":"Synthetic source controls only.","reviewer_keys":["reviewer"],"reviewed_at":"2026-08-25T12:00:00Z"},"mapping":{"key":"native-history","scope":"control-only","source":{"type":"catalog","artifact":"policy.json","href":"private-policy.json"},"target":{"type":"catalog","artifact":"old.json","href":"old.json"},"maps":[{"key":"native-map","relationship":"intersects-with","sources":[{"type":"control","id_ref":"policy-1"}],"targets":[{"type":"control","id_ref":"changed"}],"reviewer_key":"reviewer","reviewed_at":"2026-08-25T12:00:00Z","rationale":"Synthetic exact native relationship."}]}}),
            );
            let build = self.directory.path().join("mapping-build.json");
            std::fs::write(&build, mapping_manifest).unwrap();
            let mapping = crate::mapping::prepare(&build, None, false).unwrap();
            std::fs::write(self.directory.path().join("mapping.json"), mapping.artifact_json)
                .unwrap();
            let mut declared: Value = serde_json::from_slice(&self.manifest).unwrap();
            declared["mapping_collections"] =
                json!([{"artifact":"mapping.json","framework_role":"target"}]);
            let prior_manifest = crate::framework::manifest::parse(&bytes(&declared)).unwrap();
            let (prior, _) = crate::framework::analysis::analyze(
                self.directory.path(),
                &prior_manifest,
                ImpactFilters::default(),
            )
            .unwrap();
            let current_id = prior
                .findings
                .iter()
                .find(|row| row.reason_code == ReasonCode::ControlContentChanged)
                .unwrap()
                .finding_id
                .clone();
            let prior_only_id = prior
                .findings
                .iter()
                .find(|row| row.dependency_id.is_some())
                .unwrap()
                .finding_id
                .clone();
            self.prior = Some(serde_json::to_vec_pretty(&prior).unwrap());
            self.dispositions = Some(bytes(
                &json!({"schema_version":"forge.framework-impact-dispositions/1","prior_report_sha256":crate::hashing::sha256_hex(self.prior.as_deref().unwrap()),"dispositions":[{"finding_id":current_id,"status":"resolved","decided_by":" private reviewer ","decided_at":"2026-08-25T12:00:00Z","rationale":" private current rationale "},{"finding_id":prior_only_id,"status":"accepted-risk","decided_by":" private historical reviewer ","decided_at":"2026-08-25T12:00:00Z","rationale":" private historical rationale "}]}),
            ));
            declared["mapping_collections"] = json!([]);
            declared["prior_report"] = json!("prior.json");
            declared["disposition_file"] = json!("dispositions.json");
            self.manifest = bytes(&declared);
            // Ordinary parity remains file-oriented; the borrowed preparation gets no
            // producer-only files or unconfigured Mapping/policy originals.
            for name in ["policy.json", "mapping-build.json", "mapping.json"] {
                std::fs::remove_file(self.directory.path().join(name)).unwrap();
            }
            self
        }
    }

    /// Genuine maintained private reports accept equivalent JSON presentation and expose only the whitelist.
    #[test]
    fn full_native_serializer_shape_and_minimized_projection_preserve_all_private_data() {
        let fixture = fixture("changed");
        let mut ledger = ContractLedger::default();
        let (facts, raw) = fixture.native(ImpactFilters::default(), &mut ledger);
        let data = compare_stored(&raw, &facts, &mut ledger, &mut NoopControl).unwrap();
        assert_eq!(data.summary().rows(), facts.report.summary.rows());
        assert_eq!(data.change_count(), facts.report.changes.len());
        assert_eq!(data.finding_count(), facts.report.findings.len());
        assert_eq!(data.report().new.document_version, "2.0.0 private successor");
        assert_eq!(data.report().new.href, "new.json");
        let projection = serde_json::to_value(data.new_resource()).unwrap();
        assert_eq!(projection.as_object().unwrap().len(), 5);
        assert_eq!(projection["resolved_catalog_sha256"], Value::Null);
        assert!(projection.get("document_version").is_none());
        assert!(projection.get("href").is_none());
        let rendered = serde_json::to_string(&projection).unwrap();
        assert!(!rendered.contains("private successor"));
        assert!(!rendered.contains("new.json"));
        let mut reordered: Value = serde_json::from_slice(&raw).unwrap();
        let object = reordered.as_object_mut().unwrap();
        let old = object.remove("old").unwrap();
        object.insert("old".into(), old);
        let compact = serde_json::to_vec(&reordered).unwrap();
        compare_stored(&compact, &facts, &mut ledger, &mut NoopControl).unwrap();
    }

    /// Every one of fifteen summary cells and the complete matched denominator is protected.
    #[test]
    fn forged_all_fifteen_counts_and_matching_forged_native_counts_refuse() {
        let fixture = fixture("changed");
        let names = [
            "old_controls",
            "new_controls",
            "added",
            "removed",
            "content_changed",
            "identity_migrated",
            "unchanged",
            "findings",
            "blocking",
            "review_required",
            "informational",
            "dispositioned_resolved",
            "dispositioned_accepted_risk",
            "dispositioned_still_open",
            "undispositioned",
        ];
        for name in names {
            let mut ledger = ContractLedger::default();
            let (facts, raw) = fixture.native(ImpactFilters::default(), &mut ledger);
            let mut invalid: Value = serde_json::from_slice(&raw).unwrap();
            invalid["summary"][name] = json!(invalid["summary"][name].as_u64().unwrap() + 1);
            assert!(matches!(
                compare_stored(&bytes(&invalid), &facts, &mut ledger, &mut NoopControl),
                Err(ContractError::Binding)
            ));
            // Deliberately forged expected data is still not authorizing. Correlations
            // reject even when an attacker tries to make both sides serialize alike.
            let forged = LegacyImpactFacts {
                manifest: facts.manifest.clone(),
                report: serde_json::from_value(invalid.clone()).unwrap(),
                applicability_manifest: None,
            };
            assert!(matches!(
                compare_stored(&bytes(&invalid), &forged, &mut ledger, &mut NoopControl),
                Err(ContractError::Binding)
            ));
        }
        let mut ledger = ContractLedger::default();
        let (facts, raw) = fixture.native(ImpactFilters::default(), &mut ledger);
        let mut invalid: Value = serde_json::from_slice(&raw).unwrap();
        invalid["matched_findings"] = json!(0);
        assert!(matches!(
            compare_stored(&bytes(&invalid), &facts, &mut ledger, &mut NoopControl),
            Err(ContractError::Binding)
        ));
    }

    /// Full ordered changes/findings and unnormalized private tuple strings cannot be redacted away.
    #[test]
    fn complete_private_tuple_changes_findings_and_nested_field_shapes_refuse_forgery() {
        let fixture = fixture("changed");
        for path in [
            "/old/document_version",
            "/new/document_version",
            "/old/href",
            "/new/href",
            "/old/root_uuid",
            "/new/raw_sha256",
            "/findings/0/finding_id",
            "/changes/1/old_subjects/0/sha256",
        ] {
            let mut ledger = ContractLedger::default();
            let (facts, raw) = fixture.native(ImpactFilters::default(), &mut ledger);
            let mut invalid: Value = serde_json::from_slice(&raw).unwrap();
            *invalid.pointer_mut(path).unwrap() = json!("forged private value");
            assert!(
                matches!(
                    compare_stored(&bytes(&invalid), &facts, &mut ledger, &mut NoopControl),
                    Err(ContractError::Binding)
                ),
                "accepted {path}"
            );
        }
        for mode in [
            "drop-change",
            "reorder-change",
            "drop-finding",
            "reorder-finding",
            "extra-finding",
            "extra-nested",
            "missing-nested",
            "float-count",
            "explicit-none",
        ] {
            let mut ledger = ContractLedger::default();
            let (facts, raw) = fixture.native(ImpactFilters::default(), &mut ledger);
            let mut invalid: Value = serde_json::from_slice(&raw).unwrap();
            match mode {
                "drop-change" => {
                    invalid["changes"].as_array_mut().unwrap().pop();
                }
                "reorder-change" => invalid["changes"].as_array_mut().unwrap().reverse(),
                "drop-finding" => {
                    invalid["findings"].as_array_mut().unwrap().pop();
                }
                "reorder-finding" => invalid["findings"].as_array_mut().unwrap().reverse(),
                "extra-finding" => {
                    let row = invalid["findings"][0].clone();
                    invalid["findings"].as_array_mut().unwrap().push(row);
                }
                "extra-nested" => {
                    invalid["old"]["private_extension"] = json!(true);
                }
                "missing-nested" => {
                    invalid["findings"][0].as_object_mut().unwrap().remove("required_action");
                }
                "float-count" => invalid["summary"]["added"] = json!(0.0),
                "explicit-none" => invalid["old"]["resolved_catalog_sha256"] = Value::Null,
                _ => unreachable!(),
            }
            assert!(
                matches!(
                    compare_stored(&bytes(&invalid), &facts, &mut ledger, &mut NoopControl),
                    Err(ContractError::Binding)
                ),
                "accepted {mode}"
            );
        }
    }

    /// Metadata findings prevent the zero label, while genuine nonempty unchanged rows permit it.
    #[test]
    fn exact_observation_includes_full_metadata_findings_and_preserves_unchanged_rows() {
        for mode in ["metadata", "unchanged"] {
            let fixture = fixture(mode);
            let mut ledger = ContractLedger::default();
            let (facts, raw) = fixture.native(ImpactFilters::default(), &mut ledger);
            let data = compare_stored(&raw, &facts, &mut ledger, &mut NoopControl).unwrap();
            let summary = data.summary();
            assert_eq!(
                (
                    summary.added,
                    summary.removed,
                    summary.content_changed,
                    summary.identity_migrated
                ),
                (0, 0, 0, 0)
            );
            assert!(data.change_count() > 0);
            if mode == "metadata" {
                assert_eq!(data.finding_count(), 1);
                assert_eq!(
                    data.report().findings[0].reason_code,
                    ReasonCode::ResourceMetadataChanged
                );
                assert_eq!(data.observation().as_str(), "detected-native-change");
            } else {
                assert_eq!(data.finding_count(), 0);
                assert_eq!(data.observation().as_str(), "no-detected-native-change");
            }
        }
    }

    /// Complete genuine current/prior-only disposition content and order remains in equality.
    #[test]
    fn genuine_prior_only_history_and_current_private_disposition_metadata_are_complete() {
        let fixture = fixture("changed").with_history();
        let mut ledger = ContractLedger::default();
        let (facts, raw) = fixture.native(ImpactFilters::default(), &mut ledger);
        let data = compare_stored(&raw, &facts, &mut ledger, &mut NoopControl).unwrap();
        assert_eq!(data.prior_only_disposition_count(), 1);
        assert_eq!(data.summary().dispositioned_resolved, 1);
        for path in [
            "/prior_only_dispositions/0/decided_by",
            "/prior_only_dispositions/0/rationale",
            "/prior_only_dispositions/0/decided_at",
        ] {
            let mut invalid: Value = serde_json::from_slice(&raw).unwrap();
            *invalid.pointer_mut(path).unwrap() = json!("forged historical assertion");
            assert!(matches!(
                compare_stored(&bytes(&invalid), &facts, &mut ledger, &mut NoopControl),
                Err(ContractError::Binding)
            ));
        }
        let mut invalid: Value = serde_json::from_slice(&raw).unwrap();
        invalid.as_object_mut().unwrap().remove("prior_only_dispositions");
        assert!(matches!(
            compare_stored(&bytes(&invalid), &facts, &mut ledger, &mut NoopControl),
            Err(ContractError::Binding)
        ));
        let mut invalid: Value = serde_json::from_slice(&raw).unwrap();
        let row = invalid["findings"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|row| row.get("disposition").is_some())
            .unwrap();
        row["disposition"]["rationale"] = json!("forged current assertion");
        assert!(matches!(
            compare_stored(&bytes(&invalid), &facts, &mut ledger, &mut NoopControl),
            Err(ContractError::Binding)
        ));
    }

    /// All five real native filter paths and complete hidden rows must refuse this unfiltered oracle.
    #[test]
    fn all_five_native_filters_and_hidden_rows_refuse_even_when_serialized_output_matches() {
        let fixture = fixture("changed");
        for filters in [
            ImpactFilters { group: Some("group-1".into()), ..ImpactFilters::default() },
            ImpactFilters {
                decision_state: Some(crate::applicability::manifest::DecisionState::Applicable),
                ..ImpactFilters::default()
            },
            ImpactFilters {
                policy_source: Some("private-policy.json".into()),
                ..ImpactFilters::default()
            },
            ImpactFilters {
                priority: Some(FindingPriority::Informational),
                ..ImpactFilters::default()
            },
            ImpactFilters { owner: Some("private-owner".into()), ..ImpactFilters::default() },
        ] {
            let mut ledger = ContractLedger::default();
            let (facts, raw) = fixture.native(filters, &mut ledger);
            assert!(!facts.report.filters.is_empty());
            assert!(matches!(
                compare_stored(&raw, &facts, &mut ledger, &mut NoopControl),
                Err(ContractError::Binding)
            ));
        }
        let mut ledger = ContractLedger::default();
        let (mut facts, _) = fixture.native(
            ImpactFilters { owner: Some("absent-owner".into()), ..ImpactFilters::default() },
            &mut ledger,
        );
        assert!(!facts.report.filtered_out_findings.is_empty());
        facts.report.filters = ImpactFilters::default();
        let raw = serde_json::to_vec(&facts.report).unwrap();
        assert!(matches!(
            compare_stored(&raw, &facts, &mut ledger, &mut NoopControl),
            Err(ContractError::Binding)
        ));
        let mut ledger = ContractLedger::default();
        let (facts, raw) = fixture.native(ImpactFilters::default(), &mut ledger);
        for name in ["group", "decision_state", "policy_source", "priority", "owner"] {
            let mut invalid: Value = serde_json::from_slice(&raw).unwrap();
            invalid["filters"][name] = Value::Null;
            assert!(matches!(
                compare_stored(&bytes(&invalid), &facts, &mut ledger, &mut NoopControl),
                Err(ContractError::Binding)
            ));
        }
    }

    /// Root and nested duplicates, BOM, trailing data and closed optional-field substitutions refuse.
    #[test]
    fn duplicate_safe_complete_raw_admission_and_closed_native_filter_shape_are_strict() {
        let fixture = fixture("changed");
        let mut ledger = ContractLedger::default();
        let (facts, raw) = fixture.native(ImpactFilters::default(), &mut ledger);
        let compact =
            serde_json::to_string(&serde_json::from_slice::<Value>(&raw).unwrap()).unwrap();
        let duplicate_root = compact.replacen(
            "\"status\":\"complete\"",
            "\"status\":\"complete\",\"status\":\"complete\"",
            1,
        );
        let duplicate_nested =
            compact.replacen("\"filters\":{}", "\"filters\":{\"owner\":null,\"owner\":null}", 1);
        assert_ne!(duplicate_root, compact);
        assert_ne!(duplicate_nested, compact);
        let mut bom = vec![0xef, 0xbb, 0xbf];
        bom.extend_from_slice(&raw);
        let mut trailing = raw.clone();
        trailing.extend_from_slice(b" {}");
        for invalid in [
            duplicate_root.into_bytes(),
            duplicate_nested.into_bytes(),
            bom,
            trailing,
            b"{".to_vec(),
            Vec::new(),
        ] {
            assert!(matches!(
                compare_stored(&invalid, &facts, &mut ledger, &mut NoopControl),
                Err(ContractError::Invalid)
            ));
        }
        let mut invalid: Value = serde_json::from_slice(&raw).unwrap();
        invalid["filtered_out_findings"] = json!([]);
        assert!(matches!(
            compare_stored(&bytes(&invalid), &facts, &mut ledger, &mut NoopControl),
            Err(ContractError::Binding)
        ));
    }

    /// Raw equality at the existing cap and plus-one/decoded-string/depth refusal use one ledger.
    #[test]
    fn raw_and_decoded_bounds_preserve_complete_refusal_and_sticky_shared_capacity() {
        let fixture = fixture("unchanged");
        let mut ledger = ContractLedger::default();
        let (facts, mut raw) = fixture.native(ImpactFilters::default(), &mut ledger);
        raw.resize(MAX_STORED_BYTES, b' ');
        compare_stored(&raw, &facts, &mut ledger, &mut NoopControl).unwrap();
        raw.push(b' ');
        assert!(matches!(
            compare_stored(&raw, &facts, &mut ledger, &mut NoopControl),
            Err(ContractError::Capacity)
        ));
        assert!(matches!(
            compare_stored(b"{}", &facts, &mut ledger, &mut NoopControl),
            Err(ContractError::Capacity)
        ));
        for invalid in [
            format!("{{\"unknown\":\"{}\"}}", "x".repeat(MAX_STRING_BYTES + 1)).into_bytes(),
            format!("{}0{}", "[".repeat(65), "]".repeat(65)).into_bytes(),
        ] {
            let mut ledger = ContractLedger::default();
            let (facts, _) = fixture.native(ImpactFilters::default(), &mut ledger);
            assert!(matches!(
                compare_stored(&invalid, &facts, &mut ledger, &mut NoopControl),
                Err(ContractError::Invalid)
            ));
        }
        let mut ledger = ContractLedger::default();
        let (facts, raw) = fixture.native(ImpactFilters::default(), &mut ledger);
        ledger.derived(33_554_432).unwrap_err();
        assert!(matches!(
            compare_stored(&raw, &facts, &mut ledger, &mut NoopControl),
            Err(ContractError::Capacity)
        ));
    }

    /// Actual control failures latch in the existing ledger; no fake error is synthesized.
    struct FailControl {
        /// Actual checkpoint call count for detecting forbidden replacement probes.
        calls: usize,
    }
    impl WorkControl for FailControl {
        /// Fail at the real first requested boundary with the actual ordinary `WorkError`.
        fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
            self.calls += 1;
            Err(WorkError::Failed(crate::workspace::contract::Error::invalid()))
        }
        /// Ordinary control failure is distinct from an interruption.
        fn interruption(&self) -> Option<Interruption> {
            None
        }
    }
    /// A malformed parse's post-phase stop and ordinary control failure keep exact first-stop priority.
    #[test]
    fn actual_post_parse_interruption_and_control_failure_remain_sticky_across_calls() {
        let fixture = fixture("unchanged");
        let mut ledger = ContractLedger::default();
        let (facts, raw) = fixture.native(ImpactFilters::default(), &mut ledger);
        let mut recorder = Recorder::at(Stage::PrepareDomain, 4);
        assert!(matches!(
            compare_stored(b"{", &facts, &mut ledger, &mut recorder),
            Err(ContractError::Interrupted(Interruption::CancelRequested))
        ));
        assert_eq!(recorder.events.len(), 4);
        assert!(matches!(
            compare_stored(&raw, &facts, &mut ledger, &mut NoopControl),
            Err(ContractError::Interrupted(Interruption::CancelRequested))
        ));
        let mut ledger = ContractLedger::default();
        let (facts, raw) = fixture.native(ImpactFilters::default(), &mut ledger);
        let mut failure = FailControl { calls: 0 };
        assert!(matches!(
            compare_stored(&raw, &facts, &mut ledger, &mut failure),
            Err(ContractError::ControlFailed)
        ));
        assert_eq!(failure.calls, 1);
        assert!(matches!(
            compare_stored(&raw, &facts, &mut ledger, &mut failure),
            Err(ContractError::ControlFailed)
        ));
        assert_eq!(failure.calls, 1);
    }

    /// One actual original caller control/ledger continues through C into D and its final success fence.
    #[test]
    fn actual_shared_caller_continues_native_preparation_to_final_oracle_fence() {
        let fixture = fixture("unchanged");
        let mut ledger = ContractLedger::default();
        let mut baseline = Recorder::default();
        let (facts, raw) =
            fixture.native_control(ImpactFilters::default(), &mut ledger, &mut baseline);
        let before =
            baseline.events.iter().filter(|(stage, _)| *stage == Stage::PrepareDomain).count();
        compare_stored(&raw, &facts, &mut ledger, &mut baseline).unwrap();
        let final_visit =
            baseline.events.iter().filter(|(stage, _)| *stage == Stage::PrepareDomain).count();
        assert!(before > 0 && final_visit > before);
        let mut ledger = ContractLedger::default();
        let mut stopped = Recorder::at(Stage::PrepareDomain, final_visit);
        let (facts, raw) =
            fixture.native_control(ImpactFilters::default(), &mut ledger, &mut stopped);
        assert!(matches!(
            compare_stored(&raw, &facts, &mut ledger, &mut stopped),
            Err(ContractError::Interrupted(Interruption::CancelRequested))
        ));
        assert_eq!(
            stopped.events.iter().filter(|(stage, _)| *stage == Stage::PrepareDomain).count(),
            final_visit
        );
        assert!(matches!(
            compare_stored(b"{}", &facts, &mut ledger, &mut NoopControl),
            Err(ContractError::Interrupted(Interruption::CancelRequested))
        ));
    }

    /// Genuine Profile companion tuples are complete; private imports remain unopened labels.
    #[test]
    fn genuine_profile_companions_are_equal_privately_and_minimized_without_import_routes() {
        let mut fixture = fixture("changed");
        fixture.old_companion = Some(fixture.old.clone());
        fixture.new_companion = Some(fixture.new.clone());
        let profile = |version: &str| {
            bytes(
                &json!({"profile":{"uuid":ROOT_UUID,"metadata":{"title":"Synthetic Profile","last-modified":"2026-08-25T12:00:00Z","version":version,"oscal-version":crate::oscal::OSCAL_VERSION},"imports":[{"href":"unopened-import.json","include-all":{}}]}}),
            )
        };
        fixture.old = profile("1.0.0 private baseline");
        fixture.new = profile("2.0.0 private successor");
        let pair = |path: &str,
                    raw: &[u8],
                    companion: &str,
                    companion_raw: &[u8],
                    version: &str| json!({"type":"profile","artifact":path,"resolved_catalog":companion,"resolved_catalog_attestation":true,"expected_sha256":crate::hashing::sha256_hex(raw),"expected_resolved_catalog_sha256":crate::hashing::sha256_hex(companion_raw),"root_uuid":ROOT_UUID,"document_version":version,"oscal_version":crate::oscal::OSCAL_VERSION});
        fixture.manifest = bytes(
            &json!({"schema_version":"forge.framework-impact/1","old":pair("old.json",&fixture.old,"old-resolved.json",fixture.old_companion.as_deref().unwrap(),"1.0.0 private baseline"),"new":pair("new.json",&fixture.new,"new-resolved.json",fixture.new_companion.as_deref().unwrap(),"2.0.0 private successor"),"mapping_collections":[]}),
        );
        let mut ledger = ContractLedger::default();
        let (facts, raw) = fixture.native(ImpactFilters::default(), &mut ledger);
        let data = compare_stored(&raw, &facts, &mut ledger, &mut NoopControl).unwrap();
        assert_eq!(
            data.old().resolved_catalog_sha256,
            facts.report.old.resolved_catalog_sha256.as_deref()
        );
        assert_eq!(
            data.new_resource().resolved_catalog_sha256,
            facts.report.new.resolved_catalog_sha256.as_deref()
        );
        let mut invalid: Value = serde_json::from_slice(&raw).unwrap();
        invalid["new"]["resolved_catalog_sha256"] = json!("0".repeat(64));
        assert!(matches!(
            compare_stored(&bytes(&invalid), &facts, &mut ledger, &mut NoopControl),
            Err(ContractError::Binding)
        ));
        assert!(!fixture.directory.path().join("unopened-import.json").exists());
    }

    /// Genuine merge rows conserve old/new Control counts and keep complete private migration fields.
    #[test]
    fn genuine_merge_full_subject_arrays_and_private_migration_metadata_are_complete() {
        let mut fixture = fixture("changed");
        fixture.old = catalog("1.0.0 private baseline", &[("old-a", "A"), ("old-b", "B")]);
        fixture.new = catalog("2.0.0 private successor", &[("new-x", "AB")]);
        fixture.successor = Some(bytes(
            &json!({"schema_version":"forge.successor-map/1","relationships":[{"relationship":"merge","old_ids":["old-b","old-a"],"new_ids":["new-x"],"approved_by":"Synthetic private reviewer","approved_at":"2026-08-25T12:00:00Z","rationale":"Synthetic private complete merge declaration."}]}),
        ));
        fixture.manifest = bytes(
            &json!({"schema_version":"forge.framework-impact/1","old":resource("old.json",&fixture.old,"1.0.0 private baseline"),"new":resource("new.json",&fixture.new,"2.0.0 private successor"),"mapping_collections":[],"successor_map":"successor.json"}),
        );
        let mut ledger = ContractLedger::default();
        let (facts, raw) = fixture.native(ImpactFilters::default(), &mut ledger);
        let data = compare_stored(&raw, &facts, &mut ledger, &mut NoopControl).unwrap();
        assert_eq!(
            (
                data.summary().old_controls,
                data.summary().new_controls,
                data.summary().identity_migrated
            ),
            (2, 1, 1)
        );
        assert_eq!(data.report().changes[0].old_subjects.len(), 2);
        for path in [
            "/changes/0/migration/approved_by",
            "/changes/0/migration/approved_at",
            "/changes/0/migration/rationale",
        ] {
            let mut invalid: Value = serde_json::from_slice(&raw).unwrap();
            *invalid.pointer_mut(path).unwrap() = json!("forged migration metadata");
            assert!(matches!(
                compare_stored(&bytes(&invalid), &facts, &mut ledger, &mut NoopControl),
                Err(ContractError::Binding)
            ));
        }
        let mut invalid: Value = serde_json::from_slice(&raw).unwrap();
        invalid["changes"][0]["old_subjects"].as_array_mut().unwrap().reverse();
        assert!(matches!(
            compare_stored(&bytes(&invalid), &facts, &mut ledger, &mut NoopControl),
            Err(ContractError::Binding)
        ));
    }
}
