# F07 implementation contract

Status: proposed mechanical foundation, pending D064 and PRD 063 acceptance.
Base: origin/main aef0ab24f77559593b6d0b4fe5027f8a5eaa857e.

F07 validates the exact local Assessment Results result and companions and
creates an unselected Forge manifest. It never emits an empty native POA&M,
selects work, assigns owners/dates, records completion, or treats foundation
checks as complete workflow validation. F08 owns workflow and output semantics.
No new crate dependencies. Offline operation; existing schema bytes preserved.

## Frozen integration interfaces

`poam::manifest::PoamManifest` has schema_version (forge.poam/1), document
(existing DocumentManifest), source (SourceManifest), roles and parties
(existing closed AR role/party shapes), and items (Vec<ItemManifest>).
SourceManifest has assessment_results (existing ArtifactManifest), result
(ResultIdentity { uuid, key }), and context (existing ContextManifest).
ItemManifest has key, source_refs (Vec<SourceReference>), and milestones
(Vec<MilestoneIdentity { key }>). SourceReference has kind (SourceKind:
finding/risk), key, uuid, result_uuid, and expected_sha256 (computed canonical
object digest). All authored records deny unknown fields. F07 parse rejects
nonempty items/roles/parties with an explicit unsupported workflow error. This
is a draft foundation contract, not acceptance of an incomplete remediation
item. Evidence-index contexts are explicitly unsupported in this foundation.
Document metadata is explicit; no clock or actor defaults. Zero selected items
are permitted only for a scaffold and source-only check.

`manifest::parse(&[u8]) -> Result<PoamManifest, ForgeError>` is bounded,
duplicate-safe, closed, validates all source identities and metadata syntax.
`identity::{document(plan_key), item(plan_key,item_key), milestone(plan_key,
item_key,milestone_key)} -> Uuid` uses UUID v5 and length-prefixed segments;
mutable prose, order, owners and dates never enter seeds.

`source::load(&Path, &SourceManifest) -> Result<PreparedSource, ForgeError>`
captures five confined regular single-link inputs, validates exact bytes and
AR/AP/SSP/Profile/Catalog imports, UUIDs/versions/hashes, and explicit result
UUID plus stable key. No first-result default. The supported AR producer profile declares OSCAL 1.2.3;
companions preserve exact-pinned input compatibility 1.2.0–1.2.3. It rejects stale context pins,
duplicate stable keys/UUIDs, unsupported source format and wrong selections.
`source::validate_selection(&PreparedSource, &[SourceReference]) -> Result<(),
ForgeError>` compares the full result/kind/key/UUID/computed object digest tuple.
`PreparedSource::inventory() -> &SourceInventory` and `verify_inputs() ->
Result<(), ForgeError>` expose content-minimizing rows and identity/byte
revalidation. Every finding and risk is listed, including satisfied/closed
objects. Native schema-valid extensions are not silently projected into output.

`report::SourceInventory` is Serialize and has schema_version
(forge.poam-source-inventory/1), source_sha256, result_uuid, result_key, and
validation_scope (source-integrity-only), workflow_validated (false), and
objects (Vec<SourceObject>). SourceObject has kind, key, uuid, result_uuid,
sha256, state, control_ids, and optional declared_content_sha256 /
declared_rationale_sha256. Inventories omit titles, descriptions,
assessment/risk rationale and actor fields, and add no resolved processing paths.
Authored stable keys and control IDs remain unchanged; they may themselves
contain sensitive content. They assert no
review or remediation eligibility judgments. Objects are sorted by kind/key.

Root supplies `assessment_results::context::load_captured(&ContextManifest,
&BTreeMap<PathBuf, Vec<u8>>) -> Result<LoadedContext, ForgeError>`. It validates
a private snapshot of the exact captured bytes through existing context rules;
it never reopens original inputs. Root also exposes existing manifest artifact
and context validators as pub(crate), maps PoamBuild errors to exit 2, and owns
module registration, CLI, all shared enum matches, and output publication.

CLI foundation: `forge poam init` requires --assessment-results,
--assessment-plan, --ssp, --profile, --catalog, --result-uuid, --result-key,
--document-key, --title, --document-version, --last-modified, and optional
--output. Inputs are normalized descendants of explicit --root (default current
directory); output is a new .json filename directly in that root and never overwritten.
`forge poam check --manifest FILE --source-only` verifies foundation/source
integrity and returns a deterministic text or JSON source inventory. No build
command claims incomplete workflow support. Source-only flag is required.

## Ownership

Coordinator: shared files, CLI/errors/model registration, captured context
adapter, poam/mod.rs, integration tests, documentation and all Git/Cargo.
Manifest lane: src/poam/manifest.rs and identity.rs with unit tests.
Source lane: src/poam/source.rs and report.rs with unit tests.
Schema lane: exact official schema asset, provenance manifest and provenance
test, preserving all previous bytes. No shared registration edits.

## Acceptance still open

D064 editable authority, supported projection, terminal assertion evidence;
PRD 063 source acceptance; date/milestone mapping; full F08 workflows; Windows
publication qualification; independent tool interoperability and authentic
remediation pilots remain distinct gates. A passed F07 source check is only
source integrity, and never approves a remediation plan or source review.

Integration deltas: the supported AR profile requires OSCAL 1.2.3 and all four unique native back-matter context receipts. Companion declarations retain exact 1.2.0–1.2.3 compatibility. Detached reports carry explicit source-only scope and workflow_validated=false. Authored manifest arrays remain unselected until F08.

## Final roadmap documentation gate

After the completed roadmap work is integrated, perform a full documentation
review and update as requested by the user on 2026-10-02. Reconcile CLI/API
reference, examples, architecture, schema/dependency provenance, supported
platforms, verification evidence and requirement/acceptance status against the
final implementation. Keep missing owner or participant evidence explicit.
This final review does not replace the documentation checks for each PR.
