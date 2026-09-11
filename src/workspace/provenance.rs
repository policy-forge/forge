//! Bounded provenance graph. Labels describe asserted relationships, never identity proof.
use super::contract::{self, Error, Result};
use super::index::Role;
use super::services::{Snapshot, opaque, resource_id};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const MAX_ENTRIES: usize = 10000;

/// Bounded display label. Identity-bearing values are hashed or referenced separately,
/// so truncation here never changes a graph edge.
fn bounded_label(value: &str) -> String {
    value.chars().take(500).collect()
}

pub(crate) struct Graph {
    entries: BTreeMap<String, Value>,
    excerpts: BTreeMap<String, Value>,
    retained_bytes: usize,
}
impl Graph {
    fn insert(&mut self, entry: Value) -> Result<()> {
        if self.entries.len() >= MAX_ENTRIES {
            return Err(Error::invalid());
        }
        contract::validate("ProvenanceEntry", &entry)?;
        self.retained_bytes += contract::encode(&entry, 1024 * 1024, false)?.len();
        if self.retained_bytes > 10 * 1024 * 1024 {
            return Err(Error::invalid());
        }
        self.entries
            .insert(entry["entry_id"].as_str().ok_or_else(Error::invalid)?.to_owned(), entry);
        Ok(())
    }
    #[allow(clippy::too_many_lines)] // Graph assembly preserves the visible trust checks beside each edge type.
    pub(crate) fn build(snapshot: &Snapshot) -> Result<Self> {
        let mut graph =
            Self { entries: BTreeMap::new(), excerpts: BTreeMap::new(), retained_bytes: 0 };
        for item in &snapshot.items {
            let resource = resource_id(&item.registration);
            let id = opaque("prov", &[&resource, &item.captured.sha256]);
            let label: String = bounded_label(&item.registration.path);
            let mut entry = json!({"entry_id":id,"kind":"source-reference","label":label,
                "fingerprint":item.captured.sha256,"refs":[{"kind":"resource","id":resource}]});
            if item.registration.role == Role::PolicySource && item.validation["state"] == "valid" {
                let text =
                    std::str::from_utf8(&item.captured.bytes).map_err(|_| Error::invalid())?;
                let excerpt_id = opaque("ex", &[&resource, &item.captured.sha256, "1"]);
                let mut excerpt = String::new();
                let mut lines = 0;
                for line in text.split_inclusive('\n').take(100) {
                    if excerpt.len().saturating_add(line.len()) > 20000 {
                        break;
                    }
                    excerpt.push_str(line);
                    lines += 1;
                }
                // No partial UTF-8 or partial source lines are represented as exact spans.
                if lines > 0 {
                    graph.retained_bytes += excerpt.len();
                    if graph.retained_bytes > 10 * 1024 * 1024 {
                        return Err(Error::invalid());
                    }
                    graph.excerpts.insert(excerpt_id.clone(),json!({"excerpt_id":excerpt_id,"resource_id":resource,
                        "sha256":item.captured.sha256,"start_line":1,"end_line":lines,"text":excerpt,"truncated":excerpt.len()<text.len()}));
                    entry["excerpt_refs"] = json!([excerpt_id]);
                }
            }
            graph.insert(entry)?;
        }
        graph.add_traces(snapshot)?;
        if let Some(analysis) = &snapshot.analysis {
            let manifest = super::domain::selected(snapshot, Role::ApplicabilityManifest)?;
            let manifest_id = resource_id(&manifest.registration);
            for control in &analysis.controls {
                let id = opaque("prov", &[&analysis.framework.raw_sha256, &control.control_id]);
                let mut refs = vec![json!({"kind":"resource","id":manifest_id})];
                let label: String = bounded_label(&format!(
                    "{}: {}",
                    control.control_id,
                    control.classification.as_str()
                ));
                if control.reviewer_key.is_some() {
                    let decision = opaque(
                        "prov",
                        &[&manifest.captured.sha256, &control.control_id, "decision"],
                    );
                    graph.insert(json!({"entry_id":decision,"kind":"applicability-decision","label":"Asserted human scope decision",
                        "fingerprint":manifest.captured.sha256,"refs":[{"kind":"control","id":id},{"kind":"resource","id":manifest_id}]}))?;
                    refs.push(json!({"kind":"applicability-decision","id":decision}));
                }
                graph.insert(json!({"entry_id":id,"kind":"control","label":label,"fingerprint":analysis.framework.raw_sha256,"refs":refs}))?;
            }
        }
        if super::domain::mapping_manifest(snapshot).is_ok() {
            // Do not traverse stale or invalid mapping declarations as verified edges.
            if super::domain::mapping(snapshot).is_ok() {
                let item = super::domain::mapping_manifest(snapshot)?;
                let manifest = crate::mapping::manifest::parse(&item.captured.bytes)
                    .map_err(|_| Error::invalid())?;
                for subject in super::domain::subjects(snapshot)? {
                    let subject_id = subject["subject_id"].as_str().ok_or_else(Error::invalid)?;
                    let kind =
                        if subject["side"] == "policy" { "policy-subject" } else { "control" };
                    let mut refs = vec![
                        json!({"kind":"resource","id":subject["resource_id"]}),
                        json!({"kind":kind,"id":subject_id}),
                    ];
                    if let Some(analysis) = &snapshot.analysis {
                        let source = snapshot
                            .item(subject["resource_id"].as_str().ok_or_else(Error::invalid)?)?;
                        if source.captured.sha256 == analysis.framework.raw_sha256
                            && analysis
                                .controls
                                .iter()
                                .any(|control| subject["label"] == control.control_id)
                        {
                            refs.push(json!({"kind":"control","id":opaque("prov", &[&analysis.framework.raw_sha256,subject["label"].as_str().ok_or_else(Error::invalid)?])}));
                        }
                    }
                    graph.insert(json!({"entry_id":opaque("prov", &[subject_id]),"kind":kind,"label":bounded_label(subject["label"].as_str().ok_or_else(Error::invalid)?),"fingerprint":subject["fingerprint"],"refs":refs}))?;
                }
                for mapping in &manifest.mapping.maps {
                    let mut refs =
                        vec![json!({"kind":"resource","id":resource_id(&item.registration)})];
                    for (side, resource, subjects) in [
                        ("policy", &manifest.mapping.source, &mapping.sources),
                        ("framework", &manifest.mapping.target, &mapping.targets),
                    ] {
                        let path = super::domain::resolve_reference(
                            &item.registration.path,
                            &resource.artifact,
                        )?;
                        let source = snapshot
                            .items
                            .iter()
                            .find(|item| item.registration.path == path)
                            .ok_or_else(Error::invalid)?;
                        for subject in subjects {
                            if refs.len() >= 50 {
                                return Err(Error::invalid());
                            }
                            let subject_id = opaque(
                                "subj",
                                &[
                                    side,
                                    &resource_id(&source.registration),
                                    subject.subject_type.as_str(),
                                    &subject.id_ref,
                                ],
                            );
                            refs.push(json!({"kind":if side=="policy" {"policy-subject"}else{"control"},"id":opaque("prov", &[&subject_id])}));
                        }
                    }
                    graph.insert(json!({"entry_id":opaque("prov", &[&item.captured.sha256,&mapping.key]),"kind":"mapping-edge","label":bounded_label(&format!("Asserted mapping: {}",mapping.key)),"fingerprint":item.captured.sha256,"refs":refs}))?;
                }
            }
        }
        for item in snapshot.queue() {
            let mut refs = item["evidence_refs"]
                .as_array()
                .map(|ids| {
                    ids.iter()
                        .map(|id| json!({"kind":"source-reference","id":id}))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            refs.push(json!({"kind":"report-item","id":item["item_id"]}));
            graph.insert(json!({"entry_id":opaque("prov", &[item["item_id"].as_str().ok_or_else(Error::invalid)?]),"kind":"report-item",
                "label":bounded_label(item["summary"].as_str().ok_or_else(Error::invalid)?),"refs":refs}))?;
        }
        Ok(graph)
    }
    fn add_traces(&mut self, snapshot: &Snapshot) -> Result<()> {
        for item in snapshot.items.iter().filter(|item| {
            matches!(
                item.registration.role,
                Role::OscalCatalogArtifact | Role::OscalComponentArtifact
            ) && item.validation["state"] == "valid"
        }) {
            let artifact = resource_id(&item.registration);
            for (ordinal, entry) in super::domain::trace_entries(item)?.into_iter().enumerate() {
                let Some(trace) = entry.trace else {
                    continue;
                };
                let id = opaque(
                    "prov",
                    &[
                        &artifact,
                        &item.captured.sha256,
                        entry.element_type.as_str(),
                        &entry.element_id,
                        &ordinal.to_string(),
                    ],
                );
                let mut refs = vec![json!({"kind":"resource","id":artifact})];
                // Mapping control subjects use the same explicitly selected resource identity.
                if entry.element_type == crate::trace::report::ElementType::Control {
                    for side in ["policy", "framework"] {
                        refs.push(json!({"kind":"policy-subject","id":opaque("prov", &[&opaque("subj", &[side,&artifact,"control",&entry.element_id])])}));
                    }
                }
                let mut excerpts = Vec::new();
                if let Some(source) = super::domain::trace_source(snapshot, &trace) {
                    let resource = resource_id(&source.registration);
                    refs.push(json!({"kind":"resource","id":resource}));
                    if let Some(start) = trace.source_line {
                        let text = std::str::from_utf8(&source.captured.bytes)
                            .map_err(|_| Error::invalid())?;
                        if crate::trace::resolver::validate_line_reference(
                            Some(start),
                            text.lines().count(),
                        ) {
                            let mut excerpt = String::new();
                            let mut count = 0;
                            for line in text.split_inclusive('\n').skip(start - 1).take(20) {
                                if excerpt.len() + line.len() > 20000 {
                                    break;
                                }
                                excerpt.push_str(line);
                                count += 1;
                            }
                            if count > 0 {
                                let excerpt_id = opaque(
                                    "ex",
                                    &[
                                        &resource,
                                        &source.captured.sha256,
                                        &start.to_string(),
                                        "trace",
                                    ],
                                );
                                if !self.excerpts.contains_key(&excerpt_id) {
                                    self.retained_bytes += excerpt.len();
                                    if self.retained_bytes > 10 * 1024 * 1024 {
                                        return Err(Error::invalid());
                                    }
                                    self.excerpts.insert(excerpt_id.clone(),json!({"excerpt_id":excerpt_id,"resource_id":resource,"sha256":source.captured.sha256,"start_line":start,"end_line":start+count-1,"text":excerpt,"truncated":start+count-1<text.lines().count()}));
                                }
                                excerpts.push(excerpt_id);
                            }
                        }
                    }
                }
                let label: String = format!("Asserted trace for {}. Source is current captured bytes; original source hash is not supplied.",entry.element_id).chars().take(500).collect();
                self.insert(json!({"entry_id":id,"kind":"source-reference","label":label,"fingerprint":item.captured.sha256,"refs":refs,"excerpt_refs":excerpts}))?;
            }
        }
        Ok(())
    }

    pub(crate) fn entries(&self, anchor: &str) -> Vec<Value> {
        // Direct neighbors in either direction. Further traversal is explicit;
        // avoid an unbounded transitive expansion through shared resources.
        let mut ids = BTreeSet::new();
        for (id, entry) in &self.entries {
            if id == anchor
                || entry["refs"]
                    .as_array()
                    .is_some_and(|refs| refs.iter().any(|r| r["id"] == anchor))
                || entry["excerpt_refs"]
                    .as_array()
                    .is_some_and(|refs| refs.iter().any(|r| r == anchor))
            {
                ids.insert(id.as_str());
                if let Some(refs) = entry["refs"].as_array() {
                    for r in refs {
                        if let Some(id) = r["id"].as_str() {
                            ids.insert(id);
                        }
                    }
                }
            }
        }
        ids.into_iter().filter_map(|id| self.entries.get(id).cloned()).collect()
    }
    pub(crate) fn excerpt(&self, id: &str) -> Result<Value> {
        self.excerpts.get(id).cloned().ok_or_else(|| {
            Error::new(
                "not-found",
                "The source excerpt is not available in this input version.",
                false,
            )
        })
    }
}
