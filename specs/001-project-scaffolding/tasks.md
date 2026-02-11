# Tasks: Project Scaffolding

**Input**: Design documents from `specs/001-project-scaffolding/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/cli-interface.md

**Tests**: Included — constitution principle IV mandates TDD for all production code.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Initialize the Rust project with dependencies and basic configuration.

- [ ] T001 Initialize Cargo project with `cargo init --name forge` producing `Cargo.toml` and `src/main.rs`
- [ ] T002 Add dependencies to `Cargo.toml`: `clap` (4.x, features = ["derive"]) and `thiserror` (latest stable) using `cargo add`
- [ ] T003 [P] Create `.rustfmt.toml` with project formatting configuration (edition = "2021", max_width = 100, use_small_heuristics = "Max", imports_granularity = "Module", group_imports = "StdExternalCrate")
- [ ] T004 [P] Configure clippy lints in `Cargo.toml` under `[lints.clippy]`: all = warn, pedantic = warn; and `[lints.rust]`: unsafe_code = warn

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types and module stubs that MUST be complete before CLI implementation.

**Corresponds to**: US3 (Error Handling, P2) and US4 (Module Structure, P2) — foundational by nature despite being P2, as US1 depends on them.

### Tests for Foundational Phase

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T005 [P] Write unit tests for ForgeError Display output in `src/error.rs` — test that each variant (Io, Parse, Validation, Config) produces the expected display message per data-model.md
- [ ] T006 [P] Write unit tests for ForgeError From conversion in `src/error.rs` — test that `std::io::Error` converts to `ForgeError::Io` via the `?` operator

### Implementation for Foundational Phase

- [ ] T007 Create `src/error.rs` with `ForgeError` enum: variants `Io(#[from] std::io::Error)`, `Parse(String)`, `Validation(String)`, `Config(String)` with `#[derive(Debug, thiserror::Error)]` and `#[error("...")]` attributes per data-model.md
- [ ] T008 Create `src/lib.rs` as library root — declare all modules (`pub mod cli`, `pub mod error`, `pub mod ingest`, `pub mod parse`, `pub mod model`, `pub mod oscal`, `pub mod validate`, `pub mod export`) and re-export `ForgeError` from `error` module
- [ ] T009 [P] Create `src/ingest/mod.rs` as empty module stub
- [ ] T010 [P] Create `src/parse/mod.rs` as empty module stub
- [ ] T011 [P] Create `src/model/mod.rs` as empty module stub
- [ ] T012 [P] Create `src/oscal/mod.rs` as empty module stub
- [ ] T013 [P] Create `src/validate/mod.rs` as empty module stub (note: this is the schema validation module, not the CLI validate subcommand)
- [ ] T014 [P] Create `src/export/mod.rs` as empty module stub
- [ ] T015 Verify foundational phase: run `cargo build` and `cargo test` — all module stubs compile, ForgeError tests pass

**Checkpoint**: ForgeError enum works with tested Display messages and From conversion. All 7 pipeline-stage modules exist and compile. Library root re-exports key types.

---

## Phase 3: User Story 1 — Run FORGE CLI and View Help (Priority: P1) MVP

**Goal**: A developer can run `forge --help` and see usage text with `convert` and `validate` subcommands listed. Subcommands accept arguments but print "not yet implemented".

**Independent Test**: Build the project and run `cargo run -- --help`. Verify output shows `convert` and `validate` subcommands with descriptions.

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T016 [P] [US1] Write unit test for CLI parsing in `src/cli/mod.rs` — test that `Cli::try_parse_from(["forge", "convert", "test.md"])` succeeds and produces `Commands::Convert` with correct input path
- [ ] T017 [P] [US1] Write unit test for CLI parsing in `src/cli/mod.rs` — test that `Cli::try_parse_from(["forge", "validate", "artifact.json"])` succeeds and produces `Commands::Validate` with correct input path
- [ ] T018 [P] [US1] Write unit test for CLI parsing in `src/cli/mod.rs` — test that `Cli::try_parse_from(["forge", "convert", "test.md", "--strategy", "catalog", "--format", "json", "--output", "out.json"])` succeeds with all optional args populated
- [ ] T019 [P] [US1] Write unit test for CLI parsing in `src/cli/mod.rs` — test that `--verbose` and `--quiet` flags conflict (parsing fails when both provided)
- [ ] T020 [US1] Write integration test in `tests/cli_integration.rs` — test that running the binary with `--help` produces output containing "convert" and "validate" subcommand names

### Implementation for User Story 1

- [ ] T021 [US1] Create `src/cli/mod.rs` with `Cli` struct (`#[derive(Parser)]`, `#[command(name = "forge", about = "FORGE — Framework for OSCAL Risk & Governance Execution", version)]`) containing `command: Commands`, `verbose: bool` (`#[arg(short, long, conflicts_with = "quiet")]`), `quiet: bool` (`#[arg(short, long)]`) per data-model.md and contracts/cli-interface.md
- [ ] T022 [US1] Define `Commands` enum (`#[derive(Subcommand)]`) in `src/cli/mod.rs` with `Convert` variant (fields: `input: PathBuf`, `strategy: Option<Strategy>`, `format: OutputFormat` with default `Json`, `output: Option<PathBuf>`) and `Validate` variant (field: `input: PathBuf`) per data-model.md
- [ ] T023 [P] [US1] Define `Strategy` enum (`#[derive(ValueEnum, Clone, Debug)]`) with variants `Catalog` and `Component` in `src/cli/mod.rs` per data-model.md
- [ ] T024 [P] [US1] Define `OutputFormat` enum (`#[derive(ValueEnum, Clone, Debug)]`) with variants `Json`, `Xml`, `Yaml` in `src/cli/mod.rs` per data-model.md
- [ ] T025 [US1] Create `src/cli/convert.rs` with stub handler function `pub fn execute(...) -> Result<(), ForgeError>` that prints "Convert command not yet implemented" and returns `Ok(())` per contracts/cli-interface.md
- [ ] T026 [P] [US1] Create `src/cli/validate.rs` with stub handler function `pub fn execute(...) -> Result<(), ForgeError>` that prints "Validate command not yet implemented" and returns `Ok(())` per contracts/cli-interface.md
- [ ] T027 [US1] Update `src/main.rs` to parse CLI args via `Cli::parse()`, match on `Commands` variants, and dispatch to convert/validate handlers. Map `ForgeError` to process exit code 1. Print help when no subcommand is provided (use `#[command(arg_required_else_help = true)]` or equivalent)
- [ ] T028 [US1] Verify US1: run `cargo run -- --help`, `cargo run -- convert --help`, `cargo run -- --version`, and `cargo run -- convert test.md` — all produce expected output per contracts/cli-interface.md. Run `cargo test` — all unit and integration tests pass

**Checkpoint**: `forge --help` displays usage text with convert and validate subcommands. Subcommands accept arguments and print stub messages. All CLI parsing tests pass.

---

## Phase 4: User Story 2 — CI Quality Gates Enforce Standards (Priority: P1)

**Goal**: A CI pipeline configuration enforces `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` on every push.

**Independent Test**: Run `cargo fmt --check && cargo clippy -- -D warnings && cargo test` locally — all pass with zero violations.

### Implementation for User Story 2

- [ ] T029 [US2] Create `.github/workflows/ci.yml` with GitHub Actions workflow: trigger on push and pull_request, use `actions/checkout`, install Rust stable toolchain, run `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` as sequential steps
- [ ] T030 [US2] Verify US2: run `cargo fmt --check && cargo clippy -- -D warnings && cargo test` locally — all three gates pass with zero violations and zero warnings

**Checkpoint**: CI pipeline configuration exists. All quality gates pass locally. Ready for GitHub Actions to execute on push.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Final validation across all user stories.

- [ ] T031 Run full quality gate suite: `cargo fmt --check && cargo clippy -- -D warnings && cargo test` — zero violations
- [ ] T032 Run quickstart.md validation: follow all steps in `specs/001-project-scaffolding/quickstart.md` and verify each produces expected output
- [ ] T033 Verify all acceptance scenarios from spec.md: `forge --help` shows subcommands (US1-AS1), `forge` with no args shows help (US1-AS2), `forge convert` without args shows error (US1-AS3), fmt/clippy/test pass (US2-AS1/2/3), ForgeError Display messages are descriptive (US3-AS1), errors propagate with `?` (US3-AS2), all 7 modules exist and compile (US4-AS1/2)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **US1 CLI (Phase 3)**: Depends on Foundational (needs ForgeError and module declarations in lib.rs)
- **US2 CI (Phase 4)**: Depends on US1 (CI must have passing code to validate against)
- **Polish (Phase 5)**: Depends on US1 and US2 completion

### User Story Dependencies

- **US3 (Error Handling)** and **US4 (Module Structure)**: Promoted to Foundational phase — they are prerequisites for US1
- **US1 (CLI)**: Depends on Foundational phase. Cannot start until ForgeError and modules exist
- **US2 (CI)**: Depends on US1. CI configuration needs a compilable, testable project to validate

### Within Each Phase

- Tests MUST be written and FAIL before implementation (TDD per constitution principle IV)
- Type definitions (enums, structs) before handler functions
- Handlers before main.rs dispatch wiring
- All tests green before checkpoint

### Parallel Opportunities

- T003 and T004 (Setup): `.rustfmt.toml` and clippy config are independent files
- T005 and T006 (Foundational tests): test different ForgeError capabilities, independent
- T009–T014 (Module stubs): all 6 pipeline modules are independent files, all parallelizable
- T016–T019 (US1 tests): all test different CLI parsing scenarios, independent
- T023 and T024 (US1 value enums): Strategy and OutputFormat are independent types
- T025 and T026 (US1 stub handlers): convert.rs and validate.rs are independent files

---

## Parallel Example: Foundational Phase

```
# Launch all module stubs in parallel (T009–T014):
Task: "Create src/ingest/mod.rs as empty module stub"
Task: "Create src/parse/mod.rs as empty module stub"
Task: "Create src/model/mod.rs as empty module stub"
Task: "Create src/oscal/mod.rs as empty module stub"
Task: "Create src/validate/mod.rs as empty module stub"
Task: "Create src/export/mod.rs as empty module stub"
```

## Parallel Example: User Story 1

```
# Launch all CLI parsing tests in parallel (T016–T019):
Task: "Write unit test for convert subcommand parsing"
Task: "Write unit test for validate subcommand parsing"
Task: "Write unit test for convert with all optional args"
Task: "Write unit test for verbose/quiet conflict"

# Launch value enum definitions in parallel (T023–T024):
Task: "Define Strategy enum in src/cli/mod.rs"
Task: "Define OutputFormat enum in src/cli/mod.rs"

# Launch stub handlers in parallel (T025–T026):
Task: "Create convert stub handler in src/cli/convert.rs"
Task: "Create validate stub handler in src/cli/validate.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T004)
2. Complete Phase 2: Foundational (T005–T015) — ForgeError + module stubs
3. Complete Phase 3: User Story 1 (T016–T028) — CLI with help and subcommand stubs
4. **STOP and VALIDATE**: Run `forge --help`, verify output, run all tests
5. At this point FORGE has a functional CLI entry point — MVP achieved

### Incremental Delivery

1. Setup + Foundational → Project compiles, error types work, modules exist
2. Add US1 (CLI) → `forge --help` works, subcommand stubs respond → MVP!
3. Add US2 (CI) → Quality gates enforced on every push → Production-ready workflow
4. Polish → Final acceptance scenario validation

### Single Developer Strategy

Since this is a solo developer project (1 engineer, sprint S-1):
1. Execute phases sequentially: Setup → Foundational → US1 → US2 → Polish
2. Within each phase, batch parallelizable tasks (e.g., create all module stubs at once)
3. TDD cycle per task: write test → verify failure → implement → verify pass
4. Commit after each completed phase

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- US3 and US4 are promoted to Foundational phase because US1 depends on them
- Constitution principle IV: TDD is mandatory — all tests written before implementation
- No `#[allow(dead_code)]` on module stubs (AR anti-pattern guidance)
- Total tasks: 33
