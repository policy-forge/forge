# F02 implementation contract

Status: proposed mechanical implementation; D069 disposition pending. No audit,
exception owner or approval is created. This branch uses no new crate/library.
Root coordinates common formats, tests, policy documents, CI and Git.

## Commands and boundaries

Python 3.11+ stdlib tools: `scripts/dependency_inventory.py inventory|gate|validate-store`
require `--root` (default repository) and explicit `--as-of YYYY-MM-DD`;
`--format json|text` selects canonical stdout. All commands use committed inputs
only: no Cargo, subprocess, network or write. Exit 0 clean within approved
incremental policy; 1 review action (including proposed policy); 2 invalid input.
JSON prints a complete report even for action/invalid states. Text escapes controls.
`--strict` additionally requires every external runtime crate to have accepted
review or fresh owned exception, never legacy credit. Full F03 readiness remains open.

`scripts/dependency_graph.py refresh --root DIR --output FILE` is explicit,
uses locked offline Cargo metadata for every release target, and emits graph and
legacy baseline candidates. It is never invoked by inventory/gate. Offline refresh
needs cached manifests; offline consumption does not. Graph changes are reviewable.

## Shared primitives and closed sidecars

Common module: InputError; canonical_bytes(value) sorted compact ASCII JSON + LF;
identity(pkg) returns (name,version,source or "workspace"); package_id(pkg) is compact
JSON of that three-string list (no path/clock). Checksum is separately pinned.
read_bytes(root, relative, limit=16MiB) rejects traversal, links, nonregular files,
aliases and bounds before bounded reads; load_json rejects duplicate keys, NaN,
unknown authored fields and nesting >64; load_toml uses stdlib tomllib. sha256 bytes.

Every sidecar has `format` exact string and closed keys.

Graph `forge.dependency-graph/1`: inputs {lock_sha256, manifests:{relative:sha256},
release_workflow_sha256}; targets sorted triples; features ["default"];
packages sorted by identity, each {name,version,source,checksum,workspace_member,
proc_macro}; graphs array, each {target,roots:[package_id],edges:[{from,to,kind}]}
where kind normal|build|dev. Every locked identity appears exactly once. Graph
source/checksum must agree with lock; every edge/root resolves; target list must
agree with committed release workflow matrix. Compute runtime from workspace
normal roots along normal edges and stop proc_macro nodes. Runtime wins over
other usage; build/proc-macro/dev and unsupported-target-only remain distinct.
Class is safe-to-deploy for runtime, safe-to-run otherwise. Preserve all locked
identities. Report graph closure as metadata-derived, not final artifact proof.

Legacy `forge.dependency-legacy/1`: inputs {lock_sha256,config_sha256}; packages
[{name,version,source,checksum,criteria:[...]}] exact frozen locked exemption rows.
It grants no review credit. Only deploy-marked legacy identities may be tolerated
by approved incremental gate; changed source/version/checksum is new. Bind this
file's exact digest in policy; do not refresh the baseline automatically with graph.

Policy `forge.dependency-policy/1`: status proposed|approved; audit_authority
human-signoff|human-only; max_exception_days positive integer; imports array
[{name,url,snapshot_sha256}], legacy_baseline_sha256; optional decision
{owner,recorded_on,reference}, required if approved. Proposed has no decision.
Do not populate real decisions. Approved source name/URL must agree with native
config, and exact imports.lock bytes must match each source snapshot_sha256.
Policy-approved imports may prove only crates.io registry identities; another
source needs a specific versioned design. Imported proof starts at a full audit;
deltas extend only from proven versions with deploy implying run. Cycles and
orphan paths never prove review. Pending/unapproved imports remain visible.

Exceptions `forge.dependency-exceptions/1`: entries [{name,version,source,checksum,
criteria:[...],owner,rationale,created_on,review_by,expires_on,approval_reference}].
All identity/date/metadata fields required and bounded; criteria built-in only;
created<=review<=expiry; term<=policy.max_exception_days. Every entry binds a
current locked identity/checksum. No duplicate/retired/unknown record silently
counts. Age uses explicit as-of; past review date and expiry are action (1).
An exemption missing metadata is legacy-unowned, not a fake exception.

Reviews `forge.dependency-reviews/1`: records [{crate,source,record_sha256,method,
checksums:{version:sha256},signoff:{owner,recorded_on,reference}}]. method human or
agent-assisted must satisfy approved audit authority. record_sha256 binds canonical
native audits.toml audit object including full version or exact delta pair. Check
checksum for locked target and exact version/source. Unbound native audits are
pending reviews. A signed-off orphan delta is invalid proof (2), not accepted.
A full reviewed base is necessary; exemptions cannot seed a differential chain.
All real records remain empty in this proposed branch. A committed review record
is an asserted signoff, not identity authentication or safety certification.

Native Cargo files remain untouched. Validate TOML/structural audit records,
criteria and version/delta ambiguity. Inventory names exact identities, class,
status (first-party, local-full/local-delta/imported-full/imported-delta,
owned-exception, stale-exception, legacy-unowned, pending-review, unreviewed),
proof chain/auditor or owner/age, source/checksum, current and retired exemptions,
new unvetted runtime identities, pending policy and strict runtime denominator.
Trend counts have exact class/status denominators and exception-age median.

## Required checks

Black-box stdlib unit/CLI fixtures: fresh empty Cargo cache and denied subprocess;
identical bytes across order/root changes; every locked identity once; runtime vs
build/proc-macro/dev and dual-use; stale graph/hash/manifest/workflow; new runtime
name/version/source/checksum; only-run proof cannot deploy; full/delta/import chains,
exemption base/orphan/cycle; import pin/config changes; malformed/dates/expired
exceptions and boundary dates; pending policy cannot exit clean; immutable baseline
pin and no changed-version grandfathering; duplicate/unknown/special/oversized inputs.
Synthetic fixture signoffs never become production audit evidence. Add a dedicated
CI inventory job that publishes report even on nonzero outcome and never converts
proposed policy or current gaps into green acceptance.

## Frozen review clarifications

Sidecar paths are `supply-chain/dependency-graph.json`,
`legacy-baseline.json`, `dependency-policy.json`, `dependency-exceptions.json`,
and `dependency-reviews.json`. Shared module is `scripts/dependency_common.py`;
`load_json(bytes)` and `load_toml(bytes)` decode already bounded reads.
`release_targets(root)` and `read_release_targets(bytes)` share the narrow literal
release matrix parser; an unsupported workflow shape fails closed.

`validate-store` reports unowned native exemptions as invalid (exit 2, PRD-069
AC-3); approved incremental `inventory`/`gate` can tolerate only pinned legacy
identities. Future exception creation, policy decision or review signoff dates
are invalid. A local delta pins both old/new version checksums and matches the
previous proof endpoint, including versions absent from the current lock.

A null source identifies path origin but does not prove first-party membership.
Only manifest-verified workspace members receive that status. External path/Git
reviews remain unsupported. Native criteria maps, excludes, violations, custom
criteria, wildcard/trusted audits and dependency overrides are rejected until
their semantics are implemented, rather than silently ignored.

Build/proc-macro execution can emit deployed code. The PRD's runtime/build
classification needs an owner disposition against cargo-vet's deployment
criterion; the metadata runtime denominator is not total supply-chain assurance.
This generator-policy and D069 acceptance remain open.

## Integration qualification and bounds

Workspace membership and shipping roots differ: derive roots from explicit
`workspace.default-members`, or the nonvirtual root package by default, or all
members for a virtual workspace. The inventory checks the same rule as Cargo.
Native exemptions with valid exact current sidecar exception metadata can remain
for cargo-vet compatibility; only uncovered/retired/ambiguous native entries are
`legacy-unowned` and invalid for `validate-store`.

Offline graph input hashes do not prove derived edge/proc-macro authenticity.
The independent `dependency_graph.py check` command resolves locked offline
metadata and compares the complete canonical sidecar without writing. CI runs
that qualification separately from the inventory gate after explicit locked
manifest preparation; both checks must pass. Qualification 0, drift 1, invalid 2.

Inputs are at most 16 MiB with nesting 64. Authored package/audit/review/exception
collections are bounded at 5,000; a proof graph is at most 1,000 records per
crate/source and a path at most 256 records. A target graph permits at most 100,000
edges. Over-bound inputs are invalid rather than receiving partial review credit.
