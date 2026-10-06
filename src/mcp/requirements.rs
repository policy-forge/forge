//! Borrowed requirement extraction from one opaque, approved original capture.
//!
//! Recorded trace properties supply a relation, never filesystem authority. This
//! reader resolves them only through the disclosure gate and retains exact UTF-8
//! offsets in actual captured bytes. It does not use the trace walker's fallback
//! identifiers, write staged projects, reopen paths, or authenticate assessment.

use std::collections::BTreeMap;

use serde_json::Value;

use super::super::disclosure::CapturedQueryScope;
use super::super::requirement_walk::{array, native_string, unique_trace};
pub(super) use super::super::requirement_walk::{line_span, pointer_for_node};
use super::super::{NativeIdentity, Role};
use super::{QueryError, QueryResult, Reason, checkpoint, safe_token};
use crate::workspace::preparation::WorkControl;

/// Exact captured source size domain used by the approved first query profile.
const MAX_SOURCE_BYTES: usize = 10 * 1024 * 1024;

/// One borrowed native requirement and its exact captured source relation.
pub(super) struct Requirement<'a> {
    /// Opaque caller-approved artifact key, not a local filename.
    pub(super) artifact_key: &'a str,
    /// Actual Catalog control ID or implemented-requirement UUID, never invented.
    pub(super) id: &'a str,
    /// Actual control ID used by recorded control relations.
    pub(super) control_id: &'a str,
    /// Actual admitted requirement node, borrowed from the held native original.
    node: &'a Value,
    /// Complete actual native tree containing the node, not a reconstructed tree.
    tree: &'a Value,
    /// Actual native identity checked by the gate against selected expectations.
    pub(super) identity: &'a NativeIdentity,
    /// Raw native original digest, distinct from canonical subject fingerprints.
    pub(super) native_sha256: &'a str,
    /// Selected inert citation token, never a path or native title.
    pub(super) citation_label: &'a str,
    /// Exact closed native role, without widening workspace role admission.
    pub(super) role: Role,
    /// Actual declared source key selected by the captured gate association.
    pub(super) source_key: &'a str,
    /// Raw source original digest from the same held capture.
    pub(super) source_sha256: &'a str,
    /// Exact captured logical line; no title search or inferred policy prose.
    pub(super) source_line: &'a str,
    /// Inclusive UTF-8 byte start in the original source.
    pub(super) start_byte: usize,
    /// Exclusive UTF-8 byte end in the original source, excluding CRLF/LF.
    pub(super) end_byte: usize,
}

/// Complete sorted borrowed registry, including every requirement before paging.
pub(super) type RequirementSet<'a> = BTreeMap<(&'a str, &'a str), Requirement<'a>>;

/// Collect only the exact visible approved native policy roster.
///
/// Relationships are charged before registry insertion. Native/source currentness
/// is supplied only by the opaque real capture; parsing a trace is not that proof.
pub(super) fn collect<'a>(
    scope: &'a CapturedQueryScope,
    selected: Option<&str>,
    control: &mut dyn WorkControl,
) -> QueryResult<RequirementSet<'a>> {
    let mut result = BTreeMap::new();
    let mut selected_found = selected.is_none();
    for key in scope.visible_keys() {
        checkpoint(control)?;
        if selected.is_some_and(|wanted| wanted != key) {
            continue;
        }
        let resource = scope.resource(key).ok_or(Reason::IncompleteClosure)?;
        if !matches!(resource.role(), Role::OscalCatalogArtifact | Role::OscalComponentArtifact) {
            if selected.is_some() {
                return Err(Reason::UnsupportedRole.into());
            }
            continue;
        }
        selected_found = true;
        let approved = scope.approved(key, control)?.ok_or(Reason::Unapproved)?;
        let resource = approved.resource();
        let tree = resource.native_value().ok_or(Reason::InvalidArtifact)?;
        let identity = resource.native_identity().ok_or(Reason::InvalidArtifact)?;
        let native_sha = resource.raw_sha256();
        let label = resource.citation_label();
        let role = resource.role();
        match role {
            Role::OscalCatalogArtifact => {
                let root = tree.get("catalog").ok_or(Reason::InvalidArtifact)?;
                catalog_nodes(
                    root,
                    0,
                    &mut |node, control| {
                        let id = native_string(node, "id")?;
                        insert(
                            scope,
                            &mut result,
                            key,
                            id,
                            id,
                            tree,
                            node,
                            identity,
                            native_sha,
                            label,
                            role,
                            control,
                        )
                    },
                    scope,
                    control,
                )?;
            }
            Role::OscalComponentArtifact => {
                let root = tree.get("component-definition").ok_or(Reason::InvalidArtifact)?;
                for field in ["components", "capabilities"] {
                    for container in array(root, field)? {
                        checkpoint(control)?;
                        scope.charge_relationships(1)?;
                        for implementation in array(container, "control-implementations")? {
                            checkpoint(control)?;
                            scope.charge_relationships(1)?;
                            for node in array(implementation, "implemented-requirements")? {
                                let id = native_string(node, "uuid")?;
                                let control_id = native_string(node, "control-id")?;
                                insert(
                                    scope,
                                    &mut result,
                                    key,
                                    id,
                                    control_id,
                                    tree,
                                    node,
                                    identity,
                                    native_sha,
                                    label,
                                    role,
                                    control,
                                )?;
                            }
                        }
                    }
                }
            }
            _ => return Err(Reason::UnsupportedRole.into()),
        }
    }
    if !selected_found {
        return Err(Reason::NotFound.into());
    }
    checkpoint(control)?;
    Ok(result)
}

/// Admit one complete exact node/source relation before adding a borrowed row.
#[allow(clippy::too_many_arguments)]
fn insert<'a>(
    scope: &'a CapturedQueryScope,
    result: &mut RequirementSet<'a>,
    key: &'a str,
    id: &'a str,
    control_id: &'a str,
    tree: &'a Value,
    node: &'a Value,
    identity: &'a NativeIdentity,
    native_sha: &'a str,
    label: &'a str,
    role: Role,
    control: &mut dyn WorkControl,
) -> QueryResult<()> {
    checkpoint(control)?;
    scope.charge_relationships(1)?;
    if !safe_token(id) || !safe_token(control_id) || result.contains_key(&(key, id)) {
        return Err(Reason::InvalidArtifact.into());
    }
    // Reuse the maintained recorded-property extractor; it grants no capture.
    unique_trace(node)?;
    let trace = crate::trace::extractor::extract_trace_metadata(node)
        .ok_or(Reason::SourceSpanUnavailable)?;
    let line = trace.source_line.ok_or(Reason::SourceSpanUnavailable)?;
    let source = scope
        .declared_source_for_native(key, &trace.source_file, control)?
        .ok_or(Reason::IncompleteClosure)?;
    let raw = source.bytes();
    if raw.len() > MAX_SOURCE_BYTES {
        return Err(Reason::SourceSpanUnavailable.into());
    }
    let text = std::str::from_utf8(raw).map_err(|_| Reason::SourceSpanUnavailable)?;
    let (start_byte, end_byte) = line_span(text, line).ok_or(Reason::SourceSpanUnavailable)?;
    result.insert(
        (key, id),
        Requirement {
            artifact_key: key,
            id,
            control_id,
            node,
            tree,
            identity,
            native_sha256: native_sha,
            citation_label: label,
            role,
            source_key: source.key(),
            source_sha256: source.raw_sha256(),
            source_line: &text[start_byte..end_byte],
            start_byte,
            end_byte,
        },
    );
    checkpoint(control)
}

/// Delegate the complete Catalog walk while preserving the original /1 charge/checkpoint order.
fn catalog_nodes<'a>(
    node: &'a Value,
    depth: usize,
    visit: &mut impl FnMut(&'a Value, &mut dyn WorkControl) -> QueryResult<()>,
    scope: &CapturedQueryScope,
    control: &mut dyn WorkControl,
) -> QueryResult<()> {
    super::super::requirement_walk::catalog_nodes(
        node,
        depth,
        visit,
        &mut |amount| scope.charge_relationships(amount).map_err(QueryError::from),
        control,
    )
}

impl Requirement<'_> {
    /// Derive the complete actual pointer only for a retained result row.
    pub(super) fn pointer(&self, control: &mut dyn WorkControl) -> QueryResult<String> {
        pointer_for_node(self.tree, self.node, control)
    }
}

/// Explicit unavailable conversion; no private source error is echoed.
impl From<Reason> for QueryError {
    /// Preserve one fixed domain refusal without private source text or authority.
    fn from(reason: Reason) -> Self {
        Self::Unavailable(reason)
    }
}
