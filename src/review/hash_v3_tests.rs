//! Plain framing/mutation/control regressions for the seven non-authorizing /3 profiles.
//! Independent literal frame construction uses no candidate Stream or native owner.

use super::*;
use crate::review::wire_v3::{Assignment, SeatRequirement, Substitution};
use crate::workspace::contract::Error;
use crate::workspace::preparation::{
    Interruption, NoopControl, ProgressUpdate, Stage, WorkError, WorkResult,
};

/// Exact fixed lowercase digest operands decode to repeated raw bytes in the oracle.
const H1: &str = "1111111111111111111111111111111111111111111111111111111111111111";
/// A distinct full raw digest operand.
const H2: &str = "2222222222222222222222222222222222222222222222222222222222222222";
/// A third exact full digest operand.
const H3: &str = "3333333333333333333333333333333333333333333333333333333333333333";
/// A fourth exact full digest operand.
const H4: &str = "4444444444444444444444444444444444444444444444444444444444444444";
/// Canonical review identity, separate from native UUID spelling.
const QUEUE: &str = "7e4b503b-f14f-43c2-952b-f34f29f132c0";
/// Canonical occurrence identity.
const RESPONSE: &str = "1f1a3e68-0e59-4b53-a71a-1a91655ef774";
/// Second canonical occurrence identity.
const RESPONSE2: &str = "4c53a7d4-2db0-4db6-8a34-bcb7c7c4e80a";
/// Original native uppercase spelling deliberately retained by the plain pin frame.
const ROOT: &str = "F21A0D40-E1A6-4C4D-B153-71A79BFE9016";

/// Independent normative U, expressed as byte shifts rather than Stream's `to_le_bytes`.
fn u(out: &mut Vec<u8>, value: u64) {
    for shift in [0, 8, 16, 24, 32, 40, 48, 56] {
        out.push(u8::try_from((value >> shift) & 255).unwrap());
    }
}
/// Independent normative S over exact UTF-8, including generic embedded NUL.
fn s(out: &mut Vec<u8>, value: &str) {
    u(out, u64::try_from(value.len()).unwrap());
    out.extend_from_slice(value.as_bytes());
}
/// Independent normative O; present-empty gets its own length frame.
fn o(out: &mut Vec<u8>, value: Option<&str>) {
    match value {
        None => out.push(0),
        Some(v) => {
            out.push(1);
            s(out, v);
        }
    }
}
/// Independent fixed digest operand bytes, not candidate lower-hex decoding.
fn h(out: &mut Vec<u8>, byte: u8) {
    out.extend_from_slice(&[byte; 32]);
}
/// Independent string list grammar in supplied order.
fn list(out: &mut Vec<u8>, rows: &[String]) {
    u(out, u64::try_from(rows.len()).unwrap());
    for row in rows {
        s(out, row);
    }
}
/// Compute only the independent test oracle's SHA-256; no production frame helper is reused.
fn digest(raw: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut result = String::new();
    for byte in Sha256::digest(raw) {
        write!(&mut result, "{byte:02x}").unwrap();
    }
    result
}
/// Construct every /3 pin kind as ordinary declarations, never a native fixture/proof.
fn make_pins() -> Vec<SourcePinV3> {
    let rows = [
        (
            "authoring:applicability",
            SourceKindV3::ApplicabilityManifest,
            Some("forge.applicability/1"),
            "forge.authoring-applicability-intrinsic/1",
            None,
        ),
        (
            "authoring:clause:0",
            SourceKindV3::HumanClause,
            None,
            "forge.authoring-clause-validated/1",
            None,
        ),
        (
            "authoring:framework",
            SourceKindV3::Framework,
            None,
            "forge.authoring-framework-native/1",
            Some(NativeModelV3::Profile),
        ),
        (
            "authoring:gap-report",
            SourceKindV3::GapReport,
            Some("forge.applicability-report/1"),
            "forge.authoring-gap-report-complete/1",
            None,
        ),
        (
            "authoring:mapping:0",
            SourceKindV3::MappingCollection,
            None,
            "forge.authoring-mapping-native/1",
            Some(NativeModelV3::MappingCollection),
        ),
        (
            "authoring:pack",
            SourceKindV3::AuthoringPack,
            Some("forge.authoring-pack/1"),
            "forge.authoring-pack-intrinsic/1",
            None,
        ),
        (
            "authoring:plan",
            SourceKindV3::StoredPlan,
            Some("forge.authoring-plan/1"),
            "forge.authoring-plan-complete-equality/1",
            None,
        ),
        (
            "authoring:project",
            SourceKindV3::AuthorProject,
            Some("forge.author-project/1"),
            "forge.author-project-intrinsic/1",
            None,
        ),
        (
            "authoring:resolved",
            SourceKindV3::ResolvedCatalog,
            None,
            "forge.authoring-resolved-catalog-native/1",
            Some(NativeModelV3::Catalog),
        ),
    ];
    rows.into_iter()
        .map(|(key, kind, schema, profile, model)| SourcePinV3 {
            artifact_key: key.into(),
            kind,
            raw_sha256: H1.into(),
            byte_length: 512,
            schema_identity: schema.map(str::to_owned),
            validation_profile: profile.into(),
            native_model: model,
            native_root_uuid: model.map(|_| ROOT.into()),
        })
        .collect()
}
/// Full literal P-array oracle: fixed tag/model vocabulary is independent of `kind_tag`.
fn pin_frames(out: &mut Vec<u8>, pins: &[SourcePinV3]) {
    u(out, u64::try_from(pins.len()).unwrap());
    for pin in pins {
        s(out, &pin.artifact_key);
        let tag = match pin.artifact_key.as_str() {
            "authoring:project" => 1,
            "authoring:pack" => 2,
            "authoring:gap-report" => 3,
            "authoring:applicability" => 4,
            "authoring:framework" => 5,
            "authoring:resolved" => 6,
            "authoring:plan" => 9,
            key if key.starts_with("authoring:mapping:") => 7,
            key if key.starts_with("authoring:clause:") => 8,
            _ => panic!("ordinary oracle input key"),
        };
        out.push(tag);
        h(out, 0x11);
        u(out, pin.byte_length);
        o(out, pin.schema_identity.as_deref());
        s(out, &pin.validation_profile);
        let model = match pin.artifact_key.as_str() {
            "authoring:framework" => Some("profile"),
            "authoring:resolved" => Some("catalog"),
            key if key.starts_with("authoring:mapping:") => Some("mapping-collection"),
            _ => None,
        };
        o(out, model);
        o(out, pin.native_root_uuid.as_deref());
    }
}
/// All native provenance roles in actual normative tuple order as ordinary operands.
fn make_provenance() -> Vec<NativeProvenance<'static>> {
    [
        ("applicability-manifest", "applicability.json"),
        ("author-project", "project.json"),
        ("authoring-pack", "pack.json"),
        ("framework", "profile.json"),
        ("gap-report", "gap.json"),
        ("human-clause-human-one", "clauses/one.md"),
        ("mapping-collection-0", "mappings/one.json"),
        ("resolved-catalog", "catalog.json"),
    ]
    .into_iter()
    .map(|(role, path)| NativeProvenance { role, path, raw_sha256: H1, byte_length: 512 })
    .collect()
}
/// Independent profile2 full row frame bytes, never a selected native subset.
fn provenance_bytes(rows: &[NativeProvenance<'_>]) -> Vec<u8> {
    let mut out = b"forge.authoring-plan-review-native-inputs/1\0".to_vec();
    u(&mut out, u64::try_from(rows.len()).unwrap());
    for row in rows {
        s(&mut out, row.role);
        s(&mut out, row.path);
        h(&mut out, 0x11);
        u(&mut out, row.byte_length);
    }
    out
}
/// Exact minimized context declaration.
fn make_snapshot() -> ContextSnapshot {
    ContextSnapshot {
        reason_codes: vec!["authoring-plan-current".into()],
        related_subject_ids: vec![],
    }
}
/// Asserted policy includes a retained unused substitution and empty-abstention reason.
fn make_policy() -> ReviewPolicy {
    ReviewPolicy {
        key: "policy".into(),
        seats: vec![SeatRequirement { role_key: "security".into(), count: 2 }],
        substitutions: vec![Substitution {
            seat_role: "security".into(),
            reviewer_key: "unused".into(),
            asserted_role: "alternate".into(),
            reason_code: "recorded".into(),
        }],
        abstention_rule: AbstentionRule::Nonapproving,
        empty_abstention_reasons: vec!["unavailable".into()],
        author_separation: AuthorSeparation::DeclaredKeys,
    }
}
/// Ordinary item data with no captured source, native currentness or eligible reviewer proof.
fn make_item(pins: &[SourcePinV3]) -> ReviewItemV3 {
    ReviewItemV3 {
        key: "plan-review".into(),
        item_id: QUEUE.into(),
        domain: DomainV3::AuthoringPlan,
        adapter_version: ADAPTER.into(),
        subject_id: format!("authoring-plan:{H1}"),
        requested_action: RequestedAction::ReReview,
        source_keys: pins.iter().map(|p| p.artifact_key.clone()).collect(),
        subject_sha256: H2.into(),
        context: make_snapshot(),
        context_sha256: H3.into(),
        policy_key: "policy".into(),
        policy_sha256: H4.into(),
        author_keys: vec!["author".into()],
        assignments: vec![Assignment {
            reviewer_key: "reviewer".into(),
            role_key: "security".into(),
        }],
        due_at: Some("2026-10-06T12:00:00Z".into()),
        allowed_dispositions: vec![
            Disposition::Approve,
            Disposition::Reject,
            Disposition::RequestChanges,
            Disposition::Abstain,
            Disposition::Superseded,
        ],
    }
}
/// Independent exact selected policy framing, including every unused declaration.
fn policy_bytes(pins: &[SourcePinV3], policy: &ReviewPolicy, item: &ReviewItemV3) -> Vec<u8> {
    let mut out = b"forge.review-policy-binding/3\0".to_vec();
    s(&mut out, "authoring-plan");
    s(&mut out, "forge.authoring-plan-review/1");
    pin_frames(&mut out, pins);
    s(&mut out, &policy.key);
    u(&mut out, u64::try_from(policy.seats.len()).unwrap());
    for row in &policy.seats {
        s(&mut out, &row.role_key);
        u(&mut out, u64::from(row.count));
    }
    u(&mut out, u64::try_from(policy.substitutions.len()).unwrap());
    for row in &policy.substitutions {
        for v in [&row.seat_role, &row.reviewer_key, &row.asserted_role, &row.reason_code] {
            s(&mut out, v);
        }
    }
    out.push(1);
    list(&mut out, &policy.empty_abstention_reasons);
    out.push(1);
    list(&mut out, &item.author_keys);
    u(&mut out, u64::try_from(item.assignments.len()).unwrap());
    for row in &item.assignments {
        s(&mut out, &row.reviewer_key);
        s(&mut out, &row.role_key);
    }
    o(&mut out, item.due_at.as_deref());
    u(&mut out, 5);
    out.extend_from_slice(&[1, 2, 3, 4, 5]);
    out
}
/// Actual-control observer; every invocation is made by the production same-ledger fence.
#[derive(Default)]
struct Control {
    /// Exact observed checkpoint count.
    seen: usize,
    /// Optional one-based stop occurrence, calibrated from a real candidate call.
    stop: Option<usize>,
    /// Ordinary `WorkControl` failure rather than interruption.
    fail: bool,
    /// Report a cancellation after returning Ok from the selected checkpoint.
    after_ok: bool,
    /// First real cancellation reason reported by this controller.
    interrupted: Option<Interruption>,
}
impl WorkControl for Control {
    /// Preserve exact original control/stage and distinguish failed from after-Ok interruption.
    fn checkpoint(&mut self, stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        assert!(matches!(stage, Stage::PrepareDomain));
        self.seen += 1;
        if self.stop == Some(self.seen) {
            if self.fail {
                return Err(WorkError::Failed(Error::invalid()));
            }
            self.interrupted = Some(Interruption::CancelRequested);
            if !self.after_ok {
                return Err(WorkError::Interrupted(Interruption::CancelRequested));
            }
        }
        Ok(())
    }
    /// Expose the actual selected post-Ok stop to the production fence.
    fn interruption(&self) -> Option<Interruption> {
        self.interrupted
    }
}

/// Exact contract primitive literals distinguish raw bytes, endian order, null and empty.
#[test]
fn primitive_frames_match_independent_literal_bytes() {
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let mut stream = Stream::new(b"", 1024, &mut ledger, &mut control).unwrap();
    stream.u64(0).unwrap();
    stream.u64(1).unwrap();
    stream.u64(256).unwrap();
    stream.u64(u64::MAX).unwrap();
    stream.string("").unwrap();
    stream.string("a").unwrap();
    stream.string("é").unwrap();
    stream.string("a\0b").unwrap();
    stream.optional_string(None).unwrap();
    stream.optional_string(Some("")).unwrap();
    stream.hash(&"0".repeat(64)).unwrap();
    stream.strings(&[]).unwrap();
    stream.strings(&["a".into(), "b".into()]).unwrap();
    let mut literal = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255,
        255, 255, 255, 255,
    ];
    literal.extend_from_slice(&[
        0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 97, 2, 0, 0, 0, 0, 0, 0, 0, 195, 169, 3, 0,
        0, 0, 0, 0, 0, 0, 97, 0, 98, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    literal.extend_from_slice(&[0; 32]);
    literal.extend_from_slice(&[0; 8]);
    literal.extend_from_slice(&[
        2, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 97, 1, 0, 0, 0, 0, 0, 0, 0, 98,
    ]);
    assert_eq!(stream.finish_hex().unwrap(), digest(&literal));
}
/// Stable identity depends solely on exact project identity, without source/native revision.
#[test]
fn subject_id_matches_complete_literal_identity_profile() {
    let mut expected = b"forge.authoring-plan-review-subject-id/1\0".to_vec();
    for value in ["forge.author-project/1", "forge.authoring-plan/1", "demo-plan"] {
        s(&mut expected, value);
    }
    let actual = subject_id("demo-plan", &mut ContractLedger::default(), &mut NoopControl).unwrap();
    assert_eq!(actual, format!("authoring-plan:{}", digest(&expected)));
    assert_eq!(actual.len(), 79);
    assert_ne!(
        actual,
        subject_id("other-plan", &mut ContractLedger::default(), &mut NoopControl).unwrap()
    );
}
/// Every native role/path/raw32/extent is framed in complete exact tuple order.
#[test]
fn native_provenance_matches_independent_complete_frame() {
    let rows = make_provenance();
    let expected = provenance_bytes(&rows);
    assert_eq!(
        native_provenance(&rows, &mut ContractLedger::default(), &mut NoopControl).unwrap(),
        digest(&expected)
    );
    let mut changed = make_provenance();
    changed[5].byte_length += 1;
    assert_ne!(
        native_provenance(&rows, &mut ContractLedger::default(), &mut NoopControl).unwrap(),
        native_provenance(&changed, &mut ContractLedger::default(), &mut NoopControl).unwrap()
    );
}
/// Subject preserves raw native `as_of` spelling and separately binds actual stored bytes.
#[test]
fn subject_matches_literal_full_pin_profile_and_original_time() {
    let pins = make_pins();
    let as_of = "2026-10-05T12:00:00+00:00";
    let mut expected = b"forge.authoring-plan-review-subject/1\0".to_vec();
    for v in [
        "authoring-plan",
        "forge.authoring-plan-review/1",
        "forge.author-project/1",
        "forge.authoring-plan/1",
        "demo-plan",
        as_of,
    ] {
        s(&mut expected, v);
    }
    h(&mut expected, 0x22);
    h(&mut expected, 0x33);
    pin_frames(&mut expected, &pins);
    let original = subject(
        "demo-plan",
        as_of,
        H2,
        H3,
        &pins,
        &mut ContractLedger::default(),
        &mut NoopControl,
    )
    .unwrap();
    assert_eq!(original, digest(&expected));
    assert_ne!(
        original,
        subject(
            "demo-plan",
            "2026-10-05T12:00:00Z",
            H2,
            H3,
            &pins,
            &mut ContractLedger::default(),
            &mut NoopControl
        )
        .unwrap()
    );
    assert_ne!(
        original,
        subject(
            "demo-plan",
            as_of,
            H2,
            H4,
            &pins,
            &mut ContractLedger::default(),
            &mut NoopControl
        )
        .unwrap()
    );
}
/// Complete fixed minimized context framing; any extra reason/related text refuses whole data.
#[test]
fn context_matches_literal_and_refuses_extra_fields() {
    let pins = make_pins();
    let snapshot = make_snapshot();
    let mut expected = b"forge.review-context/3\0".to_vec();
    s(&mut expected, "authoring-plan");
    s(&mut expected, "forge.authoring-plan-review/1");
    pin_frames(&mut expected, &pins);
    u(&mut expected, 1);
    s(&mut expected, "authoring-plan-current");
    u(&mut expected, 0);
    assert_eq!(
        context(&pins, &snapshot, &mut ContractLedger::default(), &mut NoopControl).unwrap(),
        digest(&expected)
    );
    let mut extra = snapshot;
    extra.related_subject_ids.push("private-native-text".into());
    assert!(matches!(
        context(&pins, &extra, &mut ContractLedger::default(), &mut NoopControl),
        Err(ContractError::Invalid)
    ));
}
/// Retained unused policy assertions, authors, assignments, deadlines and all dispositions bind.
#[test]
fn policy_matches_literal_including_unused_asserted_values() {
    let pins = make_pins();
    let item = make_item(&pins);
    let mut selected = make_policy();
    let original =
        policy(&pins, &selected, &item, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    assert_eq!(original, digest(&policy_bytes(&pins, &selected, &item)));
    selected.substitutions[0].reason_code = "another".into();
    assert_ne!(
        original,
        policy(&pins, &selected, &item, &mut ContractLedger::default(), &mut NoopControl).unwrap()
    );
    let selected = make_policy();
    let mut no_due = item;
    no_due.due_at = None;
    assert_ne!(
        original,
        policy(&pins, &selected, &no_due, &mut ContractLedger::default(), &mut NoopControl)
            .unwrap()
    );
}
/// Exact raw32 digest is `UUIDv5` name; output item UUID/raw Queue hash are absent self-inputs.
#[test]
fn item_identity_matches_literal_raw_digest_namespace() {
    let pins = make_pins();
    let mut item = make_item(&pins);
    let mut expected = b"forge.review-item/3\0".to_vec();
    for v in [
        QUEUE,
        "plan-review",
        "authoring-plan",
        "forge.authoring-plan-review/1",
        item.subject_id.as_str(),
    ] {
        s(&mut expected, v);
    }
    expected.push(1);
    list(&mut expected, &item.source_keys);
    pin_frames(&mut expected, &pins);
    h(&mut expected, 0x22);
    h(&mut expected, 0x33);
    h(&mut expected, 0x44);
    let name: [u8; 32] = Sha256::digest(&expected).into();
    let namespace = Uuid::parse_str("9f7c24f8-561d-55a1-a43d-fc4d8159b460").unwrap();
    let expected = Uuid::new_v5(&namespace, &name).to_string();
    assert_eq!(
        item_id(QUEUE, &item, &pins, &mut ContractLedger::default(), &mut NoopControl).unwrap(),
        expected
    );
    item.item_id = RESPONSE.into();
    assert_eq!(
        item_id(QUEUE, &item, &pins, &mut ContractLedger::default(), &mut NoopControl).unwrap(),
        expected
    );
    item.subject_sha256 = H4.into();
    assert_ne!(
        item_id(QUEUE, &item, &pins, &mut ContractLedger::default(), &mut NoopControl).unwrap(),
        expected
    );
}
/// Exact duplicate Response occurrences remain separate; raw Queue and locator bind generation.
#[test]
fn generation_matches_literal_complete_occurrences_and_order() {
    let pins = make_pins();
    let aux = [AuxiliaryOriginal {
        purpose: AuxiliaryPurposeV3::Locator,
        raw_sha256: H3,
        byte_length: 41,
    }];
    let rows = [
        ResponseOriginal { response_id: RESPONSE, raw_sha256: H4, byte_length: 73 },
        ResponseOriginal { response_id: RESPONSE, raw_sha256: H4, byte_length: 73 },
        ResponseOriginal { response_id: RESPONSE2, raw_sha256: H1, byte_length: 75 },
    ];
    let mut expected = b"forge.review-current-closure/3\0".to_vec();
    s(&mut expected, "authoring-plan");
    s(&mut expected, "forge.authoring-plan-review/1");
    h(&mut expected, 0x22);
    pin_frames(&mut expected, &pins);
    h(&mut expected, 0x11);
    u(&mut expected, 997);
    u(&mut expected, 1);
    expected.push(1);
    h(&mut expected, 0x33);
    u(&mut expected, 41);
    u(&mut expected, 3);
    for (id, byte, len) in [(RESPONSE, 0x44, 73), (RESPONSE, 0x44, 73), (RESPONSE2, 0x11, 75)] {
        s(&mut expected, id);
        h(&mut expected, byte);
        u(&mut expected, len);
    }
    let original = closure_generation(
        ClosureGenerationInputs {
            native: H2,
            pins: &pins,
            queue: H1,
            queue_length: 997,
            aux: &aux,
            responses: &rows,
        },
        &mut ContractLedger::default(),
        &mut NoopControl,
    )
    .unwrap();
    assert_eq!(original, digest(&expected));
    let swapped = [
        ResponseOriginal { response_id: RESPONSE2, raw_sha256: H1, byte_length: 75 },
        ResponseOriginal { response_id: RESPONSE, raw_sha256: H4, byte_length: 73 },
        ResponseOriginal { response_id: RESPONSE, raw_sha256: H4, byte_length: 73 },
    ];
    assert_ne!(
        original,
        closure_generation(
            ClosureGenerationInputs {
                native: H2,
                pins: &pins,
                queue: H1,
                queue_length: 997,
                aux: &aux,
                responses: &swapped,
            },
            &mut ContractLedger::default(),
            &mut NoopControl
        )
        .unwrap()
    );
    assert_ne!(
        original,
        closure_generation(
            ClosureGenerationInputs {
                native: H2,
                pins: &pins,
                queue: H1,
                queue_length: 997,
                aux: &aux,
                responses: &rows[..2],
            },
            &mut ContractLedger::default(),
            &mut NoopControl
        )
        .unwrap()
    );
    let wrong = [AuxiliaryOriginal {
        purpose: AuxiliaryPurposeV3::InitPolicy,
        raw_sha256: H3,
        byte_length: 41,
    }];
    assert!(matches!(
        closure_generation(
            ClosureGenerationInputs {
                native: H2,
                pins: &pins,
                queue: H1,
                queue_length: 997,
                aux: &wrong,
                responses: &rows,
            },
            &mut ContractLedger::default(),
            &mut NoopControl
        ),
        Err(ContractError::Invalid)
    ));
}
/// Full native root spelling is retained; null/model/profile/key/hash/extent mutations all bind or refuse.
#[test]
fn every_pin_field_and_all_nine_kind_tags_are_observed() {
    let pins = make_pins();
    let baseline =
        context(&pins, &make_snapshot(), &mut ContractLedger::default(), &mut NoopControl).unwrap();
    assert_eq!(pins.len(), 9);
    assert_eq!(
        pins.iter().map(|p| kind_tag(p.kind)).collect::<Vec<_>>(),
        vec![4, 8, 5, 3, 7, 2, 9, 1, 6]
    );
    let mut altered = make_pins();
    altered[2].native_root_uuid = Some(ROOT.to_ascii_lowercase());
    assert_ne!(
        baseline,
        context(&altered, &make_snapshot(), &mut ContractLedger::default(), &mut NoopControl)
            .unwrap()
    );
    let mut altered = make_pins();
    altered[0].raw_sha256 = H2.into();
    assert_ne!(
        baseline,
        context(&altered, &make_snapshot(), &mut ContractLedger::default(), &mut NoopControl)
            .unwrap()
    );
    let mut altered = make_pins();
    altered[0].byte_length += 1;
    assert_ne!(
        baseline,
        context(&altered, &make_snapshot(), &mut ContractLedger::default(), &mut NoopControl)
            .unwrap()
    );
    for field in 0..5 {
        let mut altered = make_pins();
        match field {
            0 => altered[2].native_root_uuid = None,
            1 => altered[0].schema_identity = None,
            2 => altered[2].native_model = Some(NativeModelV3::Catalog),
            3 => altered[0].validation_profile = "different".into(),
            _ => altered[1].artifact_key = "authoring:clause:1".into(),
        }
        assert!(matches!(
            context(&altered, &make_snapshot(), &mut ContractLedger::default(), &mut NoopControl),
            Err(ContractError::Invalid)
        ));
    }
}
/// Lexicographic 10-before-2 ordinal order survives without numerical sorting or gaps.
#[test]
fn complete_ordinal_family_uses_lexicographic_declared_order() {
    let mut pins = make_pins();
    for ordinal in 1..12 {
        pins.push(SourcePinV3 {
            artifact_key: format!("authoring:mapping:{ordinal}"),
            kind: SourceKindV3::MappingCollection,
            raw_sha256: H1.into(),
            byte_length: 2,
            schema_identity: None,
            validation_profile: "forge.authoring-mapping-native/1".into(),
            native_model: Some(NativeModelV3::MappingCollection),
            native_root_uuid: Some(ROOT.into()),
        });
    }
    pins.sort_by(|a, b| a.artifact_key.cmp(&b.artifact_key));
    assert!(
        context(&pins, &make_snapshot(), &mut ContractLedger::default(), &mut NoopControl).is_ok()
    );
    let ten = pins.iter().position(|p| p.artifact_key == "authoring:mapping:10").unwrap();
    let two = pins.iter().position(|p| p.artifact_key == "authoring:mapping:2").unwrap();
    assert!(ten < two);
    pins.remove(ten);
    assert!(matches!(
        context(&pins, &make_snapshot(), &mut ContractLedger::default(), &mut NoopControl),
        Err(ContractError::Invalid)
    ));
}
/// Whole ordinary native roster rejects omitted rows, path aliases and route spelling changes.
#[test]
fn complete_native_rows_refuse_subset_alias_and_nonportable_route() {
    let mut rows = make_provenance();
    rows.remove(0);
    assert!(matches!(
        native_provenance(&rows, &mut ContractLedger::default(), &mut NoopControl),
        Err(ContractError::Invalid)
    ));
    let mut rows = make_provenance();
    rows[1].path = "PROFILE.JSON";
    assert!(matches!(
        native_provenance(&rows, &mut ContractLedger::default(), &mut NoopControl),
        Err(ContractError::Invalid)
    ));
    let mut rows = make_provenance();
    rows[5].path = "clauses\\one.md";
    assert!(matches!(
        native_provenance(&rows, &mut ContractLedger::default(), &mut NoopControl),
        Err(ContractError::Invalid)
    ));
    let mut rows = make_provenance();
    rows.swap(0, 1);
    assert!(matches!(
        native_provenance(&rows, &mut ContractLedger::default(), &mut NoopControl),
        Err(ContractError::Invalid)
    ));
}
/// Actual full plain file bytes are hashed including whitespace/LF; no native validation is claimed.
#[test]
fn actual_raw_file_formatting_changes_subject_and_generation() {
    let temp = tempfile::tempdir().unwrap();
    let route = temp.path().join("stored-plan.json");
    std::fs::write(&route, b"{\"plain\":1}\n").unwrap();
    let first = std::fs::read(&route).unwrap();
    let first_hash = digest(&first);
    std::fs::write(&route, b"{ \"plain\" : 1 }\n\n").unwrap();
    let second = std::fs::read(&route).unwrap();
    let second_hash = digest(&second);
    assert_ne!(first_hash, second_hash);
    let pins = make_pins();
    let before = subject(
        "demo-plan",
        "2026-10-05T12:00:00Z",
        H2,
        &first_hash,
        &pins,
        &mut ContractLedger::default(),
        &mut NoopControl,
    )
    .unwrap();
    let after = subject(
        "demo-plan",
        "2026-10-05T12:00:00Z",
        H2,
        &second_hash,
        &pins,
        &mut ContractLedger::default(),
        &mut NoopControl,
    )
    .unwrap();
    assert_ne!(before, after);
    let aux = [AuxiliaryOriginal {
        purpose: AuxiliaryPurposeV3::Locator,
        raw_sha256: H3,
        byte_length: 41,
    }];
    let before = closure_generation(
        ClosureGenerationInputs {
            native: H2,
            pins: &pins,
            queue: &first_hash,
            queue_length: u64::try_from(first.len()).unwrap(),
            aux: &aux,
            responses: &[],
        },
        &mut ContractLedger::default(),
        &mut NoopControl,
    )
    .unwrap();
    let after = closure_generation(
        ClosureGenerationInputs {
            native: H2,
            pins: &pins,
            queue: &second_hash,
            queue_length: u64::try_from(second.len()).unwrap(),
            aux: &aux,
            responses: &[],
        },
        &mut ContractLedger::default(),
        &mut NoopControl,
    )
    .unwrap();
    assert_ne!(before, after);
}
/// Ordinary invalid operands reach the actual same original postphase stop for every stop form.
#[test]
fn ordinary_invalid_reaches_postfence_for_failed_interrupted_and_after_ok() {
    for (fail, after_ok) in [(false, false), (true, false), (false, true)] {
        let mut control = Control { stop: Some(2), fail, after_ok, ..Control::default() };
        let mut ledger = ContractLedger::default();
        let result = subject_id("invalid key", &mut ledger, &mut control);
        assert_eq!(control.seen, 2);
        if fail {
            assert!(matches!(result, Err(ContractError::ControlFailed)));
        } else {
            assert!(matches!(
                result,
                Err(ContractError::Interrupted(Interruption::CancelRequested))
            ));
        }
        let later = subject_id("demo-plan", &mut ledger, &mut NoopControl);
        if fail {
            assert!(matches!(later, Err(ContractError::ControlFailed)));
        } else {
            assert!(matches!(
                later,
                Err(ContractError::Interrupted(Interruption::CancelRequested))
            ));
        }
    }
}
/// A genuinely successful hash is withheld at its calibrated final same-control fence.
#[test]
fn successful_profile_final_postfence_cannot_be_bypassed() {
    let mut observed = Control::default();
    subject_id("demo-plan", &mut ContractLedger::default(), &mut observed).unwrap();
    let last = observed.seen;
    assert!(last > 2);
    for after_ok in [false, true] {
        let mut control = Control { stop: Some(last), after_ok, ..Control::default() };
        assert!(matches!(
            subject_id("demo-plan", &mut ContractLedger::default(), &mut control),
            Err(ContractError::Interrupted(Interruption::CancelRequested))
        ));
        assert_eq!(control.seen, last);
    }
    let mut control = Control { stop: Some(last), fail: true, ..Control::default() };
    assert!(matches!(
        subject_id("demo-plan", &mut ContractLedger::default(), &mut control),
        Err(ContractError::ControlFailed)
    ));
}
/// Failed ordinary work remains charged and actual first Capacity precedes later controller stops.
#[test]
fn monotonic_failed_work_and_first_capacity_are_preserved() {
    let mut ledger = ContractLedger::default();
    assert!(matches!(
        subject_id("invalid key", &mut ledger, &mut NoopControl),
        Err(ContractError::Invalid)
    ));
    assert!(matches!(ledger.bytes(268_435_456), Err(ContractError::Capacity)));
    let mut untouched = ContractLedger::default();
    assert!(untouched.bytes(268_435_456).is_ok());
    let mut control = Control { stop: Some(1), fail: true, ..Control::default() };
    assert!(matches!(
        subject_id("demo-plan", &mut ledger, &mut control),
        Err(ContractError::Capacity)
    ));
    assert_eq!(control.seen, 0);
}
/// Actual 32KiB updates observe the original control; local stream overflow latches Capacity.
#[test]
fn chunk_boundary_stop_and_checked_stream_cap_are_real() {
    let raw = vec![b'x'; CHUNK + 7];
    let mut ledger = ContractLedger::default();
    let mut control = Control { stop: Some(2), ..Control::default() };
    let mut stream = Stream::new(b"", STREAM_CAP, &mut ledger, &mut control).unwrap();
    assert!(matches!(
        stream.put(&raw),
        Err(ContractError::Interrupted(Interruption::CancelRequested))
    ));
    assert_eq!(control.seen, 2);
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let result = phase(&mut ledger, &mut control, |ledger, control| {
        let mut stream = Stream::new(b"x", 1, ledger, control)?;
        stream.put(b"y")
    });
    assert!(matches!(result, Err(ContractError::Capacity)));
    assert!(matches!(
        subject_id("demo-plan", &mut ledger, &mut control),
        Err(ContractError::Capacity)
    ));
}
/// Exact lowercase digest syntax refuses uppercase/truncation; raw32 is not a framed hex string.
#[test]
fn strict_hash_operand_refusal_never_coerces_spelling() {
    let pins = make_pins();
    for invalid in ["A".repeat(64), "1".repeat(63), "g".repeat(64)] {
        assert!(matches!(
            subject(
                "demo-plan",
                "2026-10-05T12:00:00Z",
                &invalid,
                H3,
                &pins,
                &mut ContractLedger::default(),
                &mut NoopControl
            ),
            Err(ContractError::Invalid)
        ));
    }
}
