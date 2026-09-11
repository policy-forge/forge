//! Transport-neutral, root-confined workspace queries.

use std::collections::BTreeSet;

use serde_json::{Value, json};

use super::contract::{self, Error, Result};
use super::index::{Index, Resource, Role};
use super::root::{Captured, Root};

const MAX_CAPTURE_BYTES: usize = 50 * 1024 * 1024;
const MAX_RESOURCE_BYTES: usize = 10 * 1024 * 1024;

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

pub(crate) struct Snapshot {
    pub index: Index,
    pub index_present: bool,
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

pub(crate) fn validate_bytes(registration: &Resource, bytes: &[u8]) -> bool {
    if registration.role == Role::TraceReport {
        return super::reports::parse(bytes).is_ok_and(|report| report.kind == "trace");
    }
    if registration.role != Role::PolicySource
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
    }
}

fn validate_oscal(bytes: &[u8], kind: crate::validate::OscalModelType) -> bool {
    contract::parse(bytes, MAX_RESOURCE_BYTES, 64 * 1024).is_ok_and(|value| {
        crate::validate::run_full_validation("registered resource", &value, kind)
            .is_ok_and(|report| report.is_valid())
    })
}

impl Snapshot {
    pub(crate) fn capture(root: &Root) -> Result<Self> {
        let captured_index = root.read_index()?;
        let index_present = captured_index.is_some();
        let index = captured_index
            .as_ref()
            .map_or_else(|| Ok(Index::empty()), |captured| Index::parse(&captured.bytes))?;
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
            let remaining = MAX_CAPTURE_BYTES.checked_sub(spent).ok_or_else(Error::invalid)?;
            let captured = root.read(&registration.path, remaining.min(MAX_RESOURCE_BYTES))?;
            spent += captured.bytes.len();
            if !identities.insert(captured.identity) {
                return Err(Error::containment());
            }
            let id = resource_id(registration);
            let valid = validate_bytes(registration, &captured.bytes);
            let version = captured.sha256.clone();
            version_input.extend_from_slice(
                format!("{id}:{}:{}", captured.identity.0, captured.identity.1).as_bytes(),
            );
            version_input.extend_from_slice(version.as_bytes());
            let metadata = json!({"resource_id":id, "key":registration.key, "role":registration.role, "path":registration.path,
                "sha256":captured.sha256, "size_bytes":captured.bytes.len(), "validation_state":if valid {"valid"} else {"invalid"}, "stale":false,"version":version});
            contract::validate("Resource", &metadata)?;
            items.push(Item {
                registration: registration.clone(),
                captured,
                metadata,
                validation: validation(valid, Some(&id)),
            });
        }
        let mut snapshot = Self {
            index,
            index_present,
            version: crate::hashing::sha256_hex(&version_input),
            items,
            analysis: None,
            mapping_queue: Vec::new(),
        };
        if snapshot.items.iter().any(|item| item.registration.role == Role::ApplicabilityManifest) {
            match super::domain::analyze(&snapshot) {
                Ok(analysis) => {
                    if analysis.controls.len() > 10000
                        || analysis.review_queue.len() > 10000
                        || analysis.mapping_collections.len() > 100
                    {
                        return Err(Error::invalid());
                    }
                    snapshot.analysis = Some(analysis);
                }
                Err(_) => {
                    for item in &mut snapshot.items {
                        if item.registration.role == Role::ApplicabilityManifest {
                            item.validation =
                                validation(false, Some(&resource_id(&item.registration)));
                            item.metadata["validation_state"] = json!("invalid");
                        }
                    }
                }
            }
        }
        snapshot.mark_input_staleness();
        snapshot.populate_mapping_queue()?;
        for item in &mut snapshot.items {
            if item.registration.role == Role::ApplicabilityReport {
                let parsed = contract::parse(&item.captured.bytes, MAX_RESOURCE_BYTES, 64 * 1024);
                let matches = snapshot.analysis.as_ref().is_some_and(|analysis| {
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
            }
        }
        snapshot.validate_trace_reports()?;
        Ok(snapshot)
    }

    fn validate_trace_reports(&mut self) -> Result<()> {
        let trace_states: Vec<_> = self
            .items
            .iter()
            .filter(|item| item.registration.role == Role::TraceReport)
            .map(|item| {
                let valid = super::reports::parse(&item.captured.bytes);
                let matches =
                    valid.as_ref().is_ok_and(|report| report.matches(self).unwrap_or(false));
                (resource_id(&item.registration), valid.is_ok(), matches)
            })
            .collect();
        for (id, valid, matches) in trace_states {
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

    fn populate_mapping_queue(&mut self) -> Result<()> {
        let Ok(item) = super::domain::mapping_manifest(self) else {
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
        let Ok(built) = super::domain::mapping(self) else {
            let item = self
                .items
                .iter_mut()
                .find(|item| resource_id(&item.registration) == id)
                .ok_or_else(Error::invalid)?;
            item.validation = validation(false, Some(&id));
            item.metadata["validation_state"] = json!("invalid");
            return Ok(());
        };
        let inventory = super::domain::subject_inventory(self)?;
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
                    return Err(Error::invalid());
                }
                let bounded = bounded_label(subject, SUBJECT_LABEL_MAX).0;
                let reference = subjects
                    .iter()
                    .find(|row| {
                        row["side"] == side
                            && row["label"] == bounded
                            && row["statement_count"] == usize::from(kind == "statement")
                    })
                    .ok_or_else(Error::invalid)?;
                let label: String = subject.chars().take(350).collect();
                self.mapping_queue.push(json!({"item_id":opaque("qi", &[&hash,side,kind,subject]),
                    "reason_code":"no-reviewed-mapping","summary":format!("{side} {kind} {label} has no explicit reviewed mapping."),
                    "resource_id":reference["resource_id"],"evidence_refs":[reference["provenance_ref"],opaque("prov", &[&id,&hash])]}));
            }
        }
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
            version: "v".to_owned(),
            items: vec![fixture_item(Role::MappingCollection, "mapping.json", bytes)],
            analysis: None,
            mapping_queue: Vec::new(),
        };
        assert!(crate::workspace::domain::mapping_manifest(&snapshot).is_ok());
    }
}
