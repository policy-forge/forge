# F07 POA&M foundation: source-only contract spike

Date: 2026-10-02 (America/Los_Angeles)
Baseline: origin/main aef0ab24f77559593b6d0b4fe5027f8a5eaa857e
Status: proposed contract packet; schema bytes inspected and independently hashed in memory. No vendoring, implementation, Cargo execution, interoperability run or owner acceptance.

## Verified standards receipt

| Field | Verified value |
|---|---|
| Repository / release | usnistgov/OSCAL / v1.2.3 |
| Annotated tag object | a504f40be661d4c1758e753643b0164978a9c2f1 |
| Release commit | e061961c7702afeabffdf5e59894d35e748dade1 |
| Published | 2026-08-07T03:33:33Z |
| Asset | oscal_poam_schema.json |
| Asset URL | https://github.com/usnistgov/OSCAL/releases/download/v1.2.3/oscal_poam_schema.json |
| GitHub asset ID | 504571658 |
| Exact size | 148253 bytes |
| SHA-256 computed from fetched bytes | f4fd94487408a9589954b5b92d88965c984d4f79b85365cc574b860898759437 |
| GitHub release digest | sha256:f4fd94487408a9589954b5b92d88965c984d4f79b85365cc574b860898759437 |
| JSON Schema dialect | draft-07 |
| Schema ID | http://csrc.nist.gov/ns/oscal/1.2.3/oscal-poam-schema.json |
| External $ref values | None found; all references are fragment-local |

The release API and tag API agree with Forge's existing pinned release; the independently downloaded bytes match the reported asset digest. This is a proposed new asset row in the existing provenance process, not a vendored file or a signature/identity assurance claim. Sources: [release](https://github.com/usnistgov/OSCAL/releases/tag/v1.2.3), [release metadata](https://api.github.com/repos/usnistgov/OSCAL/releases/tags/v1.2.3), [asset metadata](https://api.github.com/repos/usnistgov/OSCAL/releases/assets/504571658), [tag metadata](https://api.github.com/repos/usnistgov/OSCAL/git/tags/a504f40be661d4c1758e753643b0164978a9c2f1).

The official artifact requires root UUID, metadata and a nonempty poam-items array. A poam-item contains title/description plus optional references, properties, links and origins. Native milestone tasks occur under a risk's remediations; task timing uses date-time values. The model reference requires either import-ssp or system-id (both are allowed),
but the JSON Schema root required array contains only uuid, metadata and
poam-items and has no composition constraint enforcing that identity choice.
A pinned import-ssp requirement is a proposed Forge profile/runtime rule. These constraints require explicit Forge mappings: a zero-selection scaffold is not a schema-valid OSCAL POA&M. Sources: [versioned reference](https://pages.nist.gov/OSCAL-Reference/models/v1.2.3/plan-of-action-and-milestones/json-reference/), [versioned outline](https://pages.nist.gov/OSCAL-Reference/models/v1.2.3/plan-of-action-and-milestones/json-outline/), [exact schema asset](https://github.com/usnistgov/OSCAL/releases/download/v1.2.3/oscal_poam_schema.json).

For later vendoring, add the exact bytes as schemas/oscal_poam_schema.json and one runtime/model=poam asset row to schemas/oscal-schema-manifest.json. Update the exact allowlist/count in tests/schema_provenance_test.rs from 11 to 12. Preserve all existing asset bytes and digests. Register a POA&M validator through the existing offline jsonschema cache and update all exhaustive OscalModelType matches through the integration owner; these are implementation recommendations only.

## Existing tools and reusable evidence

No new crate is necessary for the foundation. Cargo.toml already has serde/serde_json, jsonschema with default-features=false, sha2, uuid v5, chrono and tempfile. The schema has the same supported dialect/local-reference pattern as existing schemas. tests/schema_provenance_test.rs already checks exact sizes/digests, local-only references and offline validator construction. This spike inspected structure; it did not compile the schema or run those tests.

NIST's Java oscal-cli is a candidate independent consumer, with documented validation and format-conversion operations. It was not found on the current PATH. Exact tool/model-version compatibility must be qualified and pinned before an F21 receipt; the README's general capabilities do not prove POA&M 1.2.3 interoperability. No tool was installed or executed. [Official tool repository](https://github.com/usnistgov/oscal-cli). The current host Python lacks jsonschema/referencing; no Python package was installed.

Source patterns verified on the baseline:

- src/assessment_results/manifest.rs: closed duplicate-safe forge.assessment-results/1; explicit document/result keys, exact artifact identities, declared roles/parties and conclusion provenance.
- src/assessment_results/context.rs: AP→SSP→Profile→Catalog exact companion/import chain; input hashes and UUID/version checks; bounded reviewed-control, statement, objective, subject and task inventories.
- src/assessment_results/model.rs: length-prefixed UUID-v5 seeds [schema, document_key, kind, key]; source conclusion stable-key/content-sha256/rationale-sha256 props and origin assertions.
- src/assessment_results/baseline.rs: duplicate stable-key rejection and separate content, rationale, status and upstream changes.
- src/json_strict.rs, src/io.rs and src/cli/output.rs: strict bounded JSON, 50 MiB resource bound, shared output conventions.
- src/authoring/input.rs and src/authoring/output.rs: held/confined captures, byte revalidation and atomic no-replace publication on Linux/macOS.
- src/lifecycle/record.rs: assertion history and terminal-state concepts, without authenticated identity.

Important source-identity limit: Assessment Results metadata currently does not expose its original document key. Result and conclusion stable keys do appear. Therefore F07 can preserve exact supplied root/result/object UUIDs and keys, but cannot honestly claim to regenerate every imported UUID from artifact-only inputs. Do not reverse-engineer or fabricate the missing document key. Rebuilding from an explicitly supplied original authoring manifest is a separate optional verification path requiring an exact byte comparison.

## Decisions for owners

| Decision | Concrete recommendation | Owner / status |
|---|---|---|
| Editable authority | The closed forge.poam/1 manifest is editable; OSCAL and reports are derived snapshots. Check reads and compares, never imports hand-edited output as authority or refreshes pins silently. | Product; blocking PRD question remains open |
| Supported initial context | One FORGE-produced Assessment Results result epoch plus its explicit AP/SSP/Profile/Catalog companions. Schema-valid unsupported source extensions fail with guidance rather than being dropped. Multiple result epochs remain F10/F08 integration work. | Engineering; scope approval pending |
| Native minimum | Metadata/import-ssp, selected-source finding/risk projections, poam-items and exact hashed source links; planned risk remediations with flat milestone tasks. No fabricated risks, source outcomes, owners or dates. | Engineering; supported subset approval pending |
| Finding-only milestones | Preserve finding-only selection allowed by M-5. A poam-item has no native tasks. Decide an explicit bounded namespaced milestone projection or another reviewed representation; do not silently require a risk or synthesize one. | Engineering/Product; unresolved mapping detail for F08 |
| Full-date versus native timing | The manifest and schedule report use YYYY-MM-DD. Preserve that date as a namespaced task property in the proposed first projection. Native timing needs a supplied timestamp or an explicitly approved date/time conversion policy; never invent midnight silently. | Engineering; unresolved mapping detail |
| Workflow states | Keep the six PRD workflow labels as asserted local states; retain original Assessment Results risk/target status separately. No automatic completed-asserted→closed or accepted-risk-asserted→deviation-approved transfer. | Compliance; blocking closure/status decision remains open |
| Terminal assertion evidence | Reviewer key/role, explicit time, nonempty rationale and exact item/event/source/evidence pins; identity and authority remain asserted. Fresh linkage evidence is F08 S-4, not proof of completion. | Compliance/Legal; sufficiency/wording approval pending |
| Source acceptance | PRD063 stable-result identity and boundary acceptance remain an explicit dependency, separate from existing code. | PRD063 Engineering/Compliance owner; approval pending |

These recommendations are reviewable choices, not owner dispositions.

## Minimal scaffold and closed input contract

F07's production deliverable is init/source checking, a closed manifest foundation and stable identity primitives. Init lists source objects but emits no selected items, ownership, target dates or status events. Foundation manifest/source-only validation permits zero items; native emission requires at least one item and all F08 workflow validators. It writes only forge.poam/1 plus an explicitly requested source-inventory report; it does not serialize an empty OSCAL artifact, because the official schema requires at least one item.

Proposed CLI foundation:

- forge poam init --assessment-results FILE --assessment-plan FILE --ssp FILE --profile FILE --catalog FILE --output FILE, with explicit document-key/title/version/last-modified inputs or visibly incomplete fields consistent with existing scaffolds.
- A source-only check entry point can be introduced within the future build/check command tree. Do not expose a production build that appears complete while F08 validation is absent.
- Text/JSON source inventory names IDs, source types, exact hashes and source state only; omitted objects are unselected, not judged unnecessary.

Proposed forge.poam/1 fields:

| Field | Contract |
|---|---|
| schema_version | Exact forge.poam/1; closed structures at every authored level |
| document | Stable key; supplied title, version and RFC3339 last_modified; metadata time never read from the clock |
| source.assessment_results | Local artifact path, output href, expected SHA-256, root UUID, document version, OSCAL version |
| source.result | Exact selected result UUID and stable result key; no implicit first-result selection |
| source.context | Exact AP/SSP/Profile/Catalog artifact identities with the existing import-chain checks |
| roles / parties | Explicit local assertions; empty in init; no imported party becomes a remediation owner automatically |
| items | Empty in init; real items require immutable key, source references and reviewer-authored fields |
| review_policy | Explicit required role/count/separation contract once approved; init supplies no satisfied approvals |
| schedule_policy | Full-date semantics and a bounded due-soon window; reports require explicit --as-of |

Eligible-source inventory is output derived from the exact source, never authoritative manifest input. For each finding/risk report kind, stable_key, UUID, result UUID, canonical-object hash, source content/rationale hashes when supplied, target/control references and source state. No default titles, descriptions or evidence prose in the inventory.

For a real item, require key, title, description/outcome, nonempty source_refs, selection_review, responsible_parties, ownership_rationale, ordered milestones and explicit status evidence. source_refs contain kind=finding|risk, stable key, exact UUID and expected canonical-object hash within the chosen result. Validate the tuple; coincident IDs in another epoch/document do not match. Source file SHA-256 remains authoritative; supplied source content-sha256 properties are not trusted in place of computing/validating the source object.

A milestone requires key, explicit ordinal, title/outcome, target_date and dependency keys within the item. No dates or dependency order are derived from prose. F07 fixes identity and the bounded record shape; F08 implements the graph, schedule, history, ownership and terminal semantics. Do not accept those records as valid completion before their validators exist.

Status-evidence record shape:

- sequence, prior_state, next_state; an initial explicitly authored planned event has no prior state.
- actor_key, role_id, asserted_at, rationale.
- exact plan/item source snapshot digest and relevant milestone/owner/outcome digest.
- reviewer assertions bound to the same event payload for terminal decisions.
- optional identity-only evidence references with exact index/resource pins for F08 S-4.
- derived event UUID plus prior-event digest; append-only verification compares preserved prior records, rather than trusting array position or a caller's final state.

The visible item state is derived from validated history; do not allow a second independently editable current_status to disagree. No completed-asserted/accepted-risk-asserted event is accepted while its required evidence policy is unresolved or unavailable.

## Typed projection spike

This is a design sketch, not compiled Rust. Use owned typed source projections, not untyped serde_json::Value output or a passthrough map. Existing Assessment Results output types are Serialize-only and some labels use &'static str, so direct Deserialize reuse needs an owned adapter rather than a blanket lifetime/contract change.

Proposed source projection types: PinnedResult { identity, key, findings, risks, observations, parties, roles }; PinnedFinding and PinnedRisk preserve exact source UUID/status/provenance plus computed hash. Every accepted field in the supported subset has a typed representation; schema-valid source fields outside it are rejected explicitly until supported.

Proposed output types:

~~~rust
#[derive(Serialize)]
struct PoamEnvelope {
    #[serde(rename = "plan-of-action-and-milestones")]
    poam: PoamDocument,
}
#[derive(Serialize)]
struct PoamDocument {
    uuid: String,
    metadata: PoamMetadata,
    #[serde(rename = "import-ssp")]
    import_ssp: ImportSsp,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    findings: Vec<PoamFinding>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    risks: Vec<PoamRisk>,
    #[serde(rename = "poam-items")]
    items: Vec<PoamItem>, // runtime requires at least one for OSCAL emission
    #[serde(rename = "back-matter")]
    back_matter: BackMatter,
}
#[derive(Serialize)]
struct PoamItem {
    uuid: String, // FORGE requires it even though the schema makes it optional
    title: String,
    description: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    props: Vec<OwnedProperty>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    links: Vec<OwnedLink>,
    #[serde(rename = "related-findings", skip_serializing_if = "Vec::is_empty")]
    findings: Vec<FindingUuidRef>,
    #[serde(rename = "related-risks", skip_serializing_if = "Vec::is_empty")]
    risks: Vec<RiskUuidRef>,
}
~~~

PoamRisk has the required preserved source uuid/title/description/statement/status, source-bound properties/links and optional typed remediations. A planned RiskResponse has uuid/lifecycle/title/description and flat tasks; tasks are type=milestone with stable UUIDs, properties, dependencies and responsible roles. A source risk's status remains its asserted assessment state verbatim. OSCAL's
risk-status type accepts locally defined token values as well as its listed
standard values; any strict six-value allowlist is a Forge supported-subset rule,
not an OSCAL-wide rule. FORGE workflow status is separate namespaced data.
Optional props/links arrays must be omitted when empty (their native minItems is 1).

For source projections, do not carry dangling optional links from the Assessment Results graph into a new native graph. Either import the complete referenced closure with validated native references and imported actor definitions, or explicitly document a supported projection whose omitted optional relations remain accessible through the exact source artifact/hash link. This choice needs semantic validation by the independent tool; JSON Schema alone does not prove cross-object index correctness. Source records themselves remain byte-unchanged.

Use the new namespace https://policy-forge.github.io/ns/poam only after Engineering approves it. Properties record stable keys, source root/result/object IDs, manifest/source/context hashes, asserted workflow state and provenance. Defaults in reports expose IDs/hashes rather than source prose or actor names. A source link records a caller-supplied relative href plus a hashed back-matter resource, never a canonical processing path. No link/import is fetched.

Identity seeds use the existing FORGE_NAMESPACE_UUID and u64 big-endian length-prefixes:

- document: [forge.poam/1, plan_key, document, plan_key]
- item: [forge.poam/1, plan_key, item, item_key]
- milestone: [forge.poam/1, plan_key, milestone, item_key, milestone_key]
- plan party: [forge.poam/1, plan_key, party, party_key]
- event: [forge.poam-event/1, plan_key, item_key, sequence, payload_sha256, prior_event_sha256]

Mutable title/date/status/prose and directory location never enter document/item/milestone IDs. Events bind their immutable assertion payload. Imported source UUIDs stay exact. Terminal item versions reopen only through an explicitly linked new item key; no mutable new-version field silently changes identity. All collisions/duplicate keys fail.

## Proposed foundation bounds and output guarantees

| Limit | Proposed value |
|---|---:|
| Authored manifest bytes | 4 MiB, matching Assessment Results |
| Each source/context/baseline artifact | Existing 50 MiB |
| Aggregate held input bytes | 256 MiB across all files, checked before reads |
| Authored JSON depth / source JSON depth | 64 / 128 |
| Authored string bytes / imported string bytes | 64 KiB / 1 MiB |
| New authored identity key / imported stable key | 1 KiB / existing 64 KiB maximum |
| Findings or risks per source result | 10,000 each; total inventory 100,000 |
| Items / milestones globally | 0–10,000 / 10,000 for manifest; native emission requires 1–10,000 items |
| Milestones per item / references per item | 256 / 256 |
| History events globally | 10,000 |
| Parties / roles | 1,000 each |
| Graph edges / emitted findings | 100,000 / 10,000 |
| Schema errors | 100 |
| Serialized output / source-inventory report | 50 MiB / 10 MiB |

These are proposed constants, not measured capacity promises. Enforce aggregate expansion before allocation/cloning. Import each source object/party once regardless of the number of referring items. Reject output growth through a bounded writer; do not build an unlimited String and check afterward.

Confine inputs and output parents to the declared root; reject traversal, drive/UNC/ADS spellings, symlinks at any component, nonregular files, hard-link/input aliases, duplicate file identities and destinations colliding with transitive companions. Read/hash/parse the same held bytes, then revalidate captured inputs before publication. Init should refuse existing destinations.

Reuse the descriptor-confined no-replace publication pattern rather than relying solely on path checks followed by reopen. Its Linux/macOS support is explicit; fail closed on unsupported platforms until a qualified alternative exists. The shared integration owner should extract or wrap common primitives without changing the old authoring API/behavior.

src/io.rs::write_atomic makes one file atomic, not a multi-file artifact/report pair. It also documents that a durability error after rename can leave the new file published. Distinguish validation/prepublication failure (no output changed) from postpublication durability uncertainty; never claim every Err left bytes unchanged. For eventual artifact+report atomicity, publish one generation directory or document independent files with exact guarantees. Concurrent writers require the stated no-replace/serialization contract.

## Meaningful F07 tests to implement later

| Proposed executable case | Evidence it must establish |
|---|---|
| poam_schema_matches_exact_release_digest_and_compiles_offline | Exact receipt/allowlist; no external refs; unchanged prior schema bytes |
| empty_scaffold_contains_no_selected_items_or_assertions | items/parties/history empty; no owners/dates; no OSCAL empty artifact emitted |
| official_poam_rejects_zero_items | Native minimum is enforced rather than patched to allow the scaffold |
| source_identity_and_context_chain_are_exact | Wrong type/root/version/hash/import href, changed companion or mixed AP chain rejects before output |
| result_selection_is_explicit_and_epoch_bound | Missing/duplicate/wrong result key or UUID, including same object ID in another result, rejects |
| source_inventory_and_selection_are_complete | All finding/risk IDs remain visible; source states do not auto-select/auto-exclude; every nonempty item has explicit reviewed references |
| selection_rejects_wrong_kind_missing_uuid_or_changed_object | Tuple/hash validation; source/manifest bytes unchanged |
| source_provenance_properties_are_not_an_integrity_substitute | A claimed content hash cannot bypass recomputed object/file identity; duplicate/ambiguous namespace props reject |
| item_and_milestone_ids_ignore_order_prose_dates_and_paths | Actual reordered/mutated fixture comparison preserves identities, while a new key changes identity |
| typed_planned_risk_item_matches_official_schema | One real source risk, explicit owner/selection, one milestone, valid source projection and no closure assertion |
| finding_only_selection_is_not_rewritten_as_a_risk | Preserve the unresolved representation boundary; never fabricate a native risk |
| date_only_target_is_not_silently_made_midnight | Full-date round trip; native timing absent or explicitly supplied under approved mapping |
| closed_manifest_rejects_decoded_duplicates_unknowns_forward_versions | Top/nested strict decode before framework reads; bounded actionable paths |
| aggregate_limits_and_reference_fanout_stop_before_growth | Read/count/edge/serialization boundaries, not only oversized final-output checks |
| scaffold_publication_preserves_inputs_and_existing_destination | Symlink/hard-link aliases, special files, containment escapes, concurrent destination and interruption; complete-or-absent new file |
| inventory_bytes_are_directory_independent_and_content_minimizing | Byte comparison across separate roots; no absolute paths, prose, actor names or wall-clock additions |

These tests are proposed and were not run. The typed risk fixture is not a pilot or an independent-tool acceptance receipt.

## F08 scope remains intact

F07 establishes parts of M-1 through M-6, M-12, M-14/M-15 and foundational tests; it does not complete PRD064. F08 still owns all M-1–M-17 as an integrated workflow and all four Shoulds:

- Commands: complete build/check/baseline/report/explicit-as-of behavior (M-1, M-16).
- Owned remediation, graph/date/outcome validation, append-only transitions and evidence-bound terminal assertions (M-7–M-10).
- Schedule denominators and due/overdue boundaries (M-11).
- Complete typed native/extension projection including finding-only items and source graph semantics (M-12).
- All baseline impacts, changed source relationships, removals and explicitly reopened work (M-13).
- Safe offline non-mutation/privacy and complete transition/schema/determinism/adversarial tests (M-14–M-17).
- Explicit portfolio aggregation, static HTML timeline/trace, connector-neutral change sets and fresh linkage evidence references (S-1–S-4).
- Independent POA&M tool parsing/semantic validation, authentic remediation-team plans/pilots and actual owner dispositions with F21; exact 2.0.0 candidate documentation/provenance/platform evidence with F22.

DoR stays open for source-identity acceptance, Engineering/Compliance/Product decisions, three representative remediation teams and complete executable requirement coverage. This packet and the downloaded-in-memory schema do not satisfy those approvals. No participant contact, new dependency, schema vendoring, code mutation, Git/tracker change or Cargo run occurred.


Additional semantic cases for the official-schema boundary: reject a native
artifact missing both import-ssp and system-id even if structural schema
validation passes; preserve locally defined imported risk-status tokens or
return an explicit unsupported-subset error. Empty foundation items validate
only source/scaffold integrity, while native emission still requires full F08
validation and nonempty items. Incomplete document placeholders remain incomplete.
