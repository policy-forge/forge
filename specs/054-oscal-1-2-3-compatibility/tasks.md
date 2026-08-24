# Tasks: OSCAL v1.2.3 Compatibility

**Input**: `docs/PRD/054-prd-oscal-1-2-3-compatibility.md` and this feature's design artifacts

## Phase 1: Baseline Inventory and Contract Lock

- [x] T001 Create feature branch and spec/design artifacts.
- [x] T002 Add failing provenance manifest contract tests in `tests/schema_provenance_test.rs` (FR-001–FR-004, FR-012).
- [x] T003 Add immutable labeled v1.2.0 Catalog, Component, and Profile legacy fixtures (FR-006, FR-007, FR-011).
- [x] T004 Add failing validation-result contract and version-boundary tests (FR-006, FR-008).

## Phase 2: Schema and Metadata Baseline

- [x] T005 Vendor the nine pristine official v1.2.3 assets and `schemas/oscal-schema-manifest.json` (FR-001–FR-004).
- [x] T006 Implement offline digest, size, path, release identity, and remote-reference verification (FR-002, FR-003, FR-012).
- [x] T007 Update `OSCAL_VERSION` and generated-output expectations to `1.2.3` without rewriting legacy fixtures (FR-005, FR-011).
- [x] T008 Implement strict supported-version parsing and validation diagnostics (FR-006, FR-013).
- [x] T009 Extend validation report types/renderers additively and update CLI success/failure output (FR-008).
- [x] T010 Enforce the same supported-version policy before export while preserving supported input metadata (FR-006, FR-007).

## Phase 3: Model and Format Compatibility

- [x] T011 Add Catalog JSON/XML/YAML v1.2.3 gates and fix serializer incompatibilities (FR-010).
- [x] T012 Add Component Definition JSON/XML/YAML gates and fix serializer incompatibilities (FR-010).
- [x] T013 Add include/exclude/parameter Profile JSON/XML/YAML gates without export support (FR-010, FR-013).
- [x] T014 Add Assessment Plan JSON gate and correct existing serializer defects only (FR-010, FR-013).
- [x] T015 Add SSP JSON gate and correct existing serializer defects only (FR-010, FR-013).
- [x] T016 Refresh generated golden files/snapshots selectively and prove legacy fixtures unchanged (FR-005, FR-011).
- [x] T017 Run all existing internal format-pair equivalence tests (FR-007, FR-010).

## Phase 4: Ecosystem and Release Gate

- [x] T018 Extend `RoundTripResult` and renderers with declared/schema/tool baselines and classification (FR-009).
- [x] T019 Update oscal-cli tests for advisory older-model behavior and unavailable behavior (FR-009, FR-013).
- [x] T020 Document baseline, accepted-input policy, offline behavior, and future upgrade workflow (FR-012).
- [x] T021 Add explicit provenance verification to CI while retaining the cross-platform test owner (FR-003, FR-012).
- [x] T022 Run `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`, provenance verification, and supported optional gates (FR-013).
- [x] T023 Audit scope inventory to prove no Mapping, AP/SSP command, runtime download, or schema-selection flag was added (FR-013).

## Requirement Coverage

PRD M-1–M-4 → T002/T005/T006; M-5–M-6 → T007/T009; M-7–M-10 → T003/T004/T008–T010; M-11–M-18 → T011–T017; M-19–M-20 → T018/T019; M-21–M-22 → T006/T020/T021; M-23 → T022; M-24 → T023; M-25 → T020.
