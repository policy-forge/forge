//! Transport-neutral, root-confined workspace queries.

use std::collections::BTreeSet;

use serde_json::{Value, json};

use super::contract::{self, Error, Result};
use super::index::{Index, Resource, Role};
use super::preparation::{NoopControl, ProgressUpdate, Stage, WorkControl, WorkError, WorkResult};
use super::root::{Captured, Root};

const MAX_CAPTURE_BYTES: usize = 50 * 1024 * 1024;
const MAX_RESOURCE_BYTES: usize = 10 * 1024 * 1024;
/// Metadata effects refuse the complete current/incoming file union before resource reads.
const MAX_EFFECT_FILES: usize = 100;

/// Contract-bounded display field lengths. Exact identity is carried by the
/// opaque provenance anchor, so truncating a display label loses no evidence.
pub(crate) const SUBJECT_LABEL_MAX: usize = 500;
const CONTROL_ID_MAX: usize = 200;
const CONTROL_TITLE_MAX: usize = 500;

pub(crate) struct Item {
    pub registration: Resource,
    pub captured: Captured,
    pub metadata: Value,
    pub validation: Value,
}

/// Complete captured workspace state, including the raw index observation used to bind effects.
pub(crate) struct Snapshot {
    pub index: Index,
    pub index_present: bool,
    /// Original raw index observation for effect binding; cursor/version semantics stay unchanged.
    captured_index: Option<Captured>,
    pub version: String,
    pub items: Vec<Item>,
    pub analysis: Option<crate::applicability::model::ApplicabilityReport>,
    mapping_queue: Vec<Value>,
}

pub(crate) fn resource_id(resource: &Resource) -> String {
    let identity = format!("{}\0{}", resource.key, resource.path);
    format!("res_{}", &crate::hashing::sha256_hex(identity.as_bytes())[..32])
}

pub(crate) fn validation(valid: bool, resource: Option<&str>) -> Value {
    let diagnostics = if valid {
        Vec::new()
    } else {
        vec![
            json!({"code":"invalid-resource", "severity":"error", "message":"The resource does not satisfy its declared input contract.", "resource_id":resource}),
        ]
    };
    json!({"state":if valid {"valid"} else {"invalid"},"error_count":diagnostics.len(),"warning_count":0,"diagnostics":diagnostics})
}

/// Admit ordered inert proposed bytes through native complete-closure validation.
pub(crate) fn validate_proposed_sources(
    index: &Index,
    sources: &[(&str, &[u8])],
    control: &mut dyn WorkControl,
) -> WorkResult<()> {
    super::source_validation::validate_proposed_sources(index, sources, control)
}

/// Validate captured bytes using intrinsic admission; report structure is not freshness.
pub(crate) fn validate_bytes(registration: &Resource, bytes: &[u8]) -> bool {
    if registration.role == Role::TraceReport {
        return super::reports::parse(bytes).is_ok_and(|report| report.kind == "trace");
    }
    if !matches!(registration.role, Role::PolicySource | Role::LifecycleSource)
        && contract::parse(bytes, MAX_RESOURCE_BYTES, 64 * 1024).is_err()
    {
        return false;
    }
    match registration.role {
        Role::PolicySource => {
            std::path::Path::new(&registration.path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
                && std::str::from_utf8(bytes).is_ok_and(|text| {
                    !text.trim().is_empty()
                        && text.bytes().filter(|byte| *byte == b'\n').take(100_001).count()
                            <= 100_000
                        && crate::parse::extract_sections(text).is_ok()
                })
        }
        Role::ApplicabilityManifest => crate::applicability::manifest::parse(bytes).is_ok(),
        Role::MappingCollection => {
            if crate::mapping::manifest::parse(bytes).is_ok() {
                return true;
            }
            validate_oscal(bytes, crate::validate::OscalModelType::Mapping)
        }
        Role::OscalCatalogArtifact => {
            validate_oscal(bytes, crate::validate::OscalModelType::Catalog)
        }
        Role::OscalComponentArtifact => {
            validate_oscal(bytes, crate::validate::OscalModelType::ComponentDefinition)
        }
        Role::ApplicabilityReport => crate::applicability::parse_stored_report(bytes).is_ok(),
        Role::TraceReport => false,
        Role::LifecycleRecord => crate::lifecycle::record::parse(bytes).is_ok(),
        Role::LifecycleSource => bytes.len() <= MAX_RESOURCE_BYTES,
        Role::OscalProfileArtifact => {
            validate_oscal(bytes, crate::validate::OscalModelType::Profile)
        }
        Role::OscalSspArtifact => {
            validate_oscal(bytes, crate::validate::OscalModelType::SystemSecurityPlan)
        }
        Role::FrameworkImpactManifest => crate::framework::manifest::parse(bytes).is_ok(),
        Role::SuccessorMap => crate::migration::parse_successor(bytes).is_ok(),
        Role::FrameworkImpactReport => {
            crate::framework::analysis::admit_prior_report(bytes).is_ok()
        }
        Role::FrameworkImpactDispositions => crate::framework::disposition::parse(bytes).is_ok(),
    }
}

fn validate_oscal(bytes: &[u8], kind: crate::validate::OscalModelType) -> bool {
    contract::parse(bytes, MAX_RESOURCE_BYTES, 64 * 1024).is_ok_and(|value| {
        crate::validate::run_full_validation("registered resource", &value, kind)
            .is_ok_and(|report| report.is_valid())
    })
}

/// Count every explicit current/incoming path and present raw index before opening resources.
fn admit_effect_paths(index: &Index, present: bool, incoming: Option<&Index>) -> Result<()> {
    let mut paths = Vec::<&str>::new();
    if present {
        paths.push(super::index::INDEX_PATH);
    }
    for registration in
        index.resources.iter().chain(incoming.into_iter().flat_map(|incoming| &incoming.resources))
    {
        let path = registration.path.as_str();
        if paths.iter().any(|old| *old != path && old.eq_ignore_ascii_case(path)) {
            return Err(Error::containment());
        }
        if !paths.contains(&path) {
            if paths.len() >= MAX_EFFECT_FILES {
                return Err(Error::new(
                    "invalid-request",
                    "The prepared effect consumes more than 100 inputs. Reduce the inputs this effect binds.",
                    false,
                ));
            }
            paths.push(path);
        }
    }
    Ok(())
}

/// Admit every planned source-restore path, including an absent index target, before reads.
fn admit_source_restore_paths(
    index: &Index,
    incoming: Option<&Index>,
    planned_output: Option<&str>,
) -> Result<()> {
    let mut paths = vec![super::index::INDEX_PATH];
    if let Some(output) = planned_output {
        super::index::validate_path(output)?;
        if output.eq_ignore_ascii_case(super::index::INDEX_PATH)
            || index.resources.iter().any(|resource| resource.path.eq_ignore_ascii_case(output))
        {
            return Err(Error::containment());
        }
        paths.push(output);
    }
    for registration in
        index.resources.iter().chain(incoming.into_iter().flat_map(|incoming| &incoming.resources))
    {
        let path = registration.path.as_str();
        if paths.iter().any(|old| *old != path && old.eq_ignore_ascii_case(path)) {
            return Err(Error::containment());
        }
        if !paths.contains(&path) {
            if paths.len() >= MAX_EFFECT_FILES {
                return Err(Error::new(
                    "payload-too-large",
                    "The complete source restore plan exceeds its input limit.",
                    false,
                ));
            }
            paths.push(path);
        }
    }
    Ok(())
}

impl Snapshot {
    /// Move every actual registered observation and the optional raw index into a private plan.
    /// Proposed/new source bytes are supplied separately and never receive these identities.
    pub(crate) fn into_captured_inputs(self) -> Vec<(String, Captured)> {
        let mut inputs: Vec<_> =
            self.items.into_iter().map(|item| (item.registration.path, item.captured)).collect();
        if let Some(index) = self.captured_index {
            inputs.push((super::index::INDEX_PATH.to_owned(), index));
        }
        inputs
    }

    /// Capture current inputs after the complete proposed path union is admitted.
    /// Incoming absent destinations are planned paths, not fabricated physical captures.
    pub(crate) fn capture_source_bundle_effect_with_control(
        root: &Root,
        incoming: Option<&Index>,
        planned_output: Option<&str>,
        control: &mut dyn WorkControl,
    ) -> WorkResult<Self> {
        Self::capture_selected(
            root,
            contract::ApiMajor::V2,
            incoming,
            true,
            true,
            planned_output,
            control,
        )
    }

    /// Borrow the already-read raw index without reopening files or changing snapshot identity.
    pub(crate) fn captured_index(&self) -> Option<&Captured> {
        self.captured_index.as_ref()
    }

    /// Capture for existing synchronous launch/query/direct-effect callers.
    /// No operation lock, numeric progress or deadline is invented by this wrapper.
    pub(crate) fn capture(root: &Root) -> Result<Self> {
        Self::capture_with_control(root, &mut NoopControl).map_err(WorkError::into_error)
    }

    /// Capture under the explicit launch major, rejecting unsupported index versions before resource reads.
    pub(crate) fn capture_for_api(root: &Root, api_major: contract::ApiMajor) -> Result<Self> {
        match api_major {
            contract::ApiMajor::V2 => Self::capture(root),
            contract::ApiMajor::V1 => {
                Self::capture_with_control_for_api(root, api_major, &mut NoopControl)
                    .map_err(WorkError::into_error)
            }
        }
    }

    /// Capture the entire ordered explicit index, checking before/after bounded
    /// reads and classification. Counts advance only after complete Item install;
    /// derived snapshot work remains indeterminate and preserves interruption.
    pub(crate) fn capture_with_control(
        root: &Root,
        control: &mut dyn WorkControl,
    ) -> WorkResult<Self> {
        Self::capture_with_control_for_api(root, contract::ApiMajor::V2, control)
    }

    /// Fence selected v1 against index2 after parsing and before any resource byte read.
    pub(crate) fn capture_with_control_for_api(
        root: &Root,
        api_major: contract::ApiMajor,
        control: &mut dyn WorkControl,
    ) -> WorkResult<Self> {
        Self::capture_selected(root, api_major, None, false, false, None, control)
    }

    /// Admit the complete bundle-effect path union after raw-index parsing and before resource reads.
    /// Queries retain their existing thousand-registration scope and cursor algorithm.
    pub(crate) fn capture_bundle_effect_with_control(
        root: &Root,
        api_major: contract::ApiMajor,
        incoming: Option<&Index>,
        control: &mut dyn WorkControl,
    ) -> WorkResult<Self> {
        if api_major != contract::ApiMajor::V2 {
            return Err(Error::invalid().into());
        }
        Self::capture_selected(root, api_major, incoming, true, false, None, control)
    }

    /// Preserve one capture implementation; only explicit bundle effects add whole-union preflight.
    fn capture_selected(
        root: &Root,
        api_major: contract::ApiMajor,
        incoming: Option<&Index>,
        effect_bound: bool,
        source_bound: bool,
        planned_output: Option<&str>,
        control: &mut dyn WorkControl,
    ) -> WorkResult<Self> {
        control.checkpoint(Stage::ReadIndex, ProgressUpdate::Unchanged)?;
        let captured_index = root.read_index();
        control.checkpoint(Stage::ReadIndex, ProgressUpdate::Unchanged)?;
        let captured_index = captured_index?;
        let index_present = captured_index.is_some();
        let index = captured_index
            .as_ref()
            .map_or_else(|| Ok(Index::empty()), |captured| Index::parse(&captured.bytes));
        control.checkpoint(Stage::ReadIndex, ProgressUpdate::Unchanged)?;
        let index = index?;
        if api_major == contract::ApiMajor::V1 && index.schema_version != "forge.workspace/1" {
            return Err(Error::new(
                "invalid-request",
                "This workspace index requires --api-major 2.",
                false,
            )
            .into());
        }
        if effect_bound {
            if source_bound {
                admit_source_restore_paths(&index, incoming, planned_output)?;
            } else {
                admit_effect_paths(&index, index_present, incoming)?;
            }
        }
        let total = index.resources.len();
        control
            .checkpoint(Stage::CaptureResource, ProgressUpdate::Capture { completed: 0, total })?;
        let mut spent = captured_index.as_ref().map_or(0, |captured| captured.bytes.len());
        let mut identities = BTreeSet::new();
        if let Some(captured) = &captured_index {
            identities.insert(captured.identity);
        }
        let mut version_input = captured_index.as_ref().map_or_else(
            || b"absent-index".to_vec(),
            |captured| captured.sha256.as_bytes().to_vec(),
        );
        let mut items = Vec::with_capacity(index.resources.len());
        for registration in &index.resources {
            control.checkpoint(Stage::CaptureResource, ProgressUpdate::Unchanged)?;
            let remaining = MAX_CAPTURE_BYTES.checked_sub(spent).ok_or_else(Error::invalid)?;
            let captured = root.read(&registration.path, remaining.min(MAX_RESOURCE_BYTES));
            control.checkpoint(Stage::CaptureResource, ProgressUpdate::Unchanged)?;
            let captured = captured?;
            spent += captured.bytes.len();
            if !identities.insert(captured.identity) {
                return Err(Error::containment().into());
            }
            let id = resource_id(registration);
            control.checkpoint(Stage::ValidateResource, ProgressUpdate::Unchanged)?;
            let valid = validate_bytes(registration, &captured.bytes);
            control.checkpoint(Stage::ValidateResource, ProgressUpdate::Unchanged)?;
            let version = captured.sha256.clone();
            version_input.extend_from_slice(
                format!("{id}:{}:{}", captured.identity.0, captured.identity.1).as_bytes(),
            );
            version_input.extend_from_slice(version.as_bytes());
            let mut metadata = json!({"resource_id":id, "key":registration.key, "role":registration.role, "path":registration.path,
                "sha256":captured.sha256, "size_bytes":captured.bytes.len(), "validation_state":if valid {"valid"} else {"invalid"}, "stale":false,"version":version});
            if let Some(profile) = registration.role.validation_profile() {
                metadata["validation_profile"] = json!(profile);
            }
            let metadata_major = if index.schema_version == "forge.workspace/2" {
                contract::ApiMajor::V2
            } else {
                contract::ApiMajor::V1
            };
            let checked_metadata = contract::validate_for(metadata_major, "Resource", &metadata);
            control.checkpoint(Stage::ValidateResource, ProgressUpdate::Unchanged)?;
            checked_metadata?;
            items.push(Item {
                registration: registration.clone(),
                captured,
                metadata,
                validation: validation(valid, Some(&id)),
            });
            control.checkpoint(
                Stage::CaptureResource,
                ProgressUpdate::Capture { completed: items.len(), total },
            )?;
        }
        control.checkpoint(Stage::SnapshotAnalysis, ProgressUpdate::Clear)?;
        let mut snapshot = Self {
            index,
            index_present,
            captured_index,
            version: crate::hashing::sha256_hex(&version_input),
            items,
            analysis: None,
            mapping_queue: Vec::new(),
        };
        snapshot.populate_analysis_with_control(control)?;
        control.checkpoint(Stage::SnapshotReports, ProgressUpdate::Clear)?;
        snapshot.mark_input_staleness();
        control.checkpoint(Stage::SnapshotReports, ProgressUpdate::Unchanged)?;
        snapshot.populate_mapping_queue_with_control(control)?;
        snapshot.validate_applicability_reports_with_control(control)?;
        snapshot.validate_trace_reports_with_control(control)?;
        control.checkpoint(Stage::SnapshotReports, ProgressUpdate::Clear)?;
        Ok(snapshot)
    }

    /// Populate captured applicability analysis while preserving sticky interruption.
    /// Ordinary domain failure still classifies its manifest as invalid.
    fn populate_analysis_with_control(&mut self, control: &mut dyn WorkControl) -> WorkResult<()> {
        if self.items.iter().any(|item| item.registration.role == Role::ApplicabilityManifest) {
            let analysis = super::domain::analyze_with_control(self, control);
            if let Err(WorkError::Interrupted(reason)) = analysis {
                return Err(WorkError::Interrupted(reason));
            }
            control.checkpoint(Stage::SnapshotAnalysis, ProgressUpdate::Unchanged)?;
            match analysis {
                Ok(analysis) => {
                    if analysis.controls.len() > 10000
                        || analysis.review_queue.len() > 10000
                        || analysis.mapping_collections.len() > 100
                    {
                        return Err(Error::invalid().into());
                    }
                    self.analysis = Some(analysis);
                }
                Err(WorkError::Failed(_)) => {
                    for item in &mut self.items {
                        if item.registration.role == Role::ApplicabilityManifest {
                            item.validation =
                                validation(false, Some(&resource_id(&item.registration)));
                            item.metadata["validation_state"] = json!("invalid");
                        }
                    }
                }
                Err(WorkError::Interrupted(reason)) => {
                    return Err(WorkError::Interrupted(reason));
                }
            }
        }
        Ok(())
    }

    /// Compare stored applicability reports at bounded report-check boundaries.
    fn validate_applicability_reports_with_control(
        &mut self,
        control: &mut dyn WorkControl,
    ) -> WorkResult<()> {
        for item in &mut self.items {
            if item.registration.role == Role::ApplicabilityReport {
                control.checkpoint(Stage::SnapshotReports, ProgressUpdate::Clear)?;
                let parsed = contract::parse(&item.captured.bytes, MAX_RESOURCE_BYTES, 64 * 1024);
                let matches = self.analysis.as_ref().is_some_and(|analysis| {
                    parsed.as_ref().is_ok_and(|value| {
                        serde_json::to_value(analysis).is_ok_and(|expected| *value == expected)
                    })
                });
                item.validation = validation(matches, Some(&resource_id(&item.registration)));
                let historical =
                    crate::applicability::parse_stored_report(&item.captured.bytes).is_ok();
                item.metadata["validation_state"] = json!(if matches {
                    "valid"
                } else if historical {
                    "stale"
                } else {
                    "invalid"
                });
                item.metadata["stale"] = json!(!matches && historical);
                control.checkpoint(Stage::SnapshotReports, ProgressUpdate::Unchanged)?;
            }
        }
        Ok(())
    }

    /// Compare stored trace reports with current capture facts. Controls surround
    /// ordinary callback-free parser/matches calls, outside Result-to-bool
    /// fallbacks, so an interruption cannot be swallowed as stale/invalid data.
    fn validate_trace_reports_with_control(
        &mut self,
        control: &mut dyn WorkControl,
    ) -> WorkResult<()> {
        let mut trace_states = Vec::new();
        for item in self.items.iter().filter(|item| item.registration.role == Role::TraceReport) {
            control.checkpoint(Stage::SnapshotReports, ProgressUpdate::Clear)?;
            let valid = super::reports::parse(&item.captured.bytes);
            control.checkpoint(Stage::SnapshotReports, ProgressUpdate::Unchanged)?;
            let matches = valid.as_ref().is_ok_and(|report| report.matches(self).unwrap_or(false));
            control.checkpoint(Stage::SnapshotReports, ProgressUpdate::Unchanged)?;
            trace_states.push((resource_id(&item.registration), valid.is_ok(), matches));
        }
        for (id, valid, matches) in trace_states {
            control.checkpoint(Stage::SnapshotReports, ProgressUpdate::Unchanged)?;
            let item = self
                .items
                .iter_mut()
                .find(|item| resource_id(&item.registration) == id)
                .ok_or_else(Error::invalid)?;
            item.validation = validation(matches, Some(&id));
            item.metadata["validation_state"] = json!(if matches {
                "valid"
            } else if valid {
                "stale"
            } else {
                "invalid"
            });
            item.metadata["stale"] = json!(valid && !matches);
        }
        Ok(())
    }

    /// Derive the existing mapping review queue while preserving controlled
    /// interruptions before any ordinary invalid-manifest fallback.
    fn populate_mapping_queue_with_control(
        &mut self,
        control: &mut dyn WorkControl,
    ) -> WorkResult<()> {
        control.checkpoint(Stage::SnapshotMapping, ProgressUpdate::Clear)?;
        let declaration = super::domain::mapping_manifest(self);
        control.checkpoint(Stage::SnapshotMapping, ProgressUpdate::Unchanged)?;
        let Ok(item) = declaration else {
            // Ambiguous declarations cannot become an empty, apparently ready queue.
            for item in &mut self.items {
                if item.registration.role == Role::MappingCollection
                    && crate::mapping::manifest::parse(&item.captured.bytes).is_ok()
                {
                    item.validation = validation(false, Some(&resource_id(&item.registration)));
                    item.metadata["validation_state"] = json!("invalid");
                }
            }
            return Ok(());
        };
        let id = resource_id(&item.registration);
        let hash = item.captured.sha256.clone();
        let built = match super::domain::mapping_with_control(self, control) {
            Ok(built) => built,
            Err(WorkError::Interrupted(reason)) => {
                return Err(WorkError::Interrupted(reason));
            }
            Err(WorkError::Failed(_)) => {
                let item = self
                    .items
                    .iter_mut()
                    .find(|item| resource_id(&item.registration) == id)
                    .ok_or_else(Error::invalid)?;
                item.validation = validation(false, Some(&id));
                item.metadata["validation_state"] = json!("invalid");
                return Ok(());
            }
        };
        control.checkpoint(Stage::SnapshotMapping, ProgressUpdate::Unchanged)?;
        let inventory = super::domain::subject_inventory_with_control(self, control)?;
        // An over-long identifier is a bounded display limitation, not a broken
        // snapshot: record it on the supplying resource instead of failing.
        for (resource, _count) in &inventory.truncated {
            if let Some(item) =
                self.items.iter_mut().find(|item| resource_id(&item.registration) == *resource)
            {
                add_warning(
                    &mut item.validation,
                    "subject-label-truncated",
                    "A subject identifier exceeds the bounded display label; its opaque provenance reference preserves the exact identifier.",
                );
            }
        }
        let subjects = &inventory.rows;
        for (side, kind, participation) in [
            ("policy", "control", &built.report.source_controls),
            ("policy", "statement", &built.report.source_statements),
            ("framework", "control", &built.report.target_controls),
            ("framework", "statement", &built.report.target_statements),
        ] {
            for subject in &participation.unmapped_ids {
                control.checkpoint(Stage::SnapshotMapping, ProgressUpdate::Unchanged)?;
                // The applicability engine already reports an eligible framework control
                // when it uses this exact framework snapshot; do not count it twice.
                if side == "framework"
                    && kind == "control"
                    && self.analysis.as_ref().is_some_and(|analysis| {
                        analysis.framework.raw_sha256 == built.report.target.raw_sha256
                            && analysis.review_queue.iter().any(|item| item.control_id == *subject)
                    })
                {
                    continue;
                }
                if self.mapping_queue.len() >= 10000 {
                    return Err(Error::invalid().into());
                }
                // The bounded display label is not identity: two identifiers that
                // share a truncated label must still select their own row. The
                // opaque `subject_id` is derived exactly as `append_subjects`
                // derives it, from the supplying resource, side, type, and the
                // exact identifier, so the lookup is an exact identity match.
                // The bounded display label is not identity: two identifiers that
                // share a truncated label must still select their own row. The
                // opaque `subject_id` is derived exactly as `append_subjects`
                // derives it, from the supplying resource, side, type, and the
                // exact identifier, so the lookup is an exact identity match.
                let reference = subjects
                    .iter()
                    .find(|row| {
                        row["resource_id"].as_str().is_some_and(|resource| {
                            row["subject_id"].as_str().is_some_and(|id| {
                                id == opaque("subj", &[side, resource, kind, subject])
                            })
                        })
                    })
                    .ok_or_else(Error::invalid)?;
                let label: String = subject.chars().take(350).collect();
                self.mapping_queue.push(json!({"item_id":opaque("qi", &[&hash,side,kind,subject]),
                    "reason_code":"no-reviewed-mapping","summary":format!("{side} {kind} {label} has no explicit reviewed mapping."),
                    "resource_id":reference["resource_id"],"evidence_refs":[reference["provenance_ref"],opaque("prov", &[&id,&hash])]}));
            }
        }
        control.checkpoint(Stage::SnapshotMapping, ProgressUpdate::Unchanged)?;
        Ok(())
    }

    pub(crate) fn item(&self, id: &str) -> Result<&Item> {
        self.items
            .iter()
            .find(|item| item.metadata["resource_id"] == id)
            .ok_or_else(|| Error::new("not-found", "The registered resource was not found.", false))
    }

    pub(crate) fn queue(&self) -> Vec<Value> {
        let mut queue: Vec<_> = self.items.iter().filter(|item| item.validation["state"] == "invalid").map(|item| {
            let identity = format!("invalid:{}:{}", resource_id(&item.registration), item.captured.sha256);
            json!({"item_id":format!("qi_{}", &crate::hashing::sha256_hex(identity.as_bytes())[..32]),
                "reason_code":if item.metadata["stale"] == true {"stale-input"} else {"invalid-resource"}, "summary":"Review the invalid or stale registered resource.", "resource_id":resource_id(&item.registration),"evidence_refs":[opaque("prov", &[&resource_id(&item.registration),&item.captured.sha256])]})
        }).collect();
        if let Some(analysis) = &self.analysis {
            for item in &analysis.review_queue {
                let identity = format!(
                    "{}:{}:{}",
                    analysis.framework.raw_sha256,
                    item.control_id,
                    item.reason_code.as_str()
                );
                queue.push(json!({"item_id":format!("qi_{}", &crate::hashing::sha256_hex(identity.as_bytes())[..32]),
                    "reason_code":item.reason_code.as_str(), "control_id":bounded_label(&item.control_id, CONTROL_ID_MAX).0,
                    "classification":analysis.controls.iter().find(|c|c.control_id == item.control_id).map(|c|c.classification),
                    "evidence_refs":[opaque("prov", &[&analysis.framework.raw_sha256,&item.control_id])],
                    "summary":match item.reason_code.as_str() {
                        "reviewed-no-positive-relationship" => "Reviewed mappings assert no positive relationship for this control.",
                        "no-reviewed-mapping" => "This applicable control has no reviewed mapping.",
                        "deferred-scope-decision" => "This control has an explicitly deferred scope decision.",
                        _ => "An explicit framework review decision is needed."
                    }}));
            }
        }
        queue.extend(self.mapping_queue.iter().cloned());
        queue.sort_by(|left, right| {
            reason_priority(left["reason_code"].as_str().unwrap_or_default())
                .cmp(&reason_priority(right["reason_code"].as_str().unwrap_or_default()))
                .then_with(|| left["item_id"].as_str().cmp(&right["item_id"].as_str()))
        });
        queue
    }

    pub(crate) fn counts(&self) -> Value {
        let queue = self.queue();
        let mut reasons = std::collections::BTreeMap::<String, usize>::new();
        for item in &queue {
            *reasons.entry(item["reason_code"].as_str().unwrap_or_default().into()).or_default() +=
                1;
        }
        json!({"total_open":queue.len(),"by_reason":reasons.into_iter().map(|(reason,count)|json!({"reason_code":reason,"count":count})).collect::<Vec<_>>()})
    }

    pub(crate) fn summary(&self) -> Value {
        let invalid =
            self.items.iter().filter(|item| item.metadata["validation_state"] == "invalid").count();
        let stale =
            self.items.iter().filter(|item| item.metadata["validation_state"] == "stale").count();
        let queue = self.queue().len();
        let setup = !self.index_present || self.items.is_empty();
        json!({"version":self.version,"project_label":self.index.label,"workspace_index_present":self.index_present,
            "health":if setup {"setup"} else if queue > 0 {"needs-attention"} else {"ready"},
            "resource_counts":{"total":self.items.len(),"valid":self.items.len()-invalid-stale,"invalid":invalid,"stale":stale,"not_validated":0},
            "review_counts":{"total_open":queue},
            "next_action":if setup {"register-resources"} else if invalid > 0 {"fix-invalid-inputs"} else if stale > 0 {"regenerate-analysis"} else if queue > 0 {"resolve-review-items"} else {"none"}})
    }

    pub(crate) fn controls(&self) -> Result<Vec<Value>> {
        let analysis = self.analysis.as_ref().ok_or_else(|| {
            Error::new(
                "validation-failed",
                "Register one valid applicability manifest and its dependencies.",
                false,
            )
        })?;
        analysis.controls.iter().map(|control| {
            let reason = analysis.review_queue.iter().find(|item| item.control_id == control.control_id).map(|item| item.reason_code.as_str());
            let state = match control.classification {
                crate::applicability::model::GapClassification::NotApplicable => "not-applicable",
                crate::applicability::model::GapClassification::Deferred => "deferred",
                crate::applicability::model::GapClassification::UnderReview => "under-review",
                _ => "applicable",
            };
            // Portable display defaults to the exact control ID, never copied framework prose.
            let control_id = bounded_label(&control.control_id, CONTROL_ID_MAX).0;
            let title = bounded_label(&control.control_id, CONTROL_TITLE_MAX).0;
            let value = json!({"control_id":control_id,"title":title,"classification":control.classification,
                "provenance_ref":opaque("prov", &[&analysis.framework.raw_sha256,&control.control_id]),"decision_state":state,"has_positive_mapping":control.positive_mapping_count>0,"review_reason":reason});
            contract::validate("ControlInventoryItem", &value)?;
            Ok(value)
        }).collect()
    }
}

/// Cursors bind offset, collection version and filters. No snapshot change can
/// silently continue an old traversal. The adapter validates allowed query keys.
pub(crate) fn paginate(
    items: Vec<Value>,
    version: &str,
    query: &[(String, String)],
) -> Result<Value> {
    let size = query
        .iter()
        .find(|(key, _)| key == "page_size")
        .map_or(Ok(50), |(_, value)| value.parse::<usize>().map_err(|_| Error::invalid()))?;
    if !(1..=200).contains(&size) {
        return Err(Error::invalid());
    }
    let mut filters: Vec<_> = query.iter().filter(|(key, _)| key != "cursor").collect();
    filters.sort();
    let filter_hash =
        crate::hashing::sha256_hex(&serde_json::to_vec(&filters).map_err(|_| Error::invalid())?);
    let offset = if let Some((_, cursor)) = query.iter().find(|(key, _)| key == "cursor") {
        let parts: Vec<_> = cursor.split(':').collect();
        if parts.len() != 3 || parts[0] != version || parts[2] != filter_hash {
            let mut error = Error::new(
                "version-conflict",
                "The collection changed. Restart from its first page.",
                true,
            );
            error.resource_version = Some(version.to_owned());
            return Err(error);
        }
        parts[1].parse::<usize>().map_err(|_| Error::invalid())?
    } else {
        0
    };
    if offset > items.len() {
        return Err(Error::invalid());
    }
    let total = items.len();
    let end = offset.saturating_add(size).min(total);
    let next = (end < total).then(|| format!("{version}:{end}:{filter_hash}"));
    Ok(
        json!({"resource_version":version,"page":{"items":items.into_iter().skip(offset).take(size).collect::<Vec<_>>(),"next_cursor":next,"total_matching":total}}),
    )
}

pub(crate) fn opaque(prefix: &str, fields: &[&str]) -> String {
    use std::fmt::Write as _;
    let mut encoded = String::new();
    for field in fields {
        let _ = write!(encoded, "{}:{field}", field.len());
    }
    format!("{prefix}_{}", &crate::hashing::sha256_hex(encoded.as_bytes())[..32])
}

/// Truncate a display field to its contract bound and report whether it was
/// truncated. A byte-short value cannot exceed the character bound.
pub(crate) fn bounded_label(value: &str, max: usize) -> (String, bool) {
    if value.len() <= max {
        return (value.to_owned(), false);
    }
    (value.chars().take(max).collect(), true)
}

/// Record a non-failing limitation in an existing validation report. Warnings
/// never change the resource's validation state.
fn add_warning(validation: &mut Value, code: &str, message: &str) {
    if let Some(diagnostics) = validation["diagnostics"].as_array_mut() {
        diagnostics.push(json!({"code":code,"severity":"warning","message":message}));
    }
    let count = validation["warning_count"].as_u64().unwrap_or(0);
    validation["warning_count"] = json!(count + 1);
}

pub(crate) fn config_status(root: &Root) -> Result<Value> {
    let Some(captured) = root.read_config()? else {
        return Ok(json!({"present":false,"valid":false,"issues":[]}));
    };
    // Validate against the canonical project root, not the process working
    // directory, so referenced project files resolve where they were registered.
    let config_path = root.project_path().join(".forge.toml");
    let valid = std::str::from_utf8(&captured.bytes).is_ok_and(|text| {
        crate::config::parse_and_validate(&config_path, crate::config::SourceKind::Discovered, text)
            .is_ok()
    });
    Ok(
        json!({"present":true,"valid":valid,"issues":if valid { vec![] } else {vec!["The project configuration is invalid."]}}),
    )
}

pub(crate) fn filtered(
    mut items: Vec<Value>,
    query: &[(String, String)],
    keys: &[&str],
) -> Vec<Value> {
    items.retain(|item| {
        query.iter().all(|(key, value)| {
            !keys.contains(&key.as_str())
                || if let Some(boolean) = item[key].as_bool() {
                    value == if boolean { "true" } else { "false" }
                } else {
                    item[key] == value.as_str()
                }
        })
    });
    items
}

impl Snapshot {
    pub(crate) fn validate_selection(&self, request: &Value) -> Result<Value> {
        let selected = request["scope"] == "selected";
        let ids = request["resource_ids"].as_array();
        if selected && ids.is_none_or(Vec::is_empty) {
            return Err(Error::invalid());
        }
        if let Some(ids) = ids {
            for id in ids {
                self.item(id.as_str().ok_or_else(Error::invalid)?)?;
            }
        }
        let mut diagnostics = Vec::new();
        for item in &self.items {
            if selected && !ids.is_some_and(|ids| ids.contains(&item.metadata["resource_id"])) {
                continue;
            }
            if let Some(issues) = item.validation["diagnostics"].as_array() {
                diagnostics.extend(issues.iter().cloned());
            }
        }
        if diagnostics.len() > 500 {
            return Err(Error::invalid());
        }
        let errors =
            diagnostics.iter().filter(|diagnostic| diagnostic["severity"] == "error").count();
        Ok(
            json!({"state":if errors == 0 {"valid"} else {"invalid"},"error_count":errors,"warning_count":diagnostics.len()-errors,"diagnostics":diagnostics}),
        )
    }

    pub(crate) fn classification_counts(&self) -> Result<Value> {
        let analysis = self.analysis.as_ref().ok_or_else(Error::invalid)?;
        let c = &analysis.counts;
        Ok(json!([
            {"classification":"applicable-mapped","count":c.applicable_mapped},
            {"classification":"applicable-reviewed-no-relationship","count":c.applicable_reviewed_no_relationship},
            {"classification":"applicable-unmapped","count":c.applicable_unmapped},
            {"classification":"not-applicable","count":c.not_applicable},
            {"classification":"deferred","count":c.deferred},
            {"classification":"under-review","count":c.under_review}
        ]))
    }

    pub(crate) fn report_view(&self) -> Result<Value> {
        let item = super::domain::selected(self, Role::ApplicabilityReport)?;
        let historical =
            crate::applicability::parse_stored_report(&item.captured.bytes).map_err(|_| {
                Error::new("validation-failed", "The committed report is invalid.", false)
            })?;
        let fingerprints = self.report_input_fingerprints(&historical)?;
        let c = &historical.counts;
        let counts = json!([
            {"classification":"applicable-mapped","count":c.applicable_mapped},
            {"classification":"applicable-reviewed-no-relationship","count":c.applicable_reviewed_no_relationship},
            {"classification":"applicable-unmapped","count":c.applicable_unmapped},
            {"classification":"not-applicable","count":c.not_applicable},
            {"classification":"deferred","count":c.deferred},
            {"classification":"under-review","count":c.under_review}]);
        Ok(
            json!({"version":item.metadata["version"],"stale":item.metadata["stale"],"input_fingerprints":fingerprints,"classification_counts":counts,"eligible_controls":c.total}),
        )
    }

    /// Compare the committed report's recorded input fingerprints with the
    /// current captured resources. Rows carry the opaque resource id, so callers
    /// can mark exactly the inputs that no longer match.
    fn report_input_fingerprints(
        &self,
        historical: &crate::applicability::model::ApplicabilityReport,
    ) -> Result<Vec<Value>> {
        let manifest = super::domain::selected(self, Role::ApplicabilityManifest)?;
        let mut fingerprints = vec![
            json!({"resource_id":resource_id(&manifest.registration),"sha256":historical.manifest_sha256,"matches_current":historical.manifest_sha256==manifest.captured.sha256}),
        ];
        if historical.manifest_sha256 == manifest.captured.sha256 {
            let parsed = crate::applicability::manifest::parse(&manifest.captured.bytes)
                .map_err(|_| Error::invalid())?;
            let framework_path = super::domain::resolve_reference(
                &manifest.registration.path,
                &parsed.framework.artifact,
            )?;
            let framework = self
                .items
                .iter()
                .find(|item| item.registration.path == framework_path)
                .ok_or_else(Error::invalid)?;
            fingerprints.push(json!({"resource_id":resource_id(&framework.registration),"sha256":historical.framework.raw_sha256,"matches_current":historical.framework.raw_sha256==framework.captured.sha256}));
            for reference in &parsed.mapping_collections {
                let path =
                    super::domain::resolve_reference(&manifest.registration.path, reference)?;
                let item = self
                    .items
                    .iter()
                    .find(|item| item.registration.path == path)
                    .ok_or_else(Error::invalid)?;
                let value = contract::parse(&item.captured.bytes, MAX_RESOURCE_BYTES, 64 * 1024)?;
                if let Some(evidence) = historical
                    .mapping_collections
                    .iter()
                    .find(|evidence| value["mapping-collection"]["uuid"] == evidence.uuid)
                {
                    fingerprints.push(json!({"resource_id":resource_id(&item.registration),"sha256":evidence.raw_sha256,"matches_current":evidence.raw_sha256==item.captured.sha256}));
                }
            }
            if fingerprints.len() > 100 {
                return Err(Error::invalid());
            }
        }
        Ok(fingerprints)
    }

    /// A resource that supplied an input to the committed report and whose bytes
    /// no longer match the recorded fingerprint is stale. Best-effort: an
    /// absent, ambiguous, or unparsable report leaves the default state.
    fn mark_input_staleness(&mut self) {
        let Ok(report) = super::domain::selected(self, Role::ApplicabilityReport) else {
            return;
        };
        let Ok(historical) = crate::applicability::parse_stored_report(&report.captured.bytes)
        else {
            return;
        };
        let Ok(fingerprints) = self.report_input_fingerprints(&historical) else {
            return;
        };
        let stale: Vec<String> = fingerprints
            .iter()
            .filter(|fingerprint| fingerprint["matches_current"] == false)
            .filter_map(|fingerprint| fingerprint["resource_id"].as_str().map(str::to_owned))
            .collect();
        for id in stale {
            if let Some(item) =
                self.items.iter_mut().find(|item| resource_id(&item.registration) == id)
            {
                // An already-invalid input keeps its precise diagnostics.
                if item.metadata["validation_state"] != "invalid" {
                    item.validation = validation(false, Some(&id));
                    item.metadata["validation_state"] = json!("stale");
                    item.metadata["stale"] = json!(true);
                }
            }
        }
    }

    pub(crate) fn analysis_inputs(&self) -> Result<Vec<&Item>> {
        let manifest = super::domain::selected(self, Role::ApplicabilityManifest)?;
        let parsed = crate::applicability::manifest::parse(&manifest.captured.bytes)
            .map_err(|_| Error::invalid())?;
        let mut paths = vec![manifest.registration.path.clone()];
        for reference in std::iter::once(&parsed.framework.artifact)
            .chain(parsed.framework.resolved_catalog.iter())
            .chain(parsed.mapping_collections.iter())
        {
            paths.push(super::domain::resolve_reference(&manifest.registration.path, reference)?);
        }
        if paths.len() > 100 {
            return Err(Error::invalid());
        }
        paths
            .iter()
            .map(|path| {
                self.items
                    .iter()
                    .find(|item| item.registration.path == *path)
                    .ok_or_else(Error::invalid)
            })
            .collect()
    }
}

fn reason_priority(reason: &str) -> usize {
    [
        "invalid-resource",
        "stale-input",
        "external-conflict",
        "scope-decision-required",
        "deferred-scope-decision",
        "no-reviewed-mapping",
        "reviewed-no-positive-relationship",
    ]
    .iter()
    .position(|r| *r == reason)
    .unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Construct valid explicit registrations whose absent files must never be opened by a refused effect.
    fn effect_preflight_index(prefix: &str, count: usize) -> Index {
        Index {
            resources: (0..count)
                .map(|n| Resource {
                    key: format!("{prefix}-{n}"),
                    role: Role::PolicySource,
                    path: format!("{prefix}-{n}.md"),
                })
                .collect(),
            ..Index::empty()
        }
    }

    /// Current plus raw index and current/incoming union overflow refuse before current resource reads.
    #[test]
    fn bundle_effect_capture_preflights_complete_union_before_resource_reads() {
        for (current_count, incoming_count, present) in
            [(100, 0, true), (50, 50, true), (0, 101, false)]
        {
            let directory = tempfile::tempdir().unwrap();
            let current = effect_preflight_index("current", current_count);
            if present {
                std::fs::write(
                    directory.path().join(super::super::index::INDEX_PATH),
                    current.bytes().unwrap(),
                )
                .unwrap();
            }
            let root = Root::open(directory.path()).unwrap();
            let incoming = effect_preflight_index("incoming", incoming_count);
            let mut observer = super::super::preparation::test_support::Recorder::default();
            let result = Snapshot::capture_bundle_effect_with_control(
                &root,
                contract::ApiMajor::V2,
                Some(&incoming),
                &mut observer,
            );
            assert!(
                matches!(result,Err(WorkError::Failed(ref error)) if error.code=="invalid-request")
            );
            assert!(!observer.events.iter().any(|(stage, _)| *stage == Stage::CaptureResource));
            if present {
                assert_eq!(
                    Snapshot::capture_for_api(&root, contract::ApiMajor::V2).err().unwrap().code,
                    "not-found"
                );
            }
        }
    }

    /// Cross-index case aliases refuse before even the current missing source can be read.
    #[test]
    fn bundle_effect_capture_preflights_cross_index_aliases() {
        let directory = tempfile::tempdir().unwrap();
        let current = effect_preflight_index("current", 1);
        std::fs::write(
            directory.path().join(super::super::index::INDEX_PATH),
            current.bytes().unwrap(),
        )
        .unwrap();
        let root = Root::open(directory.path()).unwrap();
        let mut incoming = current.clone();
        incoming.resources[0].path = incoming.resources[0].path.to_ascii_uppercase();
        let mut observer = super::super::preparation::test_support::Recorder::default();
        let result = Snapshot::capture_bundle_effect_with_control(
            &root,
            contract::ApiMajor::V2,
            Some(&incoming),
            &mut observer,
        );
        assert!(
            matches!(result,Err(WorkError::Failed(ref error)) if error.code=="resource-containment")
        );
        assert!(!observer.events.iter().any(|(stage, _)| *stage == Stage::CaptureResource));
    }

    #[test]
    fn setup_never_scans_unregistered_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("private.md"), "never registered").unwrap();
        let snapshot = Snapshot::capture(&Root::open(dir.path()).unwrap()).unwrap();
        assert!(snapshot.items.is_empty());
        assert_eq!(snapshot.summary()["health"], "setup");
        contract::validate("ProjectSummary", &snapshot.summary()).unwrap();
        contract::validate("ReviewQueueCounts", &snapshot.counts()).unwrap();
    }

    #[test]
    fn cursor_changes_fail_and_pages_preserve_total() {
        let values = vec![json!(1), json!(2), json!(3)];
        let query = vec![("page_size".into(), "2".into())];
        let first = paginate(values.clone(), "12345678", &query).unwrap();
        let mut next = query;
        next.push(("cursor".into(), first["page"]["next_cursor"].as_str().unwrap().into()));
        let second = paginate(values.clone(), "12345678", &next).unwrap();
        assert_eq!(second["page"]["items"], json!([3]));
        assert_eq!(second["page"]["total_matching"], 3);
        assert!(paginate(values, "87654321", &next).is_err());
    }

    fn fixture_item(role: Role, path: &str, bytes: Vec<u8>) -> Item {
        let registration = Resource {
            key: path.trim_end_matches(".json").replace('/', "-"),
            role,
            path: path.to_owned(),
        };
        let id = resource_id(&registration);
        let sha256 = crate::hashing::sha256_hex(&bytes);
        let metadata = json!({"resource_id":id, "key":registration.key, "role":registration.role, "path":registration.path,
            "sha256":sha256, "size_bytes":bytes.len(), "validation_state":"valid", "stale":false, "version":sha256});
        Item {
            registration,
            captured: Captured { bytes, identity: (1, 1), sha256 },
            metadata,
            validation: validation(true, Some(&id)),
        }
    }

    #[test]
    fn config_status_validates_against_the_project_root() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("profiles")).unwrap();
        std::fs::write(dir.path().join("profiles/source.json"), "{}").unwrap();
        std::fs::write(
            dir.path().join(".forge.toml"),
            "schema-version = 1\n[convert]\nsource-profile = \"profiles/source.json\"\n",
        )
        .unwrap();
        let status = config_status(&Root::open(dir.path()).unwrap()).unwrap();
        assert_eq!(status["present"], true);
        assert_eq!(status["valid"], true, "{status}");
    }

    /// Report freshness uses captured resource facts; this manual view grants no raw-index binding.
    #[test]
    fn changed_committed_report_input_is_stale_and_filterable() {
        let manifest_bytes =
            include_str!("../../examples/authoring/applicability.json").as_bytes().to_vec();
        let manifest_sha = crate::hashing::sha256_hex(&manifest_bytes);
        let mut report: Value =
            serde_json::from_str(include_str!("../../examples/authoring/gap-report.json")).unwrap();
        report["manifest_sha256"] = json!(manifest_sha);
        report["framework"]["raw_sha256"] = json!("b".repeat(64));
        let report_bytes = serde_json::to_vec(&report).unwrap();
        let mut snapshot = Snapshot {
            index: Index::empty(),
            index_present: true,
            captured_index: None,
            version: "v".to_owned(),
            items: vec![
                fixture_item(Role::ApplicabilityManifest, "scope.json", manifest_bytes),
                fixture_item(
                    Role::OscalCatalogArtifact,
                    "framework.json",
                    b"changed framework bytes".to_vec(),
                ),
                fixture_item(Role::ApplicabilityReport, "applicability-report.json", report_bytes),
            ],
            analysis: None,
            mapping_queue: Vec::new(),
        };
        snapshot.mark_input_staleness();
        let framework =
            snapshot.items.iter().find(|item| item.registration.path == "framework.json").unwrap();
        assert_eq!(framework.metadata["stale"], true);
        assert_eq!(framework.metadata["validation_state"], "stale");
        assert_eq!(framework.validation["state"], "invalid");
        let listed = filtered(
            vec![framework.metadata.clone()],
            &[("stale".into(), "true".into())],
            &["role", "validation_state", "stale"],
        );
        assert_eq!(listed.len(), 1);
        let manifest =
            snapshot.items.iter().find(|item| item.registration.path == "scope.json").unwrap();
        assert_eq!(manifest.metadata["stale"], false);
        let report_item = snapshot
            .items
            .iter()
            .find(|item| item.registration.path == "applicability-report.json")
            .unwrap();
        assert_eq!(report_item.metadata["stale"], false);
    }

    #[test]
    fn warnings_do_not_invalidate_a_validation_report() {
        let id = "res_0123456789abcdef0123456789abcdef".to_owned();
        let mut report = validation(true, Some(&id));
        add_warning(
            &mut report,
            "subject-label-truncated",
            "A subject identifier exceeds the bounded display label.",
        );
        assert_eq!(report["state"], "valid");
        assert_eq!(report["error_count"], 0);
        assert_eq!(report["warning_count"], 1);
        contract::validate("ValidationReport", &report).unwrap();
        let (bounded, truncated) = bounded_label(&"c".repeat(600), SUBJECT_LABEL_MAX);
        assert!(truncated);
        assert_eq!(bounded.chars().count(), SUBJECT_LABEL_MAX);
        let (short, truncated) = bounded_label("res_short", SUBJECT_LABEL_MAX);
        assert!(!truncated);
        assert_eq!(short, "res_short");
    }

    /// Domain-valid large mapping bytes retain their original admission without an effect index capture.
    #[test]
    fn domain_valid_large_mapping_manifest_is_accepted() {
        let mut reviewers = Vec::new();
        let mut keys = Vec::new();
        for index in 0..25 {
            let key = format!("r{index}");
            keys.push(key.clone());
            reviewers.push(json!({"key":key,"type":"person","name":"n".repeat(60_000)}));
        }
        let manifest = json!({
            "schema_version":"forge.mapping-manifest/1",
            "collection":{"key":"collection","title":"Collection","version":"1","last_modified":"2026-09-10T00:00:00Z"},
            "reviewers":reviewers,
            "provenance":{"method":"human","matching_rationale":"semantic","status":"complete","mapping_description":"Reviewed mapping.","reviewer_keys":keys,"reviewed_at":"2026-09-10T00:00:00Z"},
            "mapping":{"key":"mapping",
                "source":{"type":"catalog","artifact":"source.json","href":"source.json"},
                "target":{"type":"catalog","artifact":"target.json","href":"target.json"},
                "maps":[{"key":"map-1","relationship":"no-relationship","sources":[{"type":"control","id_ref":"a"}],"targets":[{"type":"control","id_ref":"b"}],"reviewer_key":"r0","reviewed_at":"2026-09-10T00:00:00Z","rationale":"Explicit review."}]}
        });
        let bytes = serde_json::to_vec(&manifest).unwrap();
        assert!(
            bytes.len() > 1024 * 1024
                && bytes.len()
                    < usize::try_from(crate::mapping::manifest::MAX_MANIFEST_BYTES).unwrap(),
            "{}",
            bytes.len()
        );
        crate::mapping::manifest::parse(&bytes).expect("domain-valid mapping manifest");
        let registration = Resource {
            key: "mapping".to_owned(),
            role: Role::MappingCollection,
            path: "mapping.json".to_owned(),
        };
        assert!(validate_bytes(&registration, &bytes));
        let snapshot = Snapshot {
            index: Index::empty(),
            index_present: true,
            captured_index: None,
            version: "v".to_owned(),
            items: vec![fixture_item(Role::MappingCollection, "mapping.json", bytes)],
            analysis: None,
            mapping_queue: Vec::new(),
        };
        assert!(crate::workspace::domain::mapping_manifest(&snapshot).is_ok());
    }

    fn catalog(controls: &[&str], uuid: &str) -> Vec<u8> {
        serde_json::to_vec(&json!({"catalog":{"uuid":uuid,
            "metadata":{"title":"Synthetic catalog","last-modified":"2026-09-10T00:00:00Z","version":"1","oscal-version":"1.2.3"},
            "controls":controls.iter().map(|id| json!({"id":id,"title":"Synthetic control"})).collect::<Vec<_>>()}}))
        .unwrap()
    }

    /// Two identifiers can share the 500-character bounded label while remaining
    /// distinct. The mapping queue must resolve each to its own inventory row
    /// (and therefore its own opaque provenance reference), never the first row
    /// that happens to display the same bounded label.
    #[test]
    fn mapping_queue_matches_subjects_by_identity_not_by_bounded_label() {
        let shared = "c".repeat(600);
        let first = format!("{shared}a");
        let second = format!("{shared}b");
        let source_bytes =
            catalog(&[&first, &second, "mapped-policy"], "11111111-1111-4111-8111-111111111111");
        let target_bytes = catalog(&["framework-a"], "22222222-2222-4222-8222-222222222222");
        let source = fixture_item(Role::OscalCatalogArtifact, "source.json", source_bytes.clone());
        let target = fixture_item(Role::OscalCatalogArtifact, "target.json", target_bytes.clone());
        let source_id = resource_id(&source.registration);
        let target_id = resource_id(&target.registration);
        let catalog_snapshot = Snapshot {
            index: Index::empty(),
            index_present: true,
            captured_index: None,
            version: "v".to_owned(),
            items: vec![source, target],
            analysis: None,
            mapping_queue: Vec::new(),
        };
        let manifest = crate::workspace::domain::initialize(
            &catalog_snapshot,
            &json!({
                "target_path":"mapping.json",
                "source_resource_id":source_id,
                "target_resource_id":target_id,
                "scope":"control-only",
                "maps":[{"key":"none","relationship":"no-relationship","sources":[{"type":"control","id_ref":"mapped-policy"}],"targets":[{"type":"control","id_ref":"framework-a"}],"reviewer_key":"reviewer","reviewed_at":"2026-09-10T00:00:00Z","rationale":"Explicit initial review."}],
                "review":{"collection":{"key":"synthetic-map","title":"Synthetic mapping","version":"1","last_modified":"2026-09-10T00:00:00Z"},
                    "reviewers":[{"key":"reviewer","type":"person","name":"Synthetic Reviewer"}],
                    "provenance":{"method":"human","matching_rationale":"semantic","status":"draft","mapping_description":"Explicit synthetic review.","reviewer_keys":["reviewer"],"reviewed_at":"2026-09-10T00:00:00Z"}}
            }),
            true,
        )
        .unwrap();
        let mut snapshot = Snapshot {
            index: Index::empty(),
            index_present: true,
            captured_index: None,
            version: "v".to_owned(),
            items: vec![
                fixture_item(Role::MappingCollection, "mapping.json", manifest),
                fixture_item(Role::OscalCatalogArtifact, "source.json", source_bytes),
                fixture_item(Role::OscalCatalogArtifact, "target.json", target_bytes),
            ],
            analysis: None,
            mapping_queue: Vec::new(),
        };
        snapshot.populate_mapping_queue_with_control(&mut NoopControl).unwrap();
        let rows: Vec<Value> = snapshot
            .queue()
            .into_iter()
            .filter(|item| item["reason_code"] == "no-reviewed-mapping")
            .collect();
        assert_eq!(rows.len(), 2, "{rows:?}");
        assert_eq!(rows[0]["resource_id"], rows[1]["resource_id"]);
        assert_ne!(
            rows[0]["evidence_refs"][0], rows[1]["evidence_refs"][0],
            "each subject must resolve to its own provenance reference: {rows:?}"
        );
    }

    /// Create a synthetic explicit ordered Markdown index with optional invalid
    /// captured content. Files are real confined inputs; no authentic review
    /// decisions or filesystem identities are fabricated.
    fn capture_fixture(count: usize, invalid_last: bool) -> (tempfile::TempDir, Root) {
        let dir = tempfile::tempdir().unwrap();
        let mut resources = Vec::new();
        for index in 0..count {
            let path = format!("source-{index}.md");
            let bytes = if invalid_last && index + 1 == count {
                &b""[..]
            } else {
                &b"# Synthetic policy\n\nStaff must review proposed changes.\n"[..]
            };
            std::fs::write(dir.path().join(&path), bytes).unwrap();
            resources
                .push(json!({"key":format!("source-{index}"),"role":"policy-source","path":path}));
        }
        std::fs::write(
            dir.path().join(super::super::index::INDEX_PATH),
            serde_json::to_vec(&json!({"schema_version":"forge.workspace/1","label":"Synthetic checkpoint fixture","resources":resources})).unwrap(),
        ).unwrap();
        let root = Root::open(dir.path()).unwrap();
        (dir, root)
    }

    /// Cancellation at the first real capture boundary prevents even malformed
    /// index input from being opened/parsed; ordinary capture still rejects it.
    #[test]
    fn controlled_capture_interrupts_before_index_io() {
        use super::super::preparation::{Interruption, test_support::Recorder};
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(super::super::index::INDEX_PATH), "not JSON").unwrap();
        let root = Root::open(dir.path()).unwrap();
        let mut control = Recorder::at(Stage::ReadIndex, 1);
        assert!(matches!(
            Snapshot::capture_with_control(&root, &mut control),
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        ));
        assert_eq!(control.events, vec![(Stage::ReadIndex, ProgressUpdate::Unchanged)]);
        let ordinary = Snapshot::capture(&root).err().unwrap();
        assert_eq!(ordinary.code, "invalid-request");
    }

    /// Progress counts all installed registrations through the full1,000-entry
    /// bound, including classified-invalid bytes, then clears for derived work.
    #[test]
    fn controlled_capture_reports_complete_registration_denominator() {
        use super::super::preparation::test_support::Recorder;
        for count in [0, 1, 3, 1000] {
            let (_dir, root) = capture_fixture(count, count == 3);
            let mut control = Recorder::default();
            let snapshot = Snapshot::capture_with_control(&root, &mut control).unwrap();
            assert_eq!(snapshot.items.len(), count);
            let facts: Vec<_> = control
                .events
                .iter()
                .filter_map(|(_, update)| match update {
                    ProgressUpdate::Capture { completed, total } => Some((*completed, *total)),
                    _ => None,
                })
                .collect();
            assert_eq!(facts, (0..=count).map(|completed| (completed, count)).collect::<Vec<_>>());
            assert_eq!(control.events.last().unwrap().1, ProgressUpdate::Clear);
            if count == 3 {
                assert_eq!(snapshot.items.last().unwrap().metadata["validation_state"], "invalid");
            }
            let ordinary = Snapshot::capture(&root).unwrap();
            assert_eq!(snapshot.version, ordinary.version);
            assert_eq!(snapshot.summary(), ordinary.summary());
        }
        let dir = tempfile::tempdir().unwrap();
        let mut control = Recorder::default();
        let absent =
            Snapshot::capture_with_control(&Root::open(dir.path()).unwrap(), &mut control).unwrap();
        assert!(!absent.index_present);
        assert_eq!(absent.items.len(), 0);
        assert!(control.events.contains(&(
            Stage::CaptureResource,
            ProgressUpdate::Capture { completed: 0, total: 0 }
        )));
    }

    /// After one complete capture, cooperative cancellation wins before the next
    /// declared missing input. Ordinary capture proves that next read would fail.
    #[test]
    fn controlled_capture_stops_before_next_registered_read() {
        use super::super::preparation::{Interruption, test_support::Recorder};
        let (dir, root) = capture_fixture(2, false);
        std::fs::remove_file(dir.path().join("source-1.md")).unwrap();
        let mut control = Recorder::before_read_after(1);
        assert!(matches!(
            Snapshot::capture_with_control(&root, &mut control),
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        ));
        assert!(control.events.contains(&(
            Stage::CaptureResource,
            ProgressUpdate::Capture { completed: 1, total: 2 }
        )));
        assert!(!control.events.contains(&(
            Stage::CaptureResource,
            ProgressUpdate::Capture { completed: 2, total: 2 }
        )));
        assert_eq!(Snapshot::capture(&root).err().unwrap().code, "not-found");
        assert_eq!(
            std::fs::read(dir.path().join("source-0.md")).unwrap(),
            b"# Synthetic policy\n\nStaff must review proposed changes.\n"
        );
    }

    /// An interruption from the controlled applicability engine's staging must
    /// escape the snapshot's normal invalid-manifest fallback.
    #[test]
    fn controlled_capture_preserves_analysis_stage_interruption() {
        use super::super::preparation::{Interruption, test_support::Recorder};
        let dir = tempfile::tempdir().unwrap();
        let catalog = json!({"catalog":{"uuid":"11111111-1111-4111-8111-111111111111","metadata":{"title":"Synthetic catalog","last-modified":"2026-09-10T00:00:00Z","version":"1","oscal-version":"1.2.3"},"controls":[{"id":"control-a","title":"Synthetic control"}]}});
        std::fs::write(dir.path().join("framework.json"), serde_json::to_vec(&catalog).unwrap())
            .unwrap();
        let mut index = json!({"schema_version":"forge.workspace/1","label":"Synthetic analysis checkpoint","resources":[{"key":"framework","role":"oscal-catalog-artifact","path":"framework.json"}]});
        std::fs::write(
            dir.path().join(super::super::index::INDEX_PATH),
            serde_json::to_vec(&index).unwrap(),
        )
        .unwrap();
        let root = Root::open(dir.path()).unwrap();
        let initial = Snapshot::capture(&root).unwrap();
        let bytes = super::super::domain::initialize(&initial,
            &json!({"target_path":"scope.json","framework_resource_id":initial.items[0].metadata["resource_id"]}), false).unwrap();
        std::fs::write(dir.path().join("scope.json"), bytes).unwrap();
        index["resources"]
            .as_array_mut()
            .unwrap()
            .push(json!({"key":"scope","role":"applicability-manifest","path":"scope.json"}));
        std::fs::write(
            dir.path().join(super::super::index::INDEX_PATH),
            serde_json::to_vec(&index).unwrap(),
        )
        .unwrap();
        let ordinary = Snapshot::capture(&root).unwrap();
        assert!(ordinary.analysis.is_some());
        let mut control = Recorder::at(Stage::CopyInputs, 3);
        assert!(matches!(
            Snapshot::capture_with_control(&root, &mut control),
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        ));
        assert!(control.events.contains(&(
            Stage::CaptureResource,
            ProgressUpdate::Capture { completed: 2, total: 2 }
        )));
    }

    /// Normal malformed mapping input remains classified invalid, while a stop
    /// after its declaration check bypasses that best-effort empty-queue path.
    #[test]
    fn controlled_capture_preserves_mapping_fallback_interruption() {
        use super::super::preparation::{Interruption, test_support::Recorder};
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("mapping.json"), "{}\n").unwrap();
        std::fs::write(dir.path().join(super::super::index::INDEX_PATH), serde_json::to_vec(&json!({"schema_version":"forge.workspace/1","label":"Synthetic invalid mapping","resources":[{"key":"mapping","role":"mapping-collection","path":"mapping.json"}]})).unwrap()).unwrap();
        let root = Root::open(dir.path()).unwrap();
        let ordinary = Snapshot::capture(&root).unwrap();
        assert_eq!(ordinary.items[0].metadata["validation_state"], "invalid");
        let mut control = Recorder::at(Stage::SnapshotMapping, 2);
        assert!(matches!(
            Snapshot::capture_with_control(&root, &mut control),
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        ));
    }

    /// Trace freshness controls surround actual callback-free parse/matches work,
    /// so both pre-readiness and post-comparison cancellation remain interruptions.
    #[test]
    fn controlled_trace_freshness_keeps_interruption_outside_fallbacks() {
        use super::super::preparation::{Interruption, test_support::Recorder};
        let (dir, root) = capture_fixture(1, false);
        let initial = Snapshot::capture(&root).unwrap();
        let report = super::super::reports::render(
            &initial,
            "trace",
            super::super::domain::trace_counts(&initial).unwrap(),
        )
        .unwrap();
        std::fs::write(dir.path().join("trace.html"), report).unwrap();
        let mut index = initial.index;
        index.resources.push(Resource {
            key: "trace".into(),
            role: Role::TraceReport,
            path: "trace.html".into(),
        });
        std::fs::write(dir.path().join(super::super::index::INDEX_PATH), index.bytes().unwrap())
            .unwrap();
        for visit in [1, 3] {
            let mut snapshot = Snapshot::capture(&root).unwrap();
            assert_eq!(snapshot.items[1].metadata["validation_state"], "valid");
            let mut control = Recorder::at(Stage::SnapshotReports, visit);
            assert!(matches!(
                snapshot.validate_trace_reports_with_control(&mut control),
                Err(WorkError::Interrupted(Interruption::CancelRequested))
            ));
        }
    }
}

#[cfg(test)]
mod source_plan_capture_controls {
    use super::super::preparation::test_support::Recorder;
    use super::*;

    /// Parse explicit index2 paths without creating or claiming their target files.
    fn registered(paths: &[String]) -> Index {
        Index::parse(&serde_json::to_vec(&json!({"schema_version":"forge.workspace/2","label":"Synthetic planned source paths",
            "resources":paths.iter().enumerate().map(|(position,path)|json!({"key":format!("source-{position}"),"role":"lifecycle-source","path":path})).collect::<Vec<_>>()
        })).unwrap()).unwrap()
    }

    /// An absent index still consumes one planned restore path, while new files remain uncaptured.
    #[test]
    fn source_plan_absent_index_fits_99_and_refuses_100_before_resource_reads() {
        let project = tempfile::tempdir().unwrap();
        let root = Root::open(project.path()).unwrap();
        let paths: Vec<_> = (0..100).map(|position| format!("new-{position}.dat")).collect();
        let incoming = registered(&paths[..99]);
        let captured = Snapshot::capture_source_bundle_effect_with_control(
            &root,
            Some(&incoming),
            None,
            &mut NoopControl,
        )
        .unwrap();
        assert!(!captured.index_present);
        assert_eq!(captured.items.len(), 0);
        assert_eq!(captured.into_captured_inputs().len(), 0);
        let incoming = registered(&paths);
        let mut control = Recorder::default();
        assert!(
            matches!(Snapshot::capture_source_bundle_effect_with_control(&root,Some(&incoming),None,&mut control),Err(WorkError::Failed(error)) if error.code=="payload-too-large")
        );
        assert!(!control.events.iter().any(|(stage, _)| *stage == Stage::CaptureResource));
        assert!(!project.path().join(super::super::index::INDEX_PATH).exists());
    }

    /// Current registrations plus incoming paths are counted together before a missing current read.
    #[test]
    fn source_plan_union_overflow_precedes_registered_byte_reads() {
        let project = tempfile::tempdir().unwrap();
        let current = registered(&["missing-current.dat".to_owned()]);
        std::fs::write(
            project.path().join(super::super::index::INDEX_PATH),
            current.bytes().unwrap(),
        )
        .unwrap();
        let root = Root::open(project.path()).unwrap();
        let incoming = registered(
            &(0..99).map(|position| format!("incoming-{position}.dat")).collect::<Vec<_>>(),
        );
        let mut control = Recorder::default();
        assert!(
            matches!(Snapshot::capture_source_bundle_effect_with_control(&root,Some(&incoming),None,&mut control),Err(WorkError::Failed(error)) if error.code=="payload-too-large")
        );
        assert!(!control.events.iter().any(|(stage, _)| *stage == Stage::CaptureResource));
    }

    /// Move extraction preserves actual native identities, binary bytes and present raw index.
    #[test]
    fn source_plan_owned_inputs_retain_native_generations_without_proposed_identities() {
        let project = tempfile::tempdir().unwrap();
        let index = registered(&["source.dat".to_owned()]);
        let bytes = b"\0\xff\xef\xbb\xbf\r\n";
        std::fs::write(project.path().join("source.dat"), bytes).unwrap();
        std::fs::write(
            project.path().join(super::super::index::INDEX_PATH),
            index.bytes().unwrap(),
        )
        .unwrap();
        let root = Root::open(project.path()).unwrap();
        let original = root.read("source.dat", 1024).unwrap();
        let captured = Snapshot::capture_source_bundle_effect_with_control(
            &root,
            Some(&index),
            None,
            &mut NoopControl,
        )
        .unwrap();
        let inputs = captured.into_captured_inputs();
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].0, "source.dat");
        assert_eq!(inputs[0].1.identity, original.identity);
        assert_eq!(inputs[0].1.bytes, bytes);
        assert_eq!(inputs[0].1.sha256, original.sha256);
        assert_eq!(inputs[1].0, super::super::index::INDEX_PATH);
        assert_eq!(inputs[1].1.bytes, index.bytes().unwrap());
    }

    /// An absent distinct export output and index reserve two slots before source reads.
    #[test]
    fn source_export_plans_absent_output_before_capture_and_fits_98_sources() {
        let project = tempfile::tempdir().unwrap();
        let paths: Vec<_> = (0..98).map(|position| format!("source-{position}.bin")).collect();
        let current = registered(&paths);
        for path in &paths {
            std::fs::write(project.path().join(path), b"binary").unwrap();
        }
        std::fs::write(
            project.path().join(super::super::index::INDEX_PATH),
            current.bytes().unwrap(),
        )
        .unwrap();
        let root = Root::open(project.path()).unwrap();
        let snapshot = Snapshot::capture_source_bundle_effect_with_control(
            &root,
            None,
            Some("export.json"),
            &mut NoopControl,
        )
        .unwrap();
        assert_eq!(snapshot.items.len(), 98);
        assert!(!project.path().join("export.json").exists());
        assert_eq!(snapshot.into_captured_inputs().len(), 99);
    }

    /// A 101st planned output refuses before missing resources or a directory target is opened.
    #[test]
    fn source_export_output_slot_overflow_and_alias_refuse_before_resource_io() {
        let project = tempfile::tempdir().unwrap();
        let current = registered(
            &(0..99).map(|position| format!("missing-{position}.bin")).collect::<Vec<_>>(),
        );
        std::fs::write(
            project.path().join(super::super::index::INDEX_PATH),
            current.bytes().unwrap(),
        )
        .unwrap();
        std::fs::create_dir(project.path().join("export.json")).unwrap();
        let root = Root::open(project.path()).unwrap();
        for (output, code) in [
            ("export.json", "payload-too-large"),
            ("FORGE.WORKSPACE.JSON", "resource-containment"),
            ("MISSING-0.BIN", "resource-containment"),
        ] {
            let mut control = Recorder::default();
            assert!(
                matches!(Snapshot::capture_source_bundle_effect_with_control(&root,None,Some(output),&mut control),Err(WorkError::Failed(error)) if error.code==code)
            );
            assert!(!control.events.iter().any(|(stage, _)| *stage == Stage::CaptureResource));
        }
        assert!(project.path().join("export.json").is_dir());
    }

    /// The public producer seam accepts inert supplied bytes without requiring a Root.
    #[test]
    fn source_plan_validator_delegate_requires_only_exact_supplied_keys_and_bytes() {
        let index = registered(&["not-created.dat".to_owned()]);
        validate_proposed_sources(&index, &[("source-0", b"\0\xff\r\n")], &mut NoopControl)
            .unwrap();
        assert!(
            matches!(validate_proposed_sources(&index,&[("wrong-key",b"bytes")],&mut NoopControl),Err(WorkError::Failed(error)) if error.code=="validation-failed")
        );
    }
}
