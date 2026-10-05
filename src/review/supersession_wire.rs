//! Inert borrowed fragments and closed recorded companion data, with no proof constructor.
//! Complete production, native finding association and finite final encoding remain
//! the genuine command's work. The private seven-field Impact locator is not duplicated.
//! Separate fixed public Impact source vocabulary never creates capture/currentness.

use super::decode::{ContractError, ContractLedger, Decoded};
use super::wire::{Domain, SourcePin};
use crate::workspace::preparation::WorkControl;
use serde::{Deserialize, Serialize};

/// Exact endpoint snapshot in the contract's fixed field order; all strings stay borrowed.
#[derive(Serialize)]
pub(crate) struct Endpoint<'a> {
    /// Exact queue item key.
    pub(crate) key: &'a str,
    /// Actual queue item UUID.
    pub(crate) item_id: &'a str,
    /// Actual declared native review family.
    pub(crate) domain: Domain,
    /// Exact maintained adapter marker.
    pub(crate) adapter_version: &'a str,
    /// Exact unnormalized native subject identifier.
    pub(crate) subject_id: &'a str,
    /// Recorded queue subject fingerprint, without historical approval credit.
    pub(crate) subject_sha256: &'a str,
    /// Exact complete typed context pin.
    pub(crate) context_sha256: &'a str,
    /// Exact selected declared policy key.
    pub(crate) policy_key: &'a str,
    /// Exact complete asserted policy/assignment/deadline pin.
    pub(crate) policy_sha256: &'a str,
    /// Fixed re-review intent, without promotion.
    pub(crate) requested_action: super::wire::RequestedAction,
    /// Complete exact ordered item source keys.
    pub(crate) source_keys: &'a [String],
}

/// Complete typed differences; no boolean is a native/currentness proof.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "The closed versioned companion requires exactly four independent difference booleans."
)]
pub(crate) struct Differences {
    /// Domain/adapter/subject identity/fingerprint differ.
    pub(crate) subject: bool,
    /// Complete existing `ContextEncoding` input fields differ.
    pub(crate) context: bool,
    /// Complete existing `PolicyEncoding` input fields differ.
    pub(crate) policy: bool,
    /// Complete corresponding item `SourcePin` values differ, not just their keys.
    pub(crate) source_pins: bool,
}

/// Complete graph counters excluding the still-unbound native finding denominator.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct GraphCounts {
    /// Entire old queue item count.
    pub(crate) old_items: usize,
    /// Entire new queue item count.
    pub(crate) new_items: usize,
    /// Unique explicit pair count, distinct from either endpoint membership count.
    pub(crate) link_edges: usize,
    /// Distinct old endpoint membership.
    pub(crate) linked_old_items: usize,
    /// Distinct new endpoint membership.
    pub(crate) linked_new_items: usize,
    /// Complete old unmatched count.
    pub(crate) unmatched_old_items: usize,
    /// Complete new unmatched count.
    pub(crate) unmatched_new_items: usize,
    /// All per-link finding occurrences, including repeats across links.
    pub(crate) link_finding_occurrences: usize,
    /// Unique requested IDs only; not a native report finding count.
    pub(crate) distinct_requested_findings: usize,
}

/// Complete raw pin fragment extracted from actual held/decoded originals by the caller.
#[derive(Serialize)]
pub(crate) struct OriginalPin<'a> {
    /// Exact raw hash, never a semantic reserialization hash.
    pub(crate) raw_sha256: &'a str,
    /// Entire actual original extent.
    pub(crate) byte_length: u64,
}

/// Complete inert queue reference in the contract's fixed field order.
#[derive(Serialize)]
pub(crate) struct QueueReference<'a> {
    /// Exact unchanged queue schema marker.
    pub(crate) schema_version: &'a str,
    /// Actual decoded queue UUID.
    pub(crate) queue_id: &'a str,
    /// Whole original digest, including whitespace and LF.
    pub(crate) raw_sha256: &'a str,
    /// Whole original byte count.
    pub(crate) byte_length: u64,
    /// Entire unchanged `SourcePin` roster, never an item-only subset.
    pub(crate) source_pins: &'a [SourcePin],
    /// Entire actual item count.
    pub(crate) item_count: usize,
}

/// Extract an inert complete reference from an actual Decoded queue, without owner proof.
pub(crate) fn queue_reference<'a>(
    queue: &'a Decoded<'_, super::wire::QueueDocument>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<QueueReference<'a>, ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        ledger.visits(1)?;
        ledger.bytes(std::mem::size_of::<QueueReference<'a>>())?;
        let byte_length = u64::try_from(queue.raw().len()).map_err(|_| ledger.capacity())?;
        let document = queue.document();
        let reference = QueueReference {
            schema_version: &document.schema_version,
            queue_id: &document.queue_id,
            raw_sha256: queue.raw_sha256(),
            byte_length,
            source_pins: &document.source_pins,
            item_count: document.items.len(),
        };
        ledger.checkpoint(control)?;
        Ok(reference)
    })
}

// All following document types are inert recorded assertions, including their
// fixed recorded-current/complete labels. They create no captured owner, native
// proof, factory or publication capability. Root owns actual complete production.

/// Require explicit nullable fields in derived serde input; schema also requires every key.
fn required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// Require every explicit null filter field rather than treating omission as a default.
fn required_null<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: serde::Deserializer<'de>,
{
    <()>::deserialize(deserializer)
}

/// Fixed separate companion marker; a schema label grants no authority.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum SupersessionSchema {
    /// Exact companion revision.
    #[serde(rename = "forge.review-queue-supersession/1")]
    V1,
}

/// Fixed existing queue reference marker.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum QueueSchema {
    /// Exact unchanged queue revision.
    #[serde(rename = "forge.review-queue/1")]
    V1,
}

/// Mandatory asserted-only identity limitation.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum AssertedIdentity {
    /// Exact existing disclaimer, not authentication.
    #[serde(
        rename = "keys-roles-authors-and-times-are-asserted-not-authenticated-signed-or-non-repudiable"
    )]
    Asserted,
}

/// Fixed declared direct relation without predecessor response transfer.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum LineageSemantics {
    /// Loaded relation remains an inert assertion.
    #[serde(rename = "declared-lineage-no-response-transfer")]
    Declared,
}

/// Explicit historical declaration label.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum OldQueueCurrentness {
    /// Never a re-established old native approval.
    #[serde(rename = "historical-unverified")]
    Historical,
}

/// Inert recorded new-queue label; genuine current owner remains separate.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum NewQueueCurrentness {
    /// Loaded bytes never restore currentness.
    #[serde(rename = "recorded-current-at-creation-fence")]
    Recorded,
}

/// Inert recorded comparison label; full native owner remains separate.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ImpactCurrentness {
    /// Loaded bytes never restore capture or recomputation.
    #[serde(rename = "captured-recomputed-at-creation-fence")]
    Recorded,
}

/// Recorded full-output claim; struct construction is not completion proof.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum CompleteStatus {
    /// Actual Root complete native gates must precede production.
    #[serde(rename = "complete")]
    Complete,
}

/// Fixed native report revision from the maintained model.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ImpactReportSchema {
    /// Exact maintained `REPORT_SCHEMA_VERSION`.
    #[serde(rename = "forge.framework-impact-report/1")]
    V1,
}

/// Each explicit link has this fixed declared relation.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum LineageRelation {
    /// No inferred identity successor or vote inheritance.
    #[serde(rename = "explicit-declared-lineage")]
    Declared,
}

/// Finding to new review item remains explicit lineage.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum NewFindingRelation {
    /// No semantic equivalence or transferable approval.
    #[serde(rename = "explicit-declared-review-lineage")]
    Declared,
}

/// Inert recorded association family; actual native correlation is separate.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum OldDependencyBinding {
    /// Genuine captured Mapping dependency and historical old pin must match.
    #[serde(rename = "captured-native-map-id-and-declared-old-pin")]
    Mapping,
    /// Genuine captured applicability control and historical old pin must match.
    #[serde(rename = "captured-native-control-id-and-declared-old-pin")]
    Applicability,
}

/// Recorded closed observation, not native/currentness/lineage authority.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum NativeChangeObservation {
    /// Only four full counters zero AND complete findings empty.
    #[serde(rename = "no-detected-native-change")]
    NoneDetected,
    /// Exact complement after actual complete native proof.
    #[serde(rename = "detected-native-change")]
    Detected,
}

/// Separate nine-kind Impact vocabulary; existing review `SourceModel` is unchanged.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ImpactSourceKind {
    /// Exact catalog capture-family label.
    #[serde(rename = "catalog")]
    Catalog,
    /// Exact profile capture-family label.
    #[serde(rename = "profile")]
    Profile,
    /// Exact resolved-catalog capture-family label.
    #[serde(rename = "resolved-catalog")]
    ResolvedCatalog,
    /// Exact mapping capture-family label.
    #[serde(rename = "mapping")]
    Mapping,
    /// Exact applicability-manifest capture-family label.
    #[serde(rename = "applicability-manifest")]
    ApplicabilityManifest,
    /// Exact framework-impact-manifest capture-family label.
    #[serde(rename = "framework-impact-manifest")]
    ImpactManifest,
    /// Exact framework-impact-report capture-family label.
    #[serde(rename = "framework-impact-report")]
    ImpactReport,
    /// Exact successor-map capture-family label.
    #[serde(rename = "successor-map")]
    SuccessorMap,
    /// Exact framework-impact-dispositions capture-family label.
    #[serde(rename = "framework-impact-dispositions")]
    Dispositions,
}

/// Only fixed native family/revision identities; no document version or href.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ImpactSchemaIdentity {
    /// Exact fixed oscal:1.2.3:catalog schema identity.
    #[serde(rename = "oscal:1.2.3:catalog")]
    Catalog,
    /// Exact fixed oscal:1.2.3:profile schema identity.
    #[serde(rename = "oscal:1.2.3:profile")]
    Profile,
    /// Exact fixed oscal:1.2.3:mapping-collection schema identity.
    #[serde(rename = "oscal:1.2.3:mapping-collection")]
    Mapping,
    /// Exact fixed forge.applicability/1 schema identity.
    #[serde(rename = "forge.applicability/1")]
    Applicability,
    /// Exact fixed forge.framework-impact/1 schema identity.
    #[serde(rename = "forge.framework-impact/1")]
    Manifest,
    /// Exact fixed forge.framework-impact-report/1 schema identity.
    #[serde(rename = "forge.framework-impact-report/1")]
    Report,
    /// Exact fixed forge.successor-map/1 schema identity.
    #[serde(rename = "forge.successor-map/1")]
    Successor,
    /// Exact fixed forge.framework-impact-dispositions/1 schema identity.
    #[serde(rename = "forge.framework-impact-dispositions/1")]
    Dispositions,
}

/// Exact admitted native framework families.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ImpactResourceType {
    /// Native Catalog, with explicit null resolved companion.
    #[serde(rename = "catalog")]
    Catalog,
    /// Native Profile, with its exact declared resolved Catalog companion.
    #[serde(rename = "profile")]
    Profile,
}

/// Fixed actual maintained OSCAL revision, never arbitrary version text.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum SupportedOscalVersion {
    /// Exact `crate::oscal::OSCAL_VERSION`.
    #[serde(rename = "1.2.3")]
    V123,
}

/// Owned inert original pin, not a held raw original.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecordedOriginalPin {
    /// Exact asserted complete raw digest.
    pub(crate) raw_sha256: String,
    /// Asserted entire raw extent.
    pub(crate) byte_length: u64,
}

/// Exact existing `SourcePin` fields with explicit required nullable native identity.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecordedQueueSourcePin {
    /// Exact queue-declared source key.
    pub(crate) artifact_key: String,
    /// Existing unchanged queue source family.
    pub(crate) model: super::wire::SourceModel,
    /// Explicit null for companion or actual declared canonical native UUID.
    #[serde(deserialize_with = "required_nullable")]
    pub(crate) native_root_uuid: Option<String>,
    /// Exact declared original digest.
    pub(crate) raw_sha256: String,
    /// Exact declared original byte count.
    pub(crate) byte_length: u64,
    /// Existing queue schema identity; genuine native binding remains separate.
    pub(crate) schema_identity: String,
}

/// Owned inert complete queue reference; bytes remain a separate original owner.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecordedQueueReference {
    /// Fixed unchanged queue marker.
    pub(crate) schema_version: QueueSchema,
    /// Actual asserted canonical queue UUID.
    pub(crate) queue_id: String,
    /// Whole original queue digest.
    pub(crate) raw_sha256: String,
    /// Whole original queue extent.
    pub(crate) byte_length: u64,
    /// Entire exact ordered queue pin roster.
    pub(crate) source_pins: Vec<RecordedQueueSourcePin>,
    /// Entire item denominator.
    pub(crate) item_count: u64,
}

/// Owned inert endpoint with the exact same eleven-field minimized whitelist.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecordedEndpoint {
    /// Exact bounded queue item key.
    pub(crate) key: String,
    /// Exact canonical item UUID.
    pub(crate) item_id: String,
    /// Actual asserted native review family.
    pub(crate) domain: Domain,
    /// Exact maintained family/version marker.
    pub(crate) adapter_version: String,
    /// Exact unnormalized queue subject ID.
    pub(crate) subject_id: String,
    /// Historical or current recorded queue subject digest, not native proof.
    pub(crate) subject_sha256: String,
    /// Complete existing context binding digest.
    pub(crate) context_sha256: String,
    /// Actual selected policy key.
    pub(crate) policy_key: String,
    /// Complete existing asserted policy binding digest.
    pub(crate) policy_sha256: String,
    /// Exact re-review intent.
    pub(crate) requested_action: super::wire::RequestedAction,
    /// Complete ordered exact item source-key roster.
    pub(crate) source_keys: Vec<String>,
}

/// Inert minimized finding row; actual old dependency association is Root-owned.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FindingReference {
    /// Exact current finding ID scoped to the comparison.
    pub(crate) finding_id: String,
    /// Actual fixed native finding priority.
    pub(crate) priority: crate::framework::model::FindingPriority,
    /// Actual fixed native reason code.
    pub(crate) reason_code: crate::framework::model::ReasonCode,
    /// Actual fixed native required action.
    pub(crate) required_action: crate::framework::model::RequiredAction,
    /// Actual exact old native control ID; never a review fingerprint substitute.
    pub(crate) native_subject_id: String,
    /// Actual fixed native change class.
    pub(crate) change_class: crate::framework::model::ChangeClass,
    /// Recorded genuine-association family, not self-authorizing evidence.
    pub(crate) old_dependency_binding: OldDependencyBinding,
    /// Exact explicit successor review lineage label.
    pub(crate) new_relation: NewFindingRelation,
}

/// Inert full recorded link after Root actually binds every selected finding.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LinkProjection {
    /// Actual full old minimized endpoint.
    pub(crate) old: RecordedEndpoint,
    /// Actual full new minimized endpoint.
    pub(crate) new: RecordedEndpoint,
    /// Complete exact typed snapshot differences.
    pub(crate) differences: Differences,
    /// Explicitly declared direct lineage only.
    pub(crate) relation: LineageRelation,
    /// Complete requested, natively bound finding rows in ID order.
    pub(crate) findings: Vec<FindingReference>,
}

/// Complete inert denominators; Root must correlate every full array and native finding union.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SupersessionCounts {
    /// Entire old item count.
    pub(crate) old_items: u64,
    /// Entire new item count.
    pub(crate) new_items: u64,
    /// Unique declared edge occurrences.
    pub(crate) link_edges: u64,
    /// Distinct old endpoint membership.
    pub(crate) linked_old_items: u64,
    /// Distinct new endpoint membership.
    pub(crate) linked_new_items: u64,
    /// Complete old unmatched row count.
    pub(crate) unmatched_old_items: u64,
    /// Complete new unmatched row count.
    pub(crate) unmatched_new_items: u64,
    /// All repeated finding uses across links.
    pub(crate) link_finding_occurrences: u64,
    /// Distinct genuinely bound current finding IDs.
    pub(crate) distinct_linked_findings: u64,
    /// Complete native findings minus genuinely bound distinct references.
    pub(crate) unreferenced_findings: u64,
}

/// Inert separate Impact source pin; kind/UUID/schema pairing is closed by schema.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ImpactSourcePin {
    /// Root-derived role/occurrence source key within the existing 128-byte token rule.
    pub(crate) key: String,
    /// Separate fixed native read-family vocabulary.
    pub(crate) kind: ImpactSourceKind,
    /// Explicit exact admitted OSCAL UUID for native kinds, null for companions.
    #[serde(deserialize_with = "required_nullable")]
    pub(crate) native_root_uuid: Option<String>,
    /// Whole exact captured original digest.
    pub(crate) raw_sha256: String,
    /// Whole exact captured original extent.
    pub(crate) byte_length: u64,
    /// Only admitted fixed family/revision identity, never document version/href/prose.
    pub(crate) schema_identity: ImpactSchemaIdentity,
}

/// Inert exact five-field native projection after full private equality.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ImpactResource {
    /// Actual admitted Catalog/Profile family.
    pub(crate) resource_type: ImpactResourceType,
    /// Exact native `ResourceEvidence` original digest.
    pub(crate) raw_sha256: String,
    /// Exact admitted native UUID spelling; maintained identity semantics stay separate.
    pub(crate) root_uuid: String,
    /// Only the actual fixed supported OSCAL revision.
    pub(crate) oscal_version: SupportedOscalVersion,
    /// Explicit null for Catalog or exact declared Profile companion digest.
    #[serde(deserialize_with = "required_nullable")]
    pub(crate) resolved_catalog_sha256: Option<String>,
}

/// All fifteen maintained full native counts; no forged/filtered summary supplies proof.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ImpactSummary {
    /// Entire old native control denominator.
    pub(crate) old_controls: u64,
    /// Entire new native control denominator.
    pub(crate) new_controls: u64,
    /// Full added controls.
    pub(crate) added: u64,
    /// Full removed controls.
    pub(crate) removed: u64,
    /// Full content-changed controls.
    pub(crate) content_changed: u64,
    /// Full identity-migrated controls.
    pub(crate) identity_migrated: u64,
    /// Full unchanged controls.
    pub(crate) unchanged: u64,
    /// Complete native findings.
    pub(crate) findings: u64,
    /// Complete blocking findings.
    pub(crate) blocking: u64,
    /// Complete review-required findings.
    pub(crate) review_required: u64,
    /// Complete informational findings.
    pub(crate) informational: u64,
    /// Complete current resolved disposition count.
    pub(crate) dispositioned_resolved: u64,
    /// Complete current accepted-risk disposition count.
    pub(crate) dispositioned_accepted_risk: u64,
    /// Complete current still-open disposition count.
    pub(crate) dispositioned_still_open: u64,
    /// Complete current undispositioned finding count.
    pub(crate) undispositioned: u64,
}

/// Five required explicit null fields; an omitted filter is not an inferred null.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EmptyImpactFilters {
    /// Required explicit null group filter.
    #[serde(deserialize_with = "required_null")]
    pub(crate) group: (),
    /// Required explicit null `decision_state` filter.
    #[serde(deserialize_with = "required_null")]
    pub(crate) decision_state: (),
    /// Required explicit null `policy_source` filter.
    #[serde(deserialize_with = "required_null")]
    pub(crate) policy_source: (),
    /// Required explicit null priority filter.
    #[serde(deserialize_with = "required_null")]
    pub(crate) priority: (),
    /// Required explicit null owner filter.
    #[serde(deserialize_with = "required_null")]
    pub(crate) owner: (),
}

/// Inert complete recorded Impact projection; source/canonical/currentness owners are separate.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ImpactReference {
    /// Exact complete manifest raw original pin.
    pub(crate) manifest: RecordedOriginalPin,
    /// Exact complete supplied current report raw original pin.
    pub(crate) report: RecordedOriginalPin,
    /// Exact private locator raw original pin.
    pub(crate) locator: RecordedOriginalPin,
    /// Fixed maintained native report revision.
    pub(crate) schema_version: ImpactReportSchema,
    /// Recorded complete-output claim only.
    pub(crate) status: CompleteStatus,
    /// Recorded creation-fence claim only.
    pub(crate) currentness: ImpactCurrentness,
    /// Complete ordered actual comparison original roster.
    pub(crate) source_pins: Vec<ImpactSourcePin>,
    /// Actual explicit old queue framework key.
    pub(crate) old_framework_source_key: String,
    /// Actual explicit new queue framework key.
    pub(crate) new_framework_source_key: String,
    /// Required nullable exact old Profile companion key.
    #[serde(deserialize_with = "required_nullable")]
    pub(crate) old_resolved_source_key: Option<String>,
    /// Required nullable exact new Profile companion key.
    #[serde(deserialize_with = "required_nullable")]
    pub(crate) new_resolved_source_key: Option<String>,
    /// Five-field old projection after complete private native equality.
    pub(crate) old: ImpactResource,
    /// Five-field new projection after complete private native equality.
    pub(crate) new: ImpactResource,
    /// All fifteen full native checked counts.
    pub(crate) summary: ImpactSummary,
    /// All five fields explicit null.
    pub(crate) filters: EmptyImpactFilters,
    /// Complete native changes length, including Unchanged entries.
    pub(crate) change_count: u64,
    /// Complete unfiltered native finding denominator.
    pub(crate) finding_count: u64,
    /// Must equal complete `finding_count` after native binding.
    pub(crate) matched_findings: u64,
    /// Exact closed full-four-counters plus complete-findings predicate.
    pub(crate) native_change_observation: NativeChangeObservation,
    /// Unique genuinely bound current finding IDs.
    pub(crate) distinct_linked_findings: u64,
    /// Complete sorted unreferenced current IDs, excluding prior-only history.
    pub(crate) unreferenced_finding_ids: Vec<String>,
    /// Complete historical-only row count; no successor settlement or vote credit.
    pub(crate) prior_only_disposition_count: u64,
}

/// Closed inert full companion; constructing or decoding it creates no genuine proof or currentness.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SupersessionDocument {
    /// Fixed separate companion revision.
    pub(crate) schema_version: SupersessionSchema,
    /// Exact existing asserted-only limitation.
    pub(crate) identity_disclaimer: AssertedIdentity,
    /// IDs/hashes remain potentially sensitive.
    pub(crate) sensitivity: super::wire::Sensitivity,
    /// Explicit caller canonical nonnil UUID.
    pub(crate) supersession_id: String,
    /// Explicit asserted UTC seconds, not a measured native-fence timestamp.
    pub(crate) created_at: String,
    /// Direct declared lineage with no response transfer.
    pub(crate) semantics: LineageSemantics,
    /// Explicit historical/unverified old declaration label.
    pub(crate) old_queue_currentness: OldQueueCurrentness,
    /// Recorded creation-fence new claim only.
    pub(crate) new_queue_currentness: NewQueueCurrentness,
    /// Complete exact old queue raw/source/item reference.
    pub(crate) old_queue: RecordedQueueReference,
    /// Complete exact new queue raw/source/item reference.
    pub(crate) new_queue: RecordedQueueReference,
    /// Complete recorded minimized comparison and full counts.
    pub(crate) impact: ImpactReference,
    /// All unique explicit edges, in pair order.
    pub(crate) links: Vec<LinkProjection>,
    /// All actual unmatched old endpoints in queue item order.
    pub(crate) unmatched_old_items: Vec<RecordedEndpoint>,
    /// All actual unmatched new endpoints in queue item order.
    pub(crate) unmatched_new_items: Vec<RecordedEndpoint>,
    /// Every checked item/edge/native-finding denominator.
    pub(crate) counts: SupersessionCounts,
}

#[cfg(test)]
#[path = "supersession_wire_tests.rs"]
/// Prospective inert-shape/nullable/schema controls; no native/currentness authority.
mod tests;
