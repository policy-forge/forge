//! Prospective real closed-decoder and plain graph controls; no native authority fixtures.

use super::*;
use crate::review::capture::ReviewControl;
use crate::review::decode::decode_queue;
use crate::review::{encode, supersession_wire, wire::*};
use crate::workspace::preparation::{
    Interruption, NoopControl, ProgressUpdate, Stage, WorkError, WorkResult,
};
use serde_json::json;
use std::cell::Cell;

/// Complete inert declaration fixture, bound by genuine maintained typed hashes.
fn queue(id: &str, domains: &[Domain]) -> QueueDocument {
    let mut queue = QueueDocument {
        schema_version: "forge.review-queue/1".into(),
        identity_disclaimer: IDENTITY_DISCLAIMER.into(),
        sensitivity: Sensitivity::IdsAndHashes,
        queue_id: id.into(),
        created_at: "2026-10-04T00:00:00Z".into(),
        source_pins: vec![SourcePin {
            artifact_key: "declared".into(),
            model: SourceModel::Catalog,
            native_root_uuid: Some("11111111-1111-4111-8111-111111111111".into()),
            raw_sha256: "0".repeat(64),
            byte_length: 100,
            schema_identity: "oscal.catalog/1".into(),
        }],
        roles: vec![RoleDefinition { key: "review".into() }],
        reviewers: vec![Reviewer { key: "reviewer".into(), role_keys: vec!["review".into()] }],
        policies: vec![ReviewPolicy {
            key: "policy".into(),
            seats: vec![SeatRequirement { role_key: "review".into(), count: 1 }],
            substitutions: vec![],
            abstention_rule: AbstentionRule::Nonapproving,
            empty_abstention_reasons: vec![],
            author_separation: AuthorSeparation::DeclaredKeys,
        }],
        items: domains
            .iter()
            .enumerate()
            .map(|(index, domain)| ReviewItem {
                key: format!("item-{index:04}"),
                item_id: String::new(),
                domain: *domain,
                adapter_version: match domain {
                    Domain::MappingAssertion => "forge.mapping-review/1",
                    Domain::ApplicabilityDecision => "forge.applicability-review/1",
                }
                .into(),
                subject_id: format!("subject-{index}"),
                requested_action: RequestedAction::ReReview,
                source_keys: vec!["declared".into()],
                subject_sha256: "1".repeat(64),
                context: ContextSnapshot {
                    reason_codes: vec!["rereview".into()],
                    related_subject_ids: vec!["PRIVATE-RELATED-ID".into()],
                },
                context_sha256: String::new(),
                policy_key: "policy".into(),
                policy_sha256: String::new(),
                author_keys: vec!["author".into()],
                assignments: vec![],
                due_at: None,
                allowed_dispositions: vec![
                    Disposition::Approve,
                    Disposition::Reject,
                    Disposition::RequestChanges,
                    Disposition::Abstain,
                    Disposition::Superseded,
                ],
            })
            .collect(),
    };
    rebind(&mut queue);
    queue
}

/// Compute real existing encoding profiles; recorded native pins stay explicitly unverified.
fn rebind(queue: &mut QueueDocument) {
    let mut items = std::mem::take(&mut queue.items);
    let mut ledger = ContractLedger::default();
    for item in &mut items {
        item.context_sha256 = validate::context_hash(queue, item, &mut ledger).unwrap();
        item.policy_sha256 =
            validate::policy_hash(queue, &queue.policies[0], item, &mut ledger).unwrap();
        item.item_id = validate::item_id(queue, item, &mut ledger).unwrap();
    }
    queue.items = items;
}

/// Produce a structurally admitted original through the real unchanged finite queue encoder.
fn raw(queue: &QueueDocument) -> Vec<u8> {
    encode::queue(queue, &mut ContractLedger::default(), &mut NoopControl).unwrap()
}

/// Encode explicit pair selections only; sort is fixture authoring, not production graph work.
fn request(old: &QueueDocument, new: &QueueDocument, pairs: &[(usize, usize, &[&str])]) -> Vec<u8> {
    let mut links: Vec<_> = pairs
        .iter()
        .map(|(a, b, findings)| LinkRequest {
            old_item_id: old.items[*a].item_id.clone(),
            new_item_id: new.items[*b].item_id.clone(),
            finding_ids: findings.iter().map(|id| (*id).to_owned()).collect(),
        })
        .collect();
    links.sort_by(|a, b| (&a.old_item_id, &a.new_item_id).cmp(&(&b.old_item_id, &b.new_item_id)));
    serde_json::to_vec(&LinksRequest { schema_version: LINKS_SCHEMA.into(), links }).unwrap()
}

/// One canonical first queue declaration, never a native capture factory.
const OLD: &str = "22222222-2222-4222-8222-222222222222";
/// One distinct canonical successor declaration.
const NEW: &str = "33333333-3333-4333-8333-333333333333";
/// Explicit unbound requested finding selection.
const FINDING_A: &str = "44444444-4444-4444-8444-444444444444";
/// Another explicit unbound requested finding selection.
const FINDING_B: &str = "55555555-5555-4555-8555-555555555555";

/// Full empty selection exposes every actual item in each original queue order.
#[test]
fn empty_links_keep_both_complete_queue_rosters_and_raw_pins() {
    let old = queue(OLD, &[Domain::MappingAssertion, Domain::ApplicabilityDecision]);
    let new = queue(
        NEW,
        &[Domain::MappingAssertion, Domain::ApplicabilityDecision, Domain::ApplicabilityDecision],
    );
    let old_raw = raw(&old);
    let new_raw = raw(&new);
    let link_raw = request(&old, &new, &[]);
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let a = decode_queue(&old_raw, &mut ledger, &mut control).unwrap();
    let b = decode_queue(&new_raw, &mut ledger, &mut control).unwrap();
    let r = decode_links(&link_raw, &mut ledger, &mut control).unwrap();
    let graph = prepare_graph(&a, &b, &r, &mut ledger, &mut control).unwrap();
    assert_eq!(
        graph.unmatched_old().iter().map(|item| item.key.as_str()).collect::<Vec<_>>(),
        vec!["item-0000", "item-0001"]
    );
    assert_eq!(graph.unmatched_new().len(), 3);
    assert!(graph.links().is_empty() && graph.requested_findings().is_empty());
    let counts = graph.counts();
    assert_eq!(
        (
            counts.old_items,
            counts.new_items,
            counts.link_edges,
            counts.linked_old_items,
            counts.linked_new_items
        ),
        (2, 3, 0, 0, 0)
    );
    let reference = supersession_wire::queue_reference(&a, &mut ledger, &mut control).unwrap();
    assert_eq!(reference.raw_sha256, crate::hashing::sha256_hex(&old_raw));
    assert_eq!(usize::try_from(reference.byte_length).expect("actual queue extent"), old_raw.len());
    assert!(reference.source_pins == a.document().source_pins.as_slice());
    assert_eq!(r.raw(), link_raw);
    assert_eq!(r.raw_sha256(), crate::hashing::sha256_hex(&link_raw));
}

/// Split/merge preserves unique edges, separate distinct memberships and repeated finding use.
#[test]
fn complete_many_to_many_memberships_conserve_without_cartesian_inference() {
    let old = queue(OLD, &[Domain::MappingAssertion; 3]);
    let new = queue(NEW, &[Domain::MappingAssertion; 3]);
    let old_raw = raw(&old);
    let new_raw = raw(&new);
    let link_raw =
        request(&old, &new, &[(0, 0, &[FINDING_A]), (0, 1, &[FINDING_A]), (1, 0, &[FINDING_B])]);
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let a = decode_queue(&old_raw, &mut ledger, &mut control).unwrap();
    let b = decode_queue(&new_raw, &mut ledger, &mut control).unwrap();
    let r = decode_links(&link_raw, &mut ledger, &mut control).unwrap();
    let graph = prepare_graph(&a, &b, &r, &mut ledger, &mut control).unwrap();
    let counts = graph.counts();
    assert_eq!((counts.link_edges, counts.linked_old_items, counts.linked_new_items), (3, 2, 2));
    assert_eq!(
        (
            counts.unmatched_old_items,
            counts.unmatched_new_items,
            counts.link_finding_occurrences,
            counts.distinct_requested_findings
        ),
        (1, 1, 3, 2)
    );
    assert_eq!(graph.unmatched_old()[0].key, "item-0002");
    assert_eq!(graph.unmatched_new()[0].key, "item-0002");
    assert_eq!(graph.requested_findings(), &[FINDING_A, FINDING_B]);
    for (selected, link) in r.document().links.iter().zip(graph.links()) {
        assert_eq!(selected.old_item_id, link.old_item().item_id);
        assert_eq!(selected.new_item_id, link.new_item().item_id);
        assert_eq!(selected.finding_ids, link.requested_findings());
    }
}

/// Missing endpoints, cross-domain links and repeated queue UUIDs refuse exact binding.
#[test]
fn exact_endpoint_domain_and_distinct_queue_identity_are_required() {
    let old = queue(OLD, &[Domain::MappingAssertion]);
    for mode in 0..3 {
        let new = queue(
            if mode == 2 { OLD } else { NEW },
            &[if mode == 1 { Domain::ApplicabilityDecision } else { Domain::MappingAssertion }],
        );
        let old_raw = raw(&old);
        let new_raw = raw(&new);
        let mut value: Value =
            serde_json::from_slice(&request(&old, &new, &[(0, 0, &[])])).unwrap();
        if mode == 0 {
            value["links"][0]["old_item_id"] = json!("ffffffff-ffff-4fff-8fff-ffffffffffff");
        }
        let link_raw = serde_json::to_vec(&value).unwrap();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let a = decode_queue(&old_raw, &mut ledger, &mut control).unwrap();
        let b = decode_queue(&new_raw, &mut ledger, &mut control).unwrap();
        let r = decode_links(&link_raw, &mut ledger, &mut control).unwrap();
        assert!(matches!(
            prepare_graph(&a, &b, &r, &mut ledger, &mut control),
            Err(ContractError::Binding)
        ));
    }
}

/// Full typed comparisons retain context/policy fields and pin values even when keys are equal.
#[test]
fn complete_typed_differences_follow_maintained_profiles() {
    let old = queue(OLD, &[Domain::MappingAssertion]);
    for mode in 0..6 {
        let mut new = queue(NEW, &[Domain::MappingAssertion]);
        match mode {
            1 => new.items[0].subject_sha256 = "2".repeat(64),
            2 => new.items[0].context.related_subject_ids = vec!["OTHER-PRIVATE-ID".into()],
            3 => new.items[0].author_keys = vec!["different-author".into()],
            4 => new.source_pins[0].raw_sha256 = "f".repeat(64),
            5 => new.policies[0].seats[0].count = 2,
            _ => {}
        }
        rebind(&mut new);
        let old_raw = raw(&old);
        let new_raw = raw(&new);
        let link_raw = request(&old, &new, &[(0, 0, &[])]);
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let a = decode_queue(&old_raw, &mut ledger, &mut control).unwrap();
        let b = decode_queue(&new_raw, &mut ledger, &mut control).unwrap();
        let r = decode_links(&link_raw, &mut ledger, &mut control).unwrap();
        let graph = prepare_graph(&a, &b, &r, &mut ledger, &mut control).unwrap();
        let d = graph.links()[0].differences();
        assert_eq!(
            (d.subject, d.context, d.policy, d.source_pins),
            match mode {
                1 => (true, false, false, false),
                2 => (false, true, false, false),
                3 | 5 => (false, false, true, false),
                4 => (false, true, true, true),
                _ => (false, false, false, false),
            }
        );
    }
}

/// No private request hash/proof fields, unknown nested keys or omitted arrays are accepted.
#[test]
fn closed_request_requires_every_exact_field_and_canonical_sorted_ids() {
    let old = queue(OLD, &[Domain::MappingAssertion; 2]);
    let new = queue(NEW, &[Domain::MappingAssertion; 2]);
    let original: Value = serde_json::from_slice(&request(
        &old,
        &new,
        &[(0, 0, &[FINDING_A, FINDING_B]), (1, 1, &[])],
    ))
    .unwrap();
    for mode in 0..8 {
        let mut value = original.clone();
        match mode {
            0 => {
                value["native_approval"] = json!(true);
            }
            1 => {
                value["links"][0]["queue_raw_sha256"] = json!("0".repeat(64));
            }
            2 => {
                value["links"][0].as_object_mut().unwrap().remove("finding_ids");
            }
            3 => {
                value["links"][0]["finding_ids"] = Value::Null;
            }
            4 => {
                value["links"][0]["finding_ids"] = json!([FINDING_A, FINDING_A]);
            }
            5 => {
                value["links"][0]["finding_ids"] = json!([FINDING_B, FINDING_A]);
            }
            6 => {
                value["links"].as_array_mut().unwrap().reverse();
            }
            _ => {
                value["links"][0]["old_item_id"] = json!("00000000-0000-0000-0000-000000000000");
            }
        }
        let raw = serde_json::to_vec(&value).unwrap();
        assert!(matches!(
            decode_links(&raw, &mut ContractLedger::default(), &mut NoopControl),
            Err(ContractError::Invalid)
        ));
    }
}

/// Actual strict parser refuses root/nested duplicates, BOM, trailing data and malformed raw.
#[test]
fn duplicate_safe_raw_parser_refuses_ambiguous_originals() {
    let mut cases = vec![
        b"{".to_vec(),
        b"{\"schema_version\":\"forge.review-queue-links-request/1\",\"links\":[],\"links\":[]}"
            .to_vec(),
        b"{\"schema_version\":\"forge.review-queue-links-request/1\",\"links\":[]} {}".to_vec(),
    ];
    let mut bom = vec![0xef, 0xbb, 0xbf];
    bom.extend_from_slice(b"{}");
    cases.push(bom);
    cases.push(format!("{{\"schema_version\":\"{LINKS_SCHEMA}\",\"links\":[{{\"old_item_id\":\"{OLD}\",\"new_item_id\":\"{NEW}\",\"finding_ids\":[],\"finding_ids\":[]}}]}}").into_bytes());
    for raw in cases {
        assert!(matches!(
            decode_links(&raw, &mut ContractLedger::default(), &mut NoopControl),
            Err(ContractError::Invalid)
        ));
    }
}

/// The actual private decoder admits raw equality and rejects plus-one complete extent.
#[test]
fn raw_request_limit_equality_and_plus_one_have_sticky_capacity() {
    let mut raw = format!("{{\"schema_version\":\"{LINKS_SCHEMA}\",\"links\":[]}}").into_bytes();
    raw.resize(RAW_CAP, b' ');
    decode_links(&raw, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    raw.push(b' ');
    let mut ledger = ContractLedger::default();
    assert!(matches!(
        decode_links(&raw, &mut ledger, &mut NoopControl),
        Err(ContractError::Capacity)
    ));
    assert_eq!(ledger.checkpoint(&mut NoopControl), Err(ContractError::Capacity));
}

/// Individual edge maxima do not promise fitting the stricter complete raw ceiling.
#[test]
fn raw_ceiling_precedes_individual_edge_maximum() {
    for count in [EDGE_CAP, EDGE_CAP + 1] {
        let links:Vec<_>=(1..=count).map(|i| json!({"old_item_id":OLD,"new_item_id":format!("00000000-0000-4000-8000-{i:012x}"),"finding_ids":[]})).collect();
        let raw =
            serde_json::to_vec(&json!({"schema_version":LINKS_SCHEMA,"links":links})).unwrap();
        assert!(raw.len() > RAW_CAP);
        assert!(matches!(
            decode_links(&raw, &mut ContractLedger::default(), &mut NoopControl),
            Err(ContractError::Capacity)
        ));
    }
}

/// Actual cooperative failure delivered on a chosen recorded checkpoint.
#[derive(Clone, Copy)]
enum Stop {
    /// Typed original interruption.
    Interrupted(Interruption),
    /// Ordinary caller failure, not fabricated capacity.
    Ordinary,
}

/// One actual caller shared across all decoders and graph preparation.
struct Caller<'a> {
    /// Actual observed invocation checkpoint counter.
    calls: &'a Cell<usize>,
    /// Actual future boundary and typed result, externally adjustable without replacing owner.
    refuse: &'a Cell<Option<(usize, Stop)>>,
}

impl WorkControl for Caller<'_> {
    /// Return exact actual caller error at the selected boundary.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        let current = self.calls.get() + 1;
        self.calls.set(current);
        match self.refuse.get() {
            Some((at, Stop::Interrupted(reason))) if current == at => {
                Err(WorkError::Interrupted(reason))
            }
            Some((at, Stop::Ordinary)) if current == at => {
                Err(WorkError::Failed(crate::workspace::contract::Error::invalid()))
            }
            _ => Ok(()),
        }
    }
    /// `ReviewControl` itself retains the actual typed first interruption.
    fn interruption(&self) -> Option<Interruption> {
        None
    }
}

/// A real syntax refusal still crosses the actual final original-control fence.
#[test]
fn post_parse_failure_fence_preserves_all_actual_control_classifications() {
    let baseline_calls = Cell::new(0);
    let refusal = Cell::new(None);
    let mut caller = Caller { calls: &baseline_calls, refuse: &refusal };
    {
        let mut control = ReviewControl::accept(&mut caller);
        assert!(matches!(
            decode_links(b"{", &mut ContractLedger::default(), &mut control),
            Err(ContractError::Invalid)
        ));
    }
    let terminal = baseline_calls.get();
    assert!(terminal > 2);
    for (stop, expected) in [
        (Stop::Ordinary, ContractError::ControlFailed),
        (
            Stop::Interrupted(Interruption::CancelRequested),
            ContractError::Interrupted(Interruption::CancelRequested),
        ),
        (
            Stop::Interrupted(Interruption::Shutdown),
            ContractError::Interrupted(Interruption::Shutdown),
        ),
        (
            Stop::Interrupted(Interruption::DeadlineExceeded),
            ContractError::Interrupted(Interruption::DeadlineExceeded),
        ),
    ] {
        let calls = Cell::new(0);
        let refusal = Cell::new(Some((terminal, stop)));
        let mut caller = Caller { calls: &calls, refuse: &refusal };
        let mut control = ReviewControl::accept(&mut caller);
        let mut ledger = ContractLedger::default();
        assert!(
            matches!(decode_links(b"{",&mut ledger,&mut control),Err(error) if error==expected)
        );
        assert_eq!(ledger.checkpoint(&mut control), Err(expected));
        assert_eq!(calls.get(), terminal);
    }
}

/// A single actual ledger/control remains stopped across complete queue/request/graph data phases.
#[test]
fn same_original_ledger_capacity_blocks_graph_without_another_caller_probe() {
    let old = queue(OLD, &[Domain::MappingAssertion]);
    let new = queue(NEW, &[Domain::MappingAssertion]);
    let old_raw = raw(&old);
    let new_raw = raw(&new);
    let link_raw = request(&old, &new, &[(0, 0, &[])]);
    let calls = Cell::new(0);
    let refusal = Cell::new(None);
    let mut caller = Caller { calls: &calls, refuse: &refusal };
    let mut control = ReviewControl::accept(&mut caller);
    let mut ledger = ContractLedger::default();
    let a = decode_queue(&old_raw, &mut ledger, &mut control).unwrap();
    let b = decode_queue(&new_raw, &mut ledger, &mut control).unwrap();
    let r = decode_links(&link_raw, &mut ledger, &mut control).unwrap();
    let observed = calls.get();
    assert_eq!(ledger.derived(33_554_433), Err(ContractError::Capacity));
    refusal.set(Some((observed + 1, Stop::Ordinary)));
    assert!(matches!(
        prepare_graph(&a, &b, &r, &mut ledger, &mut control),
        Err(ContractError::Capacity)
    ));
    assert_eq!(calls.get(), observed);
}

/// Complete graph success can be refused at its observed final original-control fence.
#[test]
fn genuine_graph_final_fence_discards_plain_success() {
    let old = queue(OLD, &[Domain::MappingAssertion]);
    let new = queue(NEW, &[Domain::MappingAssertion]);
    let old_raw = raw(&old);
    let new_raw = raw(&new);
    let link_raw = request(&old, &new, &[(0, 0, &[FINDING_A])]);
    let calls = Cell::new(0);
    let refusal = Cell::new(None);
    let mut caller = Caller { calls: &calls, refuse: &refusal };
    let mut control = ReviewControl::accept(&mut caller);
    let mut ledger = ContractLedger::default();
    let a = decode_queue(&old_raw, &mut ledger, &mut control).unwrap();
    let b = decode_queue(&new_raw, &mut ledger, &mut control).unwrap();
    let r = decode_links(&link_raw, &mut ledger, &mut control).unwrap();
    let prior = calls.get();
    prepare_graph(&a, &b, &r, &mut ledger, &mut control).unwrap();
    let graph_calls = calls.get() - prior;
    assert!(graph_calls > 1);
    refusal
        .set(Some((calls.get() + graph_calls, Stop::Interrupted(Interruption::CancelRequested))));
    assert!(matches!(
        prepare_graph(&a, &b, &r, &mut ledger, &mut control),
        Err(ContractError::Interrupted(Interruption::CancelRequested))
    ));
    assert_eq!(
        ledger.checkpoint(&mut control),
        Err(ContractError::Interrupted(Interruption::CancelRequested))
    );
}

/// Fixed endpoint export excludes private context, authors, assignments and native prose fields.
#[test]
fn endpoint_fragment_has_exact_minimized_field_whitelist() {
    let old = queue(OLD, &[Domain::MappingAssertion]);
    let new = queue(NEW, &[Domain::MappingAssertion]);
    let old_raw = raw(&old);
    let new_raw = raw(&new);
    let link_raw = request(&old, &new, &[(0, 0, &[FINDING_A])]);
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let a = decode_queue(&old_raw, &mut ledger, &mut control).unwrap();
    let b = decode_queue(&new_raw, &mut ledger, &mut control).unwrap();
    let r = decode_links(&link_raw, &mut ledger, &mut control).unwrap();
    let graph = prepare_graph(&a, &b, &r, &mut ledger, &mut control).unwrap();
    let value = serde_json::to_value(graph.links()[0].old_endpoint()).unwrap();
    assert_eq!(value.as_object().unwrap().len(), 11);
    for field in [
        "key",
        "item_id",
        "domain",
        "adapter_version",
        "subject_id",
        "subject_sha256",
        "context_sha256",
        "policy_key",
        "policy_sha256",
        "requested_action",
        "source_keys",
    ] {
        assert!(value.get(field).is_some());
    }
    for field in
        ["document_version", "href", "prose", "context", "author_keys", "assignments", "rationale"]
    {
        assert!(value.get(field).is_none());
    }
    assert_eq!(graph.links()[0].requested_findings(), &[FINDING_A.to_owned()]);
}
