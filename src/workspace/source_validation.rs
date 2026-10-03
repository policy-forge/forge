//! Native admission of an inert proposed source closure, without physical identity claims.

use serde_json::Value;

use super::contract::{self, Error};
use super::index::{Index, Resource, Role};
use super::preparation::{ProgressUpdate, Stage, WorkControl, WorkResult};

/// Reject the complete proposed source closure without disclosing private parser text.
fn invalid() -> Error {
    Error::new("validation-failed", "The complete proposed source closure is invalid.", false)
}

/// Borrow the exact ordered proposed bytes; this object carries no file identity or authority.
struct Sources<'a> {
    /// Explicit, already strictly decoded registration manifest.
    index: &'a Index,
    /// Exact key/bytes bijection in the same authorial order as the manifest.
    bytes: &'a [(&'a str, &'a [u8])],
}

impl<'a> Sources<'a> {
    /// Select bytes by exact registration key; missing data never becomes an empty file.
    fn bytes(&self, resource: &Resource) -> Result<&'a [u8], Error> {
        self.bytes
            .iter()
            .find(|(key, _)| *key == resource.key)
            .map(|(_, bytes)| *bytes)
            .ok_or_else(invalid)
    }

    /// Resolve one exact role-admitted dependency inside the proposed registered closure.
    fn dependency(
        &self,
        manifest: &str,
        reference: &std::path::Path,
        roles: &[Role],
    ) -> Result<&Resource, Error> {
        let path = super::domain::resolve_reference(manifest, reference)?;
        self.index
            .resources
            .iter()
            .find(|resource| resource.path == path && roles.contains(&resource.role))
            .ok_or_else(invalid)
    }

    /// Check a native Mapping/framework declaration and its explicit resolved companion.
    fn resource(
        &self,
        manifest: &str,
        resource: &crate::mapping::manifest::ResourceManifest,
    ) -> Result<(), Error> {
        let role = match resource.resource_type {
            crate::mapping::manifest::ResourceType::Catalog => Role::OscalCatalogArtifact,
            crate::mapping::manifest::ResourceType::Profile => Role::OscalProfileArtifact,
        };
        self.dependency(manifest, &resource.artifact, &[role])?;
        if let Some(companion) = &resource.resolved_catalog {
            self.dependency(manifest, companion, &[Role::OscalCatalogArtifact])?;
        }
        Ok(())
    }

    /// Verify that a required native Mapping artifact is not merely a valid authoring manifest.
    fn native_mapping(&self, resource: &Resource) -> Result<(), Error> {
        let value = contract::parse(self.bytes(resource)?, 10 * 1024 * 1024, 64 * 1024)?;
        if crate::validate::run_full_validation(
            "proposed source closure",
            &value,
            crate::validate::OscalModelType::Mapping,
        )
        .is_ok_and(|report| report.is_valid())
        {
            Ok(())
        } else {
            Err(invalid())
        }
    }

    /// Admit all required local framework-impact roles before its native engine can read.
    fn impact(
        &self,
        resource: &Resource,
        manifest: &crate::framework::manifest::ImpactManifest,
    ) -> Result<(), Error> {
        for framework in [&manifest.old, &manifest.new] {
            let role = match framework.resource_type {
                crate::mapping::manifest::ResourceType::Catalog => Role::OscalCatalogArtifact,
                crate::mapping::manifest::ResourceType::Profile => Role::OscalProfileArtifact,
            };
            self.dependency(&resource.path, &framework.artifact, &[role])?;
            if let Some(companion) = &framework.resolved_catalog {
                self.dependency(&resource.path, companion, &[Role::OscalCatalogArtifact])?;
            }
        }
        for mapping in &manifest.mapping_collections {
            let dependency =
                self.dependency(&resource.path, &mapping.artifact, &[Role::MappingCollection])?;
            self.native_mapping(dependency)?;
        }
        if let Some(path) = &manifest.applicability_manifest {
            self.dependency(&resource.path, path, &[Role::ApplicabilityManifest])?;
        }
        if let Some(path) = &manifest.successor_map {
            self.dependency(&resource.path, path, &[Role::SuccessorMap])?;
        }
        if let (Some(prior), Some(dispositions)) =
            (&manifest.prior_report, &manifest.disposition_file)
        {
            self.dependency(&resource.path, prior, &[Role::FrameworkImpactReport])?;
            self.dependency(&resource.path, dispositions, &[Role::FrameworkImpactDispositions])?;
        }
        Ok(())
    }

    /// Follow declared lifecycle file edges while retaining historical hash/identity drift semantics.
    fn lifecycle(&self, resource: &Resource) -> Result<(), Error> {
        let record =
            crate::lifecycle::record::parse(self.bytes(resource)?).map_err(|_| invalid())?;
        let source = &record.policy.source.path;
        super::index::validate_path(source)?;
        self.dependency(
            &resource.path,
            std::path::Path::new(source),
            &[Role::LifecycleSource, Role::PolicySource],
        )?;
        for artifact in &record.policy.generated_artifacts {
            super::index::validate_path(&artifact.path)?;
            let dependency = self.dependency(
                &resource.path,
                std::path::Path::new(&artifact.path),
                &[
                    Role::OscalCatalogArtifact,
                    Role::OscalComponentArtifact,
                    Role::OscalProfileArtifact,
                    Role::OscalSspArtifact,
                    Role::MappingCollection,
                ],
            )?;
            let value = contract::parse(self.bytes(dependency)?, 10 * 1024 * 1024, 64 * 1024)?;
            let kind = crate::validate::detect_model_type(&value).map_err(|_| invalid())?;
            let root = match kind {
                crate::OscalModelType::Catalog => "catalog",
                crate::OscalModelType::ComponentDefinition => "component-definition",
                crate::OscalModelType::Profile => "profile",
                crate::OscalModelType::SystemSecurityPlan => "system-security-plan",
                crate::OscalModelType::Mapping => "mapping-collection",
            };
            let uuid = value[root]["uuid"].as_str().ok_or_else(invalid)?;
            uuid::Uuid::parse_str(uuid).map_err(|_| invalid())?;
            // Recorded hashes, OSCAL type and UUID remain declarations; mismatches are drift.
        }
        Ok(())
    }

    /// Reconcile every reported native read to one exact privately staged registered file.
    fn observed_inputs(
        &self,
        stage: &std::path::Path,
        paths: &[std::path::PathBuf],
    ) -> Result<(), Error> {
        let admitted = self
            .index
            .resources
            .iter()
            .map(|resource| stage.join(&resource.path).canonicalize().map_err(|_| invalid()))
            .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
        for path in paths {
            let observed = path.canonicalize().map_err(|_| invalid())?;
            if !admitted.contains(&observed) {
                return Err(invalid());
            }
        }
        Ok(())
    }

    /// Stage only proposed supplied bytes in a private owned directory for native engines.
    fn stage(&self, control: &mut dyn WorkControl) -> WorkResult<tempfile::TempDir> {
        control.checkpoint(Stage::CopyInputs, ProgressUpdate::Clear)?;
        let stage = tempfile::tempdir().map_err(|_| invalid())?;
        for resource in &self.index.resources {
            control.checkpoint(Stage::CopyInputs, ProgressUpdate::Unchanged)?;
            super::index::validate_path(&resource.path)?;
            let path = stage.path().join(&resource.path);
            let parent = path.parent().ok_or_else(invalid)?;
            std::fs::create_dir_all(parent).map_err(|_| invalid())?;
            std::fs::write(&path, self.bytes(resource)?).map_err(|_| invalid())?;
            control.checkpoint(Stage::CopyInputs, ProgressUpdate::Unchanged)?;
        }
        Ok(stage)
    }
}

/// Validate exact ordered supplied bytes and complete native manifest dependencies.
/// This does not inspect current targets, create Captured identities, or certify freshness.
pub(crate) fn validate_proposed_sources(
    index: &Index,
    sources: &[(&str, &[u8])],
    control: &mut dyn WorkControl,
) -> WorkResult<()> {
    control.checkpoint(Stage::ValidateResource, ProgressUpdate::Clear)?;
    let checked = Index::parse(&index.bytes()?)?;
    if checked.resources.len() > 99 || checked.resources.len() != sources.len() {
        return Err(invalid().into());
    }
    let mut spent = 0usize;
    for (resource, (key, bytes)) in checked.resources.iter().zip(sources) {
        control.checkpoint(Stage::ValidateResource, ProgressUpdate::Unchanged)?;
        if resource.key != *key || bytes.len() > 10 * 1024 * 1024 {
            return Err(invalid().into());
        }
        spent = spent.checked_add(bytes.len()).ok_or_else(invalid)?;
        if spent > 50 * 1024 * 1024 {
            return Err(invalid().into());
        }
        if resource.role != Role::ApplicabilityReport
            && !super::services::validate_bytes(resource, bytes)
        {
            return Err(invalid().into());
        }
        control.checkpoint(Stage::ValidateResource, ProgressUpdate::Unchanged)?;
    }
    let closure = Sources { index: &checked, bytes: sources };
    let manifests: Vec<_> = checked
        .resources
        .iter()
        .filter(|resource| resource.role == Role::ApplicabilityManifest)
        .collect();
    if manifests.len() > 1 {
        return Err(invalid().into());
    }
    let mut mapping = Vec::new();
    let mut impacts = Vec::new();
    for resource in &checked.resources {
        if resource.role == Role::MappingCollection {
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
            if let Ok(parsed) = crate::mapping::manifest::parse(closure.bytes(resource)?) {
                closure.resource(&resource.path, &parsed.mapping.source)?;
                closure.resource(&resource.path, &parsed.mapping.target)?;
                mapping.push(resource);
            }
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        }
        if resource.role == Role::FrameworkImpactManifest {
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
            let manifest = crate::framework::manifest::parse(closure.bytes(resource)?)
                .map_err(|_| invalid())?;
            closure.impact(resource, &manifest)?;
            impacts.push((resource, manifest));
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        }
        if resource.role == Role::LifecycleRecord {
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
            closure.lifecycle(resource)?;
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        }
    }
    if mapping.len() > 1 {
        return Err(invalid().into());
    }
    for resource in &manifests {
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        let manifest = crate::applicability::manifest::parse(closure.bytes(resource)?)
            .map_err(|_| invalid())?;
        closure.resource(&resource.path, &manifest.framework)?;
        for reference in &manifest.mapping_collections {
            let dependency =
                closure.dependency(&resource.path, reference, &[Role::MappingCollection])?;
            closure.native_mapping(dependency)?;
        }
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    }
    let expected_analysis =
        prepare_native_analysis(&closure, &manifests, mapping, impacts, control)?;
    for resource in &checked.resources {
        if resource.role == Role::ApplicabilityReport {
            control.checkpoint(Stage::SnapshotReports, ProgressUpdate::Unchanged)?;
            let supplied = contract::parse(closure.bytes(resource)?, 10 * 1024 * 1024, 64 * 1024);
            control.checkpoint(Stage::SnapshotReports, ProgressUpdate::Unchanged)?;
            let supplied = supplied?;
            if expected_analysis.as_ref() != Some(&supplied) {
                return Err(invalid().into());
            }
        }
    }
    control.checkpoint(Stage::ValidateResource, ProgressUpdate::Clear)?;
    Ok(())
}

/// Run the proposed native closure in its private stage with the original work control.
/// Preserve exact observed-input checks and bounded reports before comparing supplied output.
fn prepare_native_analysis(
    closure: &Sources<'_>,
    manifests: &[&super::index::Resource],
    mapping: Vec<&super::index::Resource>,
    impacts: Vec<(&super::index::Resource, crate::framework::manifest::ImpactManifest)>,
    control: &mut dyn WorkControl,
) -> WorkResult<Option<Value>> {
    let mut expected_analysis: Option<Value> = None;
    if !manifests.is_empty() || !mapping.is_empty() || !impacts.is_empty() {
        let stage = closure.stage(control)?;
        for resource in mapping {
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
            let built = crate::mapping::prepare(&stage.path().join(&resource.path), None, false)
                .map_err(|_| invalid());
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
            built?;
        }
        if let Some(resource) = manifests.first() {
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
            let analyzed = crate::applicability::prepare_analysis(
                &stage.path().join(&resource.path),
                crate::applicability::model::ReportFilters::default(),
            )
            .map_err(|_| invalid());
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
            let analyzed = analyzed?;
            closure.observed_inputs(stage.path(), &analyzed.input_paths)?;
            let report = analyzed.report;
            if report.controls.len() > 10_000
                || report.review_queue.len() > 10_000
                || report.mapping_collections.len() > 100
            {
                return Err(invalid().into());
            }
            expected_analysis = Some(serde_json::to_value(report).map_err(|_| invalid())?);
        }
        for (resource, manifest) in impacts {
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
            let path = stage.path().join(&resource.path);
            let base = path.parent().ok_or_else(invalid)?;
            let analyzed = crate::framework::analysis::analyze(
                base,
                &manifest,
                crate::framework::model::ImpactFilters::default(),
            )
            .map_err(|_| invalid());
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
            let (report, paths) = analyzed?;
            if report.changes.len() > 100_000
                || report.prior_only_dispositions.len() > 100_000
                || report
                    .findings
                    .len()
                    .checked_add(report.filtered_out_findings.len())
                    .is_none_or(|count| count > 100_000)
            {
                return Err(invalid().into());
            }
            closure.observed_inputs(stage.path(), &paths)?;
        }
    }
    Ok(expected_analysis)
}

#[cfg(test)]
mod tests {
    use super::super::preparation::{Interruption, NoopControl, WorkError, test_support::Recorder};
    use super::*;
    use serde_json::json;

    /// Build actual strict index2 registrations whose targets need not exist anywhere.
    fn index(rows: &[(&str, &str, &str)]) -> Index {
        Index::parse(&serde_json::to_vec(&json!({
            "schema_version":"forge.workspace/2","label":"Synthetic proposed closure",
            "resources":rows.iter().map(|(key,role,path)|json!({"key":key,"role":role,"path":path})).collect::<Vec<_>>()
        })).unwrap()).unwrap()
    }

    /// Native schema-valid catalog and applicability bytes, with local registered references.
    fn analysis_bytes() -> (Vec<u8>, Vec<u8>) {
        let catalog=serde_json::to_vec(&json!({"catalog":{
            "uuid":"11111111-1111-4111-8111-111111111111",
            "metadata":{"title":"Synthetic proposed catalog","last-modified":"2026-10-03T00:00:00Z","version":"1","oscal-version":"1.2.3"},
            "controls":[{"id":"synthetic-control","title":"Synthetic control"}]
        }})).unwrap();
        let stage = tempfile::tempdir().unwrap();
        std::fs::write(stage.path().join("catalog.json"), &catalog).unwrap();
        let mut framework: crate::mapping::manifest::ResourceManifest =
            serde_json::from_value(json!({
                "type":"catalog","artifact":"catalog.json","href":"catalog.json",
                "expected_sha256":crate::hashing::sha256_hex(&catalog)
            }))
            .unwrap();
        framework.inventory = Some(
            crate::mapping::inventory::load(stage.path(), "synthetic proposed catalog", &framework)
                .unwrap()
                .snapshot(),
        );
        let manifest = serde_json::to_vec(&json!({
            "schema_version":"forge.applicability/1",
            "framework":framework,
            "reviewers":[],"decisions":[],"mapping_collections":[]
        }))
        .unwrap();
        (catalog, manifest)
    }

    /// Build a native impact declaration pinned to two exact proposed Catalog revisions.
    fn impact_bytes(old: &[u8], new: &[u8]) -> Vec<u8> {
        let declaration = |path: &str, bytes: &[u8]| {
            let value: Value = serde_json::from_slice(bytes).unwrap();
            json!({"type":"catalog","artifact":path,
                "expected_sha256":crate::hashing::sha256_hex(bytes),
                "root_uuid":value["catalog"]["uuid"],
                "document_version":value["catalog"]["metadata"]["version"],
                "oscal_version":"1.2.3"})
        };
        let bytes = serde_json::to_vec(&json!({
            "schema_version":"forge.framework-impact/1",
            "old":declaration("old.json",old),"new":declaration("new.json",new),
            "mapping_collections":[]
        }))
        .unwrap();
        crate::framework::manifest::parse(&bytes).unwrap();
        bytes
    }

    /// Produce an intrinsically valid draft whose recorded source hash may describe history.
    fn lifecycle_bytes(generated: &Value) -> Vec<u8> {
        let bytes = serde_json::to_vec(&json!({
            "schema_version":"forge.policy-lifecycle/2",
            "policy":{"policy_key":"synthetic-policy","version_key":"v1","title":"Synthetic",
                "owner_keys":["owner"],"source":{"path":"source.bin","sha256":"a".repeat(64)},
                "generated_artifacts":generated},
            "parties":[{"key":"owner","roles":["owner","reviewer","approver","author"]}],
            "approval_policy":{"schema_version":"forge.approval-policy/1",
                "required_roles":[{"role":"reviewer","count":1},{"role":"approver","count":1}],"separation":{}},
            "review":{"cadence_days":30,"next_review_date":"2026-10-12","due_soon_days":7,"timezone_policy":"date-only"},
            "state":"draft","history":[]
        })).unwrap();
        crate::lifecycle::record::parse(&bytes).unwrap();
        bytes
    }

    /// Distinguish complete-closure refusal from interruption and unrelated error classes.
    fn refused(result: WorkResult<()>) {
        assert!(matches!(result,Err(WorkError::Failed(error)) if error.code=="validation-failed"));
    }

    /// Catalog-shaped bytes registered as a binary source cannot satisfy a framework role.
    #[test]
    fn proposed_framework_dependencies_require_native_registration_role() {
        let (catalog, manifest) = analysis_bytes();
        let proposed = index(&[
            ("catalog", "lifecycle-source", "catalog.json"),
            ("scope", "applicability-manifest", "scope.json"),
        ]);
        refused(validate_proposed_sources(
            &proposed,
            &[("catalog", &catalog), ("scope", &manifest)],
            &mut NoopControl,
        ));
    }

    /// A valid framework-impact manifest needs its whole registered old/new dependency pair.
    #[test]
    fn proposed_impact_manifest_cannot_omit_or_misrole_dependencies() {
        let (catalog, _) = analysis_bytes();
        let manifest = impact_bytes(&catalog, &catalog);
        let isolated = index(&[("impact", "framework-impact-manifest", "impact.json")]);
        refused(validate_proposed_sources(&isolated, &[("impact", &manifest)], &mut NoopControl));
        let misrole = index(&[
            ("old", "oscal-catalog-artifact", "old.json"),
            ("new", "lifecycle-source", "new.json"),
            ("impact", "framework-impact-manifest", "impact.json"),
        ]);
        refused(validate_proposed_sources(
            &misrole,
            &[("old", &catalog), ("new", &catalog), ("impact", &manifest)],
            &mut NoopControl,
        ));
    }

    /// The native impact engine consumes a complete pair and enforces exact authorial pins.
    #[test]
    fn proposed_impact_runs_native_engine_and_rejects_wrong_expected_hash() {
        let (old, _) = analysis_bytes();
        let mut revised: Value = serde_json::from_slice(&old).unwrap();
        revised["catalog"]["metadata"]["version"] = json!("2");
        let new = serde_json::to_vec(&revised).unwrap();
        let manifest = impact_bytes(&old, &new);
        let proposed = index(&[
            ("old", "oscal-catalog-artifact", "old.json"),
            ("new", "oscal-catalog-artifact", "new.json"),
            ("impact", "framework-impact-manifest", "impact.json"),
        ]);
        validate_proposed_sources(
            &proposed,
            &[("old", &old), ("new", &new), ("impact", &manifest)],
            &mut NoopControl,
        )
        .unwrap();
        let mut stale: Value = serde_json::from_slice(&manifest).unwrap();
        stale["new"]["expected_sha256"] = json!("b".repeat(64));
        let stale = serde_json::to_vec(&stale).unwrap();
        crate::framework::manifest::parse(&stale).unwrap();
        refused(validate_proposed_sources(
            &proposed,
            &[("old", &old), ("new", &new), ("impact", &stale)],
            &mut NoopControl,
        ));
    }

    /// Lifecycle records require registered source bytes while preserving recorded hash drift.
    #[test]
    fn proposed_lifecycle_requires_source_without_rewriting_historical_hash() {
        let record = lifecycle_bytes(&json!([]));
        let isolated = index(&[("record", "lifecycle-record", "record.json")]);
        refused(validate_proposed_sources(&isolated, &[("record", &record)], &mut NoopControl));
        let proposed = index(&[
            ("source", "lifecycle-source", "source.bin"),
            ("record", "lifecycle-record", "record.json"),
        ]);
        let source = b"current bytes that intentionally differ from the historical declaration";
        validate_proposed_sources(
            &proposed,
            &[("source", source), ("record", &record)],
            &mut NoopControl,
        )
        .unwrap();
        assert_eq!(
            crate::lifecycle::record::parse(&record).unwrap().policy.source.sha256,
            "a".repeat(64)
        );
    }

    /// Missing and misregistered generated artifacts fail; historical identity drift remains valid.
    #[test]
    fn proposed_lifecycle_admits_generated_artifact_roles_and_retains_drift() {
        let (catalog, _) = analysis_bytes();
        let record = lifecycle_bytes(&json!([{"path":"catalog.json","sha256":"b".repeat(64),
            "oscal_type":"profile","root_uuid":"22222222-2222-4222-8222-222222222222"}]));
        let source = b"current source";
        let missing = index(&[
            ("source", "lifecycle-source", "source.bin"),
            ("record", "lifecycle-record", "record.json"),
        ]);
        refused(validate_proposed_sources(
            &missing,
            &[("source", source), ("record", &record)],
            &mut NoopControl,
        ));
        for role in ["lifecycle-source", "oscal-catalog-artifact"] {
            let proposed = index(&[
                ("source", "lifecycle-source", "source.bin"),
                ("artifact", role, "catalog.json"),
                ("record", "lifecycle-record", "record.json"),
            ]);
            let admitted = validate_proposed_sources(
                &proposed,
                &[("source", source), ("artifact", &catalog), ("record", &record)],
                &mut NoopControl,
            );
            if role == "lifecycle-source" {
                refused(admitted);
            } else {
                admitted.unwrap();
            }
        }
    }

    /// Native reported reads outside the staged registration set cannot pass reconciliation.
    #[test]
    fn observed_native_reads_must_belong_to_registered_stage() {
        let proposed = index(&[("source", "lifecycle-source", "source.bin")]);
        let pairs = [("source", b"binary".as_slice())];
        let closure = Sources { index: &proposed, bytes: &pairs };
        let stage = closure.stage(&mut NoopControl).unwrap();
        closure.observed_inputs(stage.path(), &[stage.path().join("source.bin")]).unwrap();
        let foreign = tempfile::NamedTempFile::new().unwrap();
        assert_eq!(
            closure.observed_inputs(stage.path(), &[foreign.path().to_owned()]).unwrap_err().code,
            "validation-failed"
        );
    }

    /// An empty explicit closure is admissible without a Root or invented file identity.
    #[test]
    fn empty_proposed_closure_needs_no_current_targets() {
        validate_proposed_sources(&index(&[]), &[], &mut NoopControl).unwrap();
    }

    /// Declared lifecycle source bytes preserve binary, BOM and CRLF without normalization.
    #[test]
    fn binary_proposed_sources_need_no_physical_capture() {
        let proposed = index(&[("binary", "lifecycle-source", "not-created/binary.dat")]);
        let bytes = b"\xef\xbb\xbf\0\xff\r\n";
        validate_proposed_sources(&proposed, &[("binary", bytes)], &mut NoopControl).unwrap();
        assert_eq!(bytes, b"\xef\xbb\xbf\0\xff\r\n");
    }

    /// Missing, duplicate or reordered keys refuse the whole proposal before staging.
    #[test]
    fn complete_ordered_bijection_is_required() {
        let proposed = index(&[
            ("first", "lifecycle-source", "first.dat"),
            ("second", "lifecycle-source", "second.dat"),
        ]);
        for sources in [
            vec![("first", b"one".as_slice())],
            vec![("first", b"one".as_slice()), ("first", b"two".as_slice())],
            vec![("second", b"two".as_slice()), ("first", b"one".as_slice())],
        ] {
            assert!(
                matches!(validate_proposed_sources(&proposed,&sources,&mut NoopControl),Err(WorkError::Failed(error)) if error.code=="validation-failed")
            );
        }
    }

    /// A declared Markdown source must satisfy its real native parser, not just hex grammar.
    #[test]
    fn intrinsically_invalid_source_refuses_complete_closure() {
        let proposed = index(&[("policy", "policy-source", "policy.md")]);
        assert!(
            matches!(validate_proposed_sources(&proposed,&[("policy",b"\xff")],&mut NoopControl),Err(WorkError::Failed(error)) if error.code=="validation-failed")
        );
    }

    /// A syntactically valid applicability manifest cannot consume an unregistered file.
    #[test]
    fn unregistered_proposed_dependencies_are_rejected() {
        let (_, manifest) = analysis_bytes();
        let proposed = index(&[("scope", "applicability-manifest", "scope.json")]);
        assert!(
            matches!(validate_proposed_sources(&proposed,&[("scope",&manifest)],&mut NoopControl),Err(WorkError::Failed(error)) if error.code=="validation-failed")
        );
    }

    /// The shared applicability engine admits a complete proposed catalog and manifest.
    #[test]
    fn native_analysis_consumes_only_complete_supplied_closure() {
        let (catalog, manifest) = analysis_bytes();
        let proposed = index(&[
            ("catalog", "oscal-catalog-artifact", "catalog.json"),
            ("scope", "applicability-manifest", "scope.json"),
        ]);
        validate_proposed_sources(
            &proposed,
            &[("catalog", &catalog), ("scope", &manifest)],
            &mut NoopControl,
        )
        .unwrap();
    }

    /// Applicability report structure alone supplies no exact proposed-analysis admission.
    #[test]
    fn report_requires_complete_matching_proposed_analysis() {
        let (catalog, manifest) = analysis_bytes();
        let base = index(&[
            ("catalog", "oscal-catalog-artifact", "catalog.json"),
            ("scope", "applicability-manifest", "scope.json"),
        ]);
        let pairs = [("catalog", catalog.as_slice()), ("scope", manifest.as_slice())];
        let closure = Sources { index: &base, bytes: &pairs };
        let stage = closure.stage(&mut NoopControl).unwrap();
        let report = crate::applicability::prepare_analysis(
            &stage.path().join("scope.json"),
            crate::applicability::model::ReportFilters::default(),
        )
        .unwrap()
        .report;
        let bytes = serde_json::to_vec(&report).unwrap();
        let proposed = index(&[
            ("catalog", "oscal-catalog-artifact", "catalog.json"),
            ("scope", "applicability-manifest", "scope.json"),
            ("report", "applicability-report", "report.json"),
        ]);
        validate_proposed_sources(
            &proposed,
            &[("catalog", &catalog), ("scope", &manifest), ("report", &bytes)],
            &mut NoopControl,
        )
        .unwrap();
        let isolated = index(&[("report", "applicability-report", "report.json")]);
        assert!(
            matches!(validate_proposed_sources(&isolated,&[("report",&bytes)],&mut NoopControl),Err(WorkError::Failed(error)) if error.code=="validation-failed")
        );
    }

    /// Cooperative cancellation before a private copy stays typed rather than invalid/stale.
    #[test]
    fn interruption_is_not_swallowed_by_native_admission() {
        let (catalog, manifest) = analysis_bytes();
        let proposed = index(&[
            ("catalog", "oscal-catalog-artifact", "catalog.json"),
            ("scope", "applicability-manifest", "scope.json"),
        ]);
        let mut stop = Recorder::at(Stage::CopyInputs, 1);
        assert!(matches!(
            validate_proposed_sources(
                &proposed,
                &[("catalog", &catalog), ("scope", &manifest)],
                &mut stop
            ),
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        ));
    }
}
