# PRD 060 Implementation Audit

This audit maps the implementation on `codex/060-evidence-implementation-linking` to the live PRD.
It separates executable implementation evidence from launch approvals and design-partner outcomes.

## Must Have

| Requirement | Disposition | Implementation and executable evidence |
|---|---|---|
| M-1 Commands | Implemented | `forge linkage init`, `build`, `check`, and the S-1 `queue`; build/check require `--as-of`; reports support text/JSON/HTML. |
| M-2 Closed manifest | Implemented | `src/linkage/manifest.rs` uses duplicate-key-safe bounded parsing, `deny_unknown_fields`, a fixed `forge.linkage/1` version, and explicit collection/string/byte limits. |
| M-3 Artifact validation | Implemented | Catalog/Profile resources reuse the canonical Mapping inventory and validators; Profile companions are mandatory and hash-pinned; Component Definition and SSP inputs use pinned OSCAL v1.2.3 schemas; exact bytes and root metadata are recorded. |
| M-4 Inventories | Implemented | Recursive Catalog control/statement inventory plus Component Definition and SSP implemented-requirement/statement inventory; both sides enforce the canonical subject-count bound, and duplicate, missing, wrong-side/type, and ambiguous IDs are rejected. |
| M-5 Link cardinality | Implemented | Stable link keys are unique; both sides require bounded non-empty unique subject lists. |
| M-6 Review evidence | Implemented | Every link requires a known reviewer key, RFC 3339 review time, and non-empty rationale; `not-applicable` requires a separate reviewed assertion. |
| M-7 Local evidence | Implemented | Descendant-only roots/paths, descriptor-bound no-follow traversal, held-handle regular-file validation, bounded reads, and rejection of every hard-link alias and duplicate file identity. |
| M-8 URI evidence | Implemented | Absolute `https` or manifest-approved custom schemes only; no retrieval path exists; user-info/query/fragment are stripped before output. |
| M-9 Evidence metadata | Implemented | Stable key, title, type, owner, collection time, optional validity date, sensitivity/source labels, and local approved hash/size or URI-unverified declaration. |
| M-10 Fingerprints | Implemented | SHA-256 and byte size are computed from one bounded regular-file read and compared to approved values; changed output carries both approved and observed values. |
| M-11 Freshness | Implemented | Only explicit `--as-of`, `valid-through`, and configured window participate. Equality with `--as-of` is documented/tested as expired. |
| M-12 Missing evidence | Implemented | Links remain in the graph with an `evidence-missing` finding; exact implementation subjects outside every reviewed link produce `implementation-subject-unlinked`; no discovery or invented reference occurs. |
| M-13 Output | Implemented | Deterministic `forge.linkage-index/1`, `forge.linkage-report/1`, and stable reason-coded findings include provenance, inventories, graph, and freshness/gap data. |
| M-14 Baseline | Implemented | Distinct findings cover requirement/implementation subject add/remove/content change, evidence add/remove/content/reference/expiry change (including non-fetched URIs), link add/remove, and relationship membership edits. |
| M-15 Stable identity | Implemented | Link/finding UUID v5 inputs are length-delimited schema namespaces plus project/stable keys/reason codes; they exclude paths, order, time, and prose. |
| M-16 No assessment claims | Implemented | Metadata-only trust-boundary language is emitted; terminology guardrail tests scan the index. `implemented` remains explicitly labeled as a reviewer assertion. |
| M-17 Safety/privacy | Implemented | Offline operation, atomic writes, input/output and evidence alias checks (including queue destinations against transitive project inputs), bounded resources, terminal/HTML escaping, redacted references, and no content/absolute-path fields. |
| M-18 Exit contract | Implemented | `0` clean under selected gate, `1` valid selected maintenance findings, `2` invalid analysis. Invalid builds are tested not to create/overwrite outputs. |
| M-19 Tests | Implemented | `tests/linkage_cli_test.rs` and module tests cover Catalog/Profile, Component Definition/SSP, both subject classes, inventory/cardinality bounds, descriptor-level traversal/symlink/non-file/hard-link hazards, URI policy/redaction/gating, freshness boundaries, changed bytes and URI baselines, cross-directory determinism, terminology, queue, and HTML. |

## Should Have

| Requirement | Disposition | Evidence |
|---|---|---|
| S-1 Portfolio owner queue | Implemented | `forge linkage queue` analyzes only explicitly supplied manifests and groups stable findings by owner. |
| S-2 PRD 057/058 references | Implemented without approval transfer | Optional `impact_finding_ids` and `policy_version_keys` are validated, sorted, and retained as metadata; they never change review status. |
| S-3 OSCAL back-matter overlay | Correctly deferred | The PRD conditions this on demonstrated schema-valid lossless round trips for every supported model. That evidence and independent model approval do not exist, so no overlay is emitted. |
| S-4 Static HTML trace | Implemented | `--format html` renders escaped requirement-to-implementation links, evidence metadata/fingerprints, and maintenance findings. |

## Acceptance Criteria

| Criterion | Evidence |
|---|---|
| AC-1 exact hashes without bytes | `build_links_every_subject_type_without_copying_evidence` |
| AC-2 wrong-version/missing ID invalid, no output | `wrong_side_duplicate_and_missing_subjects_are_invalid_analysis`; exact resource hashes are mandatory. |
| AC-3 one-byte evidence change with old/new hashes | `changed_evidence_check_reports_provenance_and_both_hashes`; `changed_evidence_is_a_stable_action_finding_and_preserves_output` |
| AC-4 credential/query redaction | `uri_reports_are_redacted_and_never_fetched` |
| AC-5 expiry equality boundary | `freshness_boundaries_use_only_explicit_date` |
| AC-6 symlink/special-file rejection | `symlink_evidence_is_rejected_before_output_changes`; `hard_link_aliases_and_non_file_evidence_are_rejected` exercises the shared non-regular-file branch. |
| AC-7 directory-independent bytes/no absolute paths | `identical_projects_in_different_directories_are_byte_identical` |

## Security, Privacy, and Legal Requirements

- Evidence content confidentiality: content is hashed through a bounded buffer and discarded; only metadata is serialized.
- Filesystem hazards: descriptor-relative no-follow traversal prevents ancestor replacement between validation and use; the final held read is regular-file and size checked; any hard-link alias, duplicate file identity, and destination alias is rejected.
- URI/SSRF boundary: the code validates and redacts URIs but contains no network client or URI dereference operation.
- Hash interpretation: every output includes the metadata-only boundary; documentation states that SHA-256 is byte-change evidence, not origin/custody evidence.
- Link interpretation: no finding or schema field assigns a control outcome or evidence-quality judgment.
- Content/licensing boundary: reports carry identifiers and fingerprints, not framework/policy prose or evidence excerpts.

## Dependencies

- Canonical Catalog/Profile inventory, SHA-256, strict JSON, path alias, atomic output, deterministic report, and OSCAL validators are reused.
- The only shared-contract prerequisite is promoting the already vendored pinned SSP schema to runtime validation and adding `SystemSecurityPlan` detection. Existing export, diff, trace, and lifecycle matches explicitly retain their prior unsupported/metadata behavior.
- No crate dependency was added.

## Definition of Ready and External Completion Gates

The following PRD readiness/launch gates require external evidence and must not be inferred from a green build:

- Product approval of the native linkage-index MVP and assessment boundary.
- Compliance approval of implementation-status and freshness semantics.
- Engineering approval of subject classes, schema, hashing, limits, and baseline behavior.
- Security approval of file and URI trust boundaries.
- Legal/privacy review of metadata, label, path, and hash handling.
- Three sanitized design-partner workflows and the PRD's comprehension/time-savings trials.

Until those gates are explicitly closed, the production implementation may be code-complete and
verified but PRD 060 is not approved for launch and its design-partner success metrics are not met.
