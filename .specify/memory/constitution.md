# forge Constitution

## Core Principles

### I. Modular Architecture (NON-NEGOTIABLE)

Features MUST be organized into well-defined, independently testable modules with clear boundaries. The appropriate granularity depends on project scope:

- **Single-purpose CLI tools**: Dedicated submodules within a single crate (one feature = one submodule with explicit `pub` API)
- **Multi-service systems**: Standalone crates within a Cargo workspace (one service/domain = one crate)

Modules/crates must be:

- **Self-contained**: Minimal coupling between modules (use trait abstractions and clear API boundaries)
- **Independently testable**: Full test coverage without requiring the entire project
- **Clearly purposed**: Single responsibility with explicit public API boundaries
- **API-documented**: All public items documented with `rustdoc` comments

**Prohibited**: God-modules/god-crates, kitchen-sink utility modules, or modules that exist solely to group unrelated code. Every module must provide concrete, reusable functionality.

**Single-Crate Architecture** (focused CLI tools like FORGE):

```text
project/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry point (thin wrapper)
│   ├── lib.rs               # Library root — re-exports public API
│   ├── error.rs             # Error types (thiserror)
│   ├── model/               # Domain model types
│   │   └── mod.rs
│   ├── parse/               # Parsing and structural extraction
│   │   ├── mod.rs           # Module root with re-exports
│   │   ├── clauses.rs       # Clause extraction (WI-4)
│   │   └── atomize.rs       # Requirement atomization (WI-6)
│   ├── export/              # Output generation
│   └── validate/            # Validation logic
```

**Workspace Architecture** (larger systems with multiple binaries or services):

```text
workspace/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── core/               # Shared types, traits, error types
│   ├── scanner/            # Core scanning/analysis engine
│   ├── reporting/          # Output formatting and report generation
│   └── cli/                # CLI binary entry point
```

**Binary vs Library**: Binary entry points (`main.rs`) are thin wrappers that compose library modules/crates. Business logic lives in library code. Binaries handle startup, configuration wiring, and signal handling only.

**Public API Design**:
- Use `pub(crate)` by default; only `pub` items that are part of the module's contract
- Re-export key types from `lib.rs` and module `mod.rs` for ergonomic imports
- Use feature flags for optional functionality, not separate modules for small variations

**Migration Path**: A single-crate project MAY be refactored into a workspace when complexity warrants it (e.g., multiple binary targets, shared library extraction). This is a future optimization, not a prerequisite.

**Rationale**: Modular architecture enables independent testing and clear separation of concerns. For focused CLI tools, submodules within a single crate provide sufficient isolation without the overhead of workspace management. Rust's module system enforces visibility boundaries at compile time.

### II. Rust-First with Strategic FFI Integration

This project uses Rust as the sole primary language:

- **Rust** MUST be used for all application logic, CLI, API, and worker processes
- **FFI boundaries** (if needed) MUST be:
  - Wrapped in safe Rust abstractions
  - Documented with safety invariants
  - Tested with both unit tests and integration tests
  - Isolated in dedicated `-sys` or `-ffi` crates
- **Build scripts** (`build.rs`) MUST be minimal and well-documented
- **Unsafe code** MUST be:
  - Justified with a `// SAFETY:` comment explaining why the invariant holds
  - Isolated behind safe public APIs
  - Minimized — prefer safe alternatives even at modest performance cost
  - Audited via `cargo geiger` in CI

**Decision Framework**:
1. **Need C library integration?** → Create a `-sys` crate with safe wrapper
2. **Performance-critical hot path?** → Profile first, use unsafe only with measured justification
3. **When in doubt**: Safe Rust. The compiler is your ally.

**Rationale**: Rust's ownership model and type system eliminate entire classes of bugs at compile time. FFI boundaries are the primary source of soundness risk and must be treated with extreme care.

### III. Contract-First Development (NON-NEGOTIABLE)

API contracts, trait definitions, and type schemas MUST be defined before implementation:

- **Trait Definitions**: Define behavior contracts as traits before implementing them
- **Type Definitions**: Define request/response types, error types, and domain models before business logic
- **API Schemas**: Define HTTP endpoints using OpenAPI via `utoipa` decorators before handler implementation
- **Database Schemas**: Define migrations (via `sqlx` or `diesel`) before data access code
- **CLI Interface**: Define `clap` argument structures before command handlers
- **Error Types**: Define error enums with `thiserror` before writing fallible functions

**Contract Review Process**:
1. Define traits, types, and error enums in the crate's `lib.rs` or `types.rs`
2. Add `rustdoc` documentation with examples for all public items
3. Review and approve contract before implementation
4. Implement using the defined types
5. Integration tests verify contract compliance

**Rationale**: Rust's type system is powerful enough to encode contracts at the type level. Defining types first leverages the compiler as a contract enforcement mechanism.

### IV. Test-First Development (NON-NEGOTIABLE)

TDD is mandatory for all production code:

- Tests MUST be written before implementation
- Tests MUST fail before implementation begins (`cargo test` shows red)
- Red-Green-Refactor cycle MUST be followed strictly
- Pull requests without tests for new functionality MUST be rejected
- Tests MUST be automated and run in CI/CD pipeline
- Minimum coverage targets: 80% line coverage (measured via `cargo-tarpaulin` or `cargo-llvm-cov`)

**TDD Cycle**:
1. **Write Test**: Create test demonstrating desired behavior
2. **User Review**: Test reviewed and approved BEFORE implementation
3. **Verify Failure**: Test must FAIL with clear error message
4. **Implement**: Write minimal code to pass test
5. **Verify Pass**: Test must PASS
6. **Refactor**: Improve code while maintaining passing tests

**Testing Stack**:
- **Unit Tests**: In-module `#[cfg(test)]` blocks
- **Integration Tests**: `tests/` directory at crate root
- **Doc Tests**: `rustdoc` examples that compile and run
- **Property Tests**: `proptest` or `quickcheck` for invariant validation
- **Snapshot Tests**: `insta` for output-sensitive testing (reports, CLI output)
- **Benchmark Tests**: `criterion` for performance regression detection

**Test Organization**:
```rust
// Unit tests — same file as implementation
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_detects_known_vulnerability() {
        // Arrange, Act, Assert
    }

    #[tokio::test]
    async fn async_operation_completes() {
        // Async test with tokio runtime
    }
}
```

```rust
// Integration tests — crate-level tests/ directory
// tests/integration_scan.rs
use my_crate::Scanner;

#[test]
fn end_to_end_scan_produces_report() {
    // Tests the public API only
}
```

**Rationale**: TDD ensures code correctness and serves as living documentation. Rust's compiler catches many bugs, but logic errors still need tests. Security-critical code demands rigorous verification.

### V. Complete Implementation (NON-NEGOTIABLE)

All tasks in `tasks.md` MUST be completed before feature branch merges:

**Binary Completion**: Tasks are either COMPLETE (✅ passing tests, documented, reviewed) or INCOMPLETE (❌). No "mostly done" or "good enough" states.

**Verification Checklist** (per task):
- [ ] All tests written and passing (`cargo test --workspace`)
- [ ] Code reviewed and approved
- [ ] `rustdoc` documentation complete for public API
- [ ] Performance benchmarks meet requirements (if applicable)
- [ ] Security review completed (no `unsafe` without justification, no secrets)
- [ ] Database migrations created and tested (up and down)
- [ ] Breaking changes documented in CHANGELOG.md
- [ ] `cargo clippy -- -D warnings` passes with zero warnings

**No Partial Merges**: Feature branches merge only when ALL tasks complete. Use WIP commits on branch, but squash before merge.

**Rationale**: Partial implementations create technical debt and maintenance burden. Complete implementation ensures every merged feature is production-ready.

### VI. Performance-First Design (NON-NEGOTIABLE)

Performance is a feature requirement, not an optimization:

**Baseline Requirements** (adapt to your project):
- CLI scan completion: < X seconds for typical project
- API endpoint p95 latency: < 100ms (non-compute endpoints)
- Memory usage: Peak RSS < X MB for standard workloads
- Startup time: < 500ms to first useful output
- Concurrent scans: Support N parallel operations without degradation

**Mandatory Benchmarks**:
- All hot paths benchmarked with `criterion`
- Performance regression tests in CI (compare against baseline)
- Memory profiling with `dhat` or `heaptrack` for allocation-heavy code
- Flamegraph generation for optimization targets (`cargo-flamegraph`)
- Benchmarks documented in `benches/` directory

**Optimization Strategy**: Profile first (`perf`, `flamegraph`), optimize hot paths, document tradeoffs. No premature optimization, but architect for performance from the start. Prefer zero-copy parsing, arena allocation for tree structures, and streaming processing over buffering.

**Rust-Specific Performance Practices**:
- Prefer `&str` over `String` in function parameters
- Use `Cow<'_, str>` when ownership is conditional
- Prefer iterators over collecting into intermediate `Vec`s
- Use `SmallVec` or `ArrayVec` for small, bounded collections
- Avoid unnecessary `clone()` — if you're cloning, justify it
- Use `#[inline]` sparingly and only with benchmark evidence

**Rationale**: Rust is chosen partly for performance. Establishing performance requirements upfront prevents costly rewrites and ensures we leverage the language's strengths.

### VII. Security-First Design (NON-NEGOTIABLE)

Security is foundational, not bolted-on:

**Memory Safety**:
- `unsafe` code MUST have a `// SAFETY:` comment explaining invariants
- `unsafe` blocks MUST be minimized and isolated behind safe APIs
- `cargo geiger` MUST be run in CI to track unsafe usage
- All FFI boundaries MUST validate inputs before passing to foreign code

**Authentication & Authorization** (if API-facing):
- Authentication MUST be required for all endpoints
- Rate limiting enforced via middleware (e.g., `tower::limit`)
- Audit logging for all data access operations

**Input Validation**:
- All external input MUST be validated before processing
- Use strong types (newtypes) to prevent primitive obsession
- File paths MUST be canonicalized and checked against allowed directories
- Deserialization MUST use `serde` with `#[serde(deny_unknown_fields)]` where appropriate
- Size limits MUST be enforced on all user-provided input

**Secrets Management**:
- Secrets MUST never be committed to code (use environment variables or secret stores)
- API keys/credentials NEVER logged or exposed in errors
- Use `secrecy` crate for in-memory secret handling (zeroize on drop)
- `.env.example` used as template, actual `.env` in `.gitignore`

**Dependency Security**:
- `cargo audit` MUST be run in CI — block on any advisory
- `cargo deny` MUST enforce license and vulnerability policies
- Supply chain attacks mitigated via `cargo-vet` or `cargo-crev`
- Dependency tree reviewed for unnecessary transitive dependencies (`cargo tree`)

**Rationale**: Security-critical tooling must itself be secure. Rust's safety guarantees are a foundation, not a substitute for security-conscious design.

### VIII. Error Handling Standards

Errors must be actionable, contextual, and never expose internal state:

**Error Design Principles**:
- Use `thiserror` for library error types (structured, composable)
- Use `anyhow` (or `eyre`) ONLY in binary crates and tests, never in library crates
- Error types MUST be enums with meaningful variants, not stringly-typed
- Every error variant MUST carry enough context to diagnose the problem
- Use `.context()` / `.with_context()` to add call-site information

**Standard Error Pattern**:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("Rule file not found: {path}")]
    RuleNotFound { path: PathBuf },

    #[error("Failed to parse rule '{rule_id}': {source}")]
    RuleParse {
        rule_id: String,
        #[source]
        source: ParseError,
    },

    #[error("Scan timed out after {elapsed:?} (limit: {limit:?})")]
    Timeout {
        elapsed: Duration,
        limit: Duration,
    },
}
```

**User-Facing vs Internal Errors**:
- CLI errors: Human-readable with actionable suggestions (use `miette` for rich diagnostics)
- API errors: Structured JSON following RFC 7807 Problem Details
- Internal logs: Full context with `tracing` spans
- Never expose: File system paths, internal module structure, or stack traces to end users

**The `?` Operator**:
- Prefer `?` with proper `From` implementations over `.unwrap()` or `.expect()`
- `.unwrap()` is ONLY acceptable in tests and proven-safe scenarios with a comment
- `.expect("reason")` is acceptable where the invariant is documented and provably upheld

**Rationale**: Rust's `Result` type makes error handling explicit. Well-designed error types turn runtime debugging into compile-time guarantees about error handling completeness.

### IX. Observability & Debuggability

Every crate must be observable in production:

**Structured Logging**:
- Use the `tracing` crate (NOT `log`) for structured, span-based instrumentation
- All public functions SHOULD have `#[instrument]` attributes (skip sensitive fields)
- Log levels: `ERROR` (failures), `WARN` (degraded), `INFO` (operations), `DEBUG` (diagnostics), `TRACE` (verbose)
- Sensitive fields MUST be excluded from spans: `#[instrument(skip(password, api_key))]`

**Tracing Configuration**:
```rust
use tracing_subscriber::{fmt, EnvFilter, prelude::*};

tracing_subscriber::registry()
    .with(fmt::layer().json()) // Structured JSON in production
    .with(EnvFilter::from_default_env()) // RUST_LOG controls levels
    .init();
```

**Metrics & Instrumentation**:
- Prometheus metrics via `metrics` crate + `metrics-exporter-prometheus`
- Health check endpoints: `/health` (liveness), `/ready` (readiness with dependency checks)
- Custom business metrics: operation durations, queue depths, cache hit rates
- Use `tracing` spans for automatic timing of operations

**CLI Diagnostics**:
- Use `miette` for rich error reports with source code snippets and help text
- Support `--verbose` / `-v` flags mapped to `tracing` filter levels
- Support `--format json` for machine-readable output

**Rationale**: Rust's zero-cost abstractions extend to observability. The `tracing` ecosystem provides structured, span-based instrumentation with negligible overhead when disabled.

### X. Simplicity & Pragmatism

Start simple, add complexity only when justified:

- YAGNI (You Aren't Gonna Need It) principle MUST be followed
- Premature optimization MUST be avoided — profile first, then optimize
- Third-party dependencies MUST be minimized and justified
- Design patterns MUST solve real problems, not demonstrate cleverness
- Complexity MUST be justified in documentation
- Monolithic binary preferred until proven bottlenecks require service extraction

**Dependency Principles**:
- Prefer well-maintained crates with active maintainers
- Check crate health: download count, recent commits, open issue response time
- Avoid crates with known soundness issues
- License compatibility: MIT, Apache-2.0, BSD-3-Clause preferred
- Use `cargo deny` to enforce license and advisory policies

**Abstraction Guidelines**:
- Prefer concrete types over trait objects until polymorphism is needed
- Prefer generics over `dyn Trait` for performance-sensitive paths
- Don't create traits with a single implementation unless testing requires it or future extensibility is documented in a PRD
- Use modules for organization, crates for compilation boundaries

**Rationale**: Unnecessary complexity increases maintenance burden. Rust's type system already provides significant safety guarantees — don't add layers that duplicate what the compiler provides for free.

### XI. Current Dependency Policy (NON-NEGOTIABLE)

All dependencies MUST use the most recent stable versions to minimize security risks and technical debt:

**Definition of "Most Recent Stable"**:
- The newest non-pre-release version published on crates.io at the time of addition or update
- "Stable" excludes pre-release tags like `-alpha`, `-beta`, `-rc`

**Version Requirements**:
- New dependencies MUST be added at their latest stable version
  - Use `cargo add <crate>@latest` (or explicitly set the latest version in `Cargo.toml`)
  - Run `cargo update -p <crate>` to ensure `Cargo.lock` resolves to that latest version
- `Cargo.lock` is the source of truth for resolved versions and MUST be updated in the
  same PR whenever dependencies are added or upgraded
- Existing dependencies MUST be kept current (no more than 1 major version behind)
- Security patches MUST be applied within 1 week of release for advisories in `RustSec`
- Dependencies more than 2 major versions behind REQUIRE documented justification and remediation plan

**Pre-Addition Security Checks** (MANDATORY):
- ALL crates MUST be checked against the RustSec Advisory Database before adding:
  ```bash
  cargo audit
  cargo deny check advisories
  ```
- Review crate on `lib.rs` / `crates.io`: last update, downloads, maintainer activity
- Check for `unsafe` usage with `cargo geiger`
- Verify license compatibility with `cargo deny check licenses`
- Review dependency tree impact: `cargo tree -p <new-crate>` (avoid pulling in the world)

**Vulnerable Crates** (STRICTLY FORBIDDEN):
- Crates with active RustSec advisories MUST NOT be added
- Crates with known soundness issues MUST NOT be added
- Unmaintained crates (no commits in 12+ months, no response to issues) REQUIRE justification
- Vulnerable crates in production MUST be patched or replaced:
  - CRITICAL: 24 hours
  - HIGH: 1 week
  - MODERATE: 1 month

**Exception Process for Older Versions**:
Using an older crate version REQUIRES:
1. **Written justification** documenting why latest version cannot be used
2. **Risk assessment** of security implications and known advisories
3. **Approval** from tech lead or security team
4. **Remediation plan** with timeline for upgrading to current version
5. **Documentation** in `SECURITY.md` or `DEPENDENCY_AUDIT.md`

**Acceptable Justifications** (examples):
- Breaking changes in latest version require extensive refactoring (with timeline)
- Dependency conflicts in workspace (with plan to resolve)
- Critical bug in latest version (with link to upstream issue)
- MSRV (Minimum Supported Rust Version) constraints (with upgrade path)

**Unacceptable Justifications**:
- "It works fine" without security analysis
- "Don't have time to upgrade" without risk assessment
- "Too much effort" without documented plan
- Convenience or laziness

**Maintenance Procedures**:
- Run `cargo audit` in CI on every push (block on any advisory)
- Run `cargo outdated` weekly and create issues for crates >1 major behind
- Run `cargo deny check` in CI (advisories, licenses, sources)
- Quarterly dependency audit with `DEPENDENCY_AUDIT.md` update
- Dependabot or Renovate configured for automated update PRs (with test gate)
- Track MSRV policy and document in `Cargo.toml` via `rust-version` field

**Rationale**: The Rust ecosystem moves quickly and security advisories are actively tracked via RustSec. Using current crates reduces vulnerability exposure, provides access to soundness fixes, and prevents accumulation of technical debt that becomes increasingly expensive to address.

## Technology Stack

### Primary Stack

| Component | Technology | Version | Rationale |
|-----------|-----------|---------|-----------|
| **Language** | Rust | Latest stable | Memory safety, performance, type system |
| **Async Runtime** | Tokio | 1.x | Industry standard, excellent ecosystem |
| **HTTP Framework** | Axum | 0.8.x | Tower-based, composable, ergonomic (or Actix-Web 4.x) |
| **CLI Framework** | Clap | 4.x | Derive macros, completions, rich help text |
| **Serialization** | Serde | 1.x | De facto standard, zero-cost when possible |
| **Database** | SQLx | 0.8.x | Compile-time checked queries, async, no ORM overhead |
| **Error Handling** | thiserror + miette | latest | Structured errors + rich diagnostics |
| **Logging/Tracing** | tracing + tracing-subscriber | latest | Structured, span-based, zero-cost when disabled |
| **Testing** | Built-in + proptest + insta + criterion | latest | Comprehensive testing strategy |
| **Build** | Cargo | (ships with rustc) | Workspace management, dependency resolution |

### Rust Toolchain

**Required Tools**:
- `rustup` — Toolchain management
- `cargo` — Build system and package manager
- `rustfmt` — Code formatting (enforced in CI)
- `clippy` — Linting (enforced in CI, `-D warnings`)
- `cargo-audit` — Vulnerability scanning
- `cargo-deny` — License and advisory enforcement
- `cargo-tarpaulin` or `cargo-llvm-cov` — Code coverage
- `cargo-geiger` — Unsafe code auditing

**Recommended Tools**:
- `cargo-watch` — Auto-rebuild on file changes
- `cargo-expand` — Macro expansion debugging
- `cargo-flamegraph` — Performance profiling
- `cargo-outdated` — Dependency freshness checks
- `cargo-tree` — Dependency tree inspection
- `cargo-nextest` — Faster test runner with better output
- `bacon` — Background code checker (alternative to cargo-watch)

**Rust Edition & MSRV**:
- Edition: 2021 (or 2024 when stable and beneficial)
- MSRV: Document in workspace `Cargo.toml` via `rust-version = "1.XX"`
- Update MSRV quarterly, tracking stable release schedule

### Infrastructure

- **Containers**: Docker with multi-stage builds (builder + runtime stages, `scratch` or `distroless` for minimal images)
- **Orchestration**: Docker Compose (local dev)
- **Environment**: `.env` files via `dotenvy` crate (template: `.env.example`)
- **CI/CD**: GitHub Actions (or equivalent) with caching for `target/` directory

### Optional / Project-Specific

| Component | Technology | When to Use |
|-----------|-----------|-------------|
| **Embedding/AI** | `ort` (ONNX Runtime) | Local inference |
| **HTTP Client** | `reqwest` | API calls to external services |
| **Template Engine** | `tera` or `askama` | Report generation |
| **Config** | `config` crate | Multi-source configuration |
| **Crypto** | `ring` or `rustls` | TLS, hashing, signatures |
| **Job Queue** | Custom with Tokio + Redis | Background processing |

## Development Workflow

### Code Quality Standards

**Formatting**: `rustfmt` with project `.rustfmt.toml`:
```toml
edition = "2021"
max_width = 100
use_small_heuristics = "Max"
imports_granularity = "Module"
group_imports = "StdExternalCrate"
```

**Linting**: `clippy` with workspace-level configuration in `Cargo.toml`:
```toml
[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
# Selectively allow where justified
module_name_repetitions = "allow"
must_use_candidate = "allow"
```

**Type Checking**: Rust compiler in strict mode:
```toml
[workspace.lints.rust]
unsafe_code = "warn"          # or "deny" for maximum safety
missing_docs = "warn"         # Enforce documentation
unused_results = "warn"       # Don't ignore Results
```

**Documentation**: `rustdoc` for all public APIs with examples:
```rust
/// Scans a target directory for security vulnerabilities.
///
/// # Arguments
///
/// * `target` - Path to the directory to scan
/// * `config` - Scan configuration including rule sets
///
/// # Errors
///
/// Returns [`ScanError::NotFound`] if the target directory doesn't exist.
/// Returns [`ScanError::PermissionDenied`] if the directory is not readable.
///
/// # Examples
///
/// ```
/// use my_crate::{Scanner, ScanConfig};
///
/// let scanner = Scanner::new(ScanConfig::default());
/// let results = scanner.scan("./target-project")?;
/// assert!(results.findings.is_empty());
/// ```
pub fn scan(&self, target: impl AsRef<Path>) -> Result<ScanReport, ScanError> {
    // ...
}
```

### Pre-commit Requirements

Enforced via Git hooks (e.g., `cargo-husky`, `pre-commit`, or custom `.githooks/`):

```bash
# Format check (fast, catches formatting issues)
cargo fmt --all -- --check

# Lint check (catches common mistakes and anti-patterns)
cargo clippy --workspace --all-targets -- -D warnings

# Security audit (catches known vulnerabilities)
cargo audit

# License/advisory check
cargo deny check
```

Before every commit:
- Code MUST be formatted (`cargo fmt`)
- Clippy MUST pass with zero warnings
- `cargo audit` MUST pass with zero advisories
- Tests SHOULD pass (`cargo nextest run` or `cargo test`)
- Documentation SHOULD build without warnings (`cargo doc --no-deps`)

### Pull Request Requirements

- All quality gates MUST pass (see Quality Gates section)
- At least one approval required from code owner
- All comments MUST be resolved before merge
- Branch MUST be up-to-date with main
- Commit messages SHOULD follow Conventional Commits

**PR Template**:
```markdown
## Constitution Compliance
- [ ] Crate-first architecture (new crate or existing boundary maintained)
- [ ] Rust-first (unsafe justified if present, FFI isolated)
- [ ] Contract-first development (traits, types, errors defined before implementation)
- [ ] TDD followed (tests before implementation)
- [ ] Complete implementation (all tasks in tasks.md done)
- [ ] Performance benchmarks meet requirements
- [ ] Security review passed (no unjustified unsafe, no secrets, input validated)
- [ ] Error handling follows standards (thiserror, no unwrap in production)
- [ ] Observability instrumented (tracing spans, metrics)
- [ ] Simplicity maintained (complexity justified if added)
- [ ] Current dependency policy (all crates current, vulnerabilities scanned)
- [ ] All quality gates pass
```

### Quality Gates (NON-NEGOTIABLE)

Run before every commit and in CI:

**Workspace Commands**:
```bash
# Format all code
cargo fmt --all

# Lint all code
cargo clippy --workspace --all-targets -- -D warnings

# Run all tests
cargo nextest run --workspace          # or: cargo test --workspace
cargo test --workspace --doc           # Doc tests (nextest doesn't run these)

# Test with coverage
cargo tarpaulin --workspace --out html # or cargo llvm-cov

# Security audit
cargo audit                            # Block on any advisory
cargo deny check                       # Licenses + advisories

# Check for outdated crates
cargo outdated --workspace             # Review weekly

# Unsafe audit
cargo geiger --all-features            # Track unsafe usage

# Build all targets
cargo build --workspace --all-targets

# Documentation
cargo doc --workspace --no-deps        # Should build without warnings

# Full pre-commit check
make commit-ready                      # Runs all of the above
```

**Docker Commands**:
```bash
make build         # Multi-stage Docker build
make up            # Start all services
make down          # Stop all services
make logs          # View logs
make health        # Check health endpoints
```

### Branching Strategy

- Main branch: `main` (always deployable)
- Feature branches: `###-feature-name` (### = issue/spec number)
- Hotfix branches: `hotfix-###-description`
- Merge strategy: Squash merge for feature branches

### Commit Messages

Follow Conventional Commits:
```text
feat(scanner): add SARIF output format support
fix(rules): correct false positive on nested structs
docs(api): update OpenAPI specs for scan endpoint
perf(parser): use arena allocation for AST nodes
security(deps): update rustls to patch advisory RUSTSEC-2024-XXXX
refactor(core): extract validation into dedicated crate
test(reporting): add property tests for report generation
```

## Documentation Standards

### Rustdoc

All public items require comprehensive `rustdoc` comments:
- Module-level `//!` docs explaining purpose and usage patterns
- All public functions, structs, enums, and traits documented
- `# Examples` section with compilable, runnable code
- `# Errors` section listing when `Result::Err` is returned
- `# Panics` section if the function can panic (prefer not to panic)
- `# Safety` section for any `unsafe fn`

### API Documentation (if HTTP API)

Use `utoipa` for OpenAPI generation:
```rust
/// Search for vulnerabilities matching the given criteria.
#[utoipa::path(
    get,
    path = "/api/v1/scan/{scan_id}/findings",
    params(
        ("scan_id" = Uuid, Path, description = "Scan identifier"),
        ("severity" = Option<Severity>, Query, description = "Filter by severity")
    ),
    responses(
        (status = 200, description = "Findings retrieved", body = Vec<Finding>),
        (status = 404, description = "Scan not found"),
    ),
    security(("api_key" = []))
)]
async fn get_findings(
    Path(scan_id): Path<Uuid>,
    Query(params): Query<FindingsQuery>,
) -> Result<Json<Vec<Finding>>, AppError> {
    // ...
}
```

### Architecture Reviews and Decisions

- Architecture reviews use `docs/ar/` with the project's AR template
- Architecture decisions use `docs/adr/` with the ADR template

## Governance

### Amendment Process

Constitution changes MUST follow this process:
1. **Propose**: Document rationale and impact in GitHub issue
2. **Discuss**: Team discussion and approval (minimum 2 business days)
   - **Core Principles (I-XI)**: Require team consensus
   - **Standards & Workflow**: Can be updated with documented justification
   - **Technology Stack**: Can be updated as ecosystem evolves
3. **Document**: Migration impact on existing code/practices
4. **Update**: Constitution with version bump (semantic versioning)
5. **Propagate**: Update dependent templates and CLAUDE.md
6. **Communicate**: Announce changes to all contributors

### Versioning Policy

Constitution version follows semantic versioning (MAJOR.MINOR.PATCH):
- **MAJOR**: Backward incompatible changes (principle redefinition, new blocking requirements)
- **MINOR**: New principle added or materially expanded guidance
- **PATCH**: Clarifications, wording fixes, typo corrections

### Compliance Review

All pull requests MUST verify constitution compliance:
- Reviewers MUST check against principles
- New crates MUST document purpose and boundaries in crate-level `//!` docs
- Breaking API changes MUST update OpenAPI specs (if applicable)
- Security changes MUST include threat consideration
- New dependencies MUST pass vulnerability scan and license check

### Constitution Authority

- This constitution supersedes all other development practices
- Conflicts between constitution and external guides MUST defer to constitution
- Constitution violations MUST be fixed or justified before merge
- Use CLAUDE.md for tactical development guidance (this constitution defines strategic principles)

**Version**: 4.0.0 | **Ratified**: 2026-02-10 | **Last Amended**: 2026-02-11

---

## Architecture Decision Records

### ADR-001: Markdown-Only Input Constraint

**Status:** Accepted
**Date:** 2026-02-10
**Deciders:** Brian Luby

#### Context

FORGE's original product roadmap included Phase 2 work items (WI-26 through WI-33) for PDF and DOCX ingestion, comprising 8 sprints of development effort. Risk RR-1 rated PDF extraction crate insufficiency as **High Likelihood / High Impact**, making it the single riskiest line item on the roadmap.

During roadmap review, the following observations emerged:

1. **PDF structural reconstruction is unreliable.** Heuristic heading detection from font size/weight/position is fragile across real-world policy documents. Extraction accuracy targets (>80%) were optimistic given the state of Rust PDF crates.
2. **DOCX parsing adds format-specific complexity** with limited additional value beyond what existing converters already provide.
3. **Mature external converters exist.** Tools like [pandoc](https://pandoc.org/), [markitdown](https://github.com/microsoft/markitdown), and numerous PDF-to-Markdown utilities handle binary format conversion as their core competency.
4. **Building ingestion for binary formats diverts effort** from FORGE's core value proposition: the Markdown-to-OSCAL conversion pipeline, OSCAL model generation, and validation.

#### Decision

FORGE will accept **Markdown as the sole input format**. Users who need to convert PDF or DOCX policy documents will use external tools to produce Markdown first, then feed the Markdown into FORGE.

#### Consequences

**Positive:**
- Eliminates 8 sprints (~2 months) of high-risk development (PDF/DOCX parsing)
- Removes the highest-rated risk on the roadmap (RR-1)
- Compresses the overall timeline by ~2 months (Phase 2: 18 → 10 sprints)
- Focuses engineering effort on OSCAL correctness, validation, and Profile generation
- Aligns with Product Principle P-4 (CLI-first, composable): users compose FORGE with external converters in their pipeline

**Negative:**
- Users must run an extra pre-processing step for non-Markdown policies
- FORGE cannot guarantee extraction quality of upstream converters
- Potential friction for users unfamiliar with Markdown conversion tools

**Neutral:**
- Documentation should recommend specific external converters and provide example pipelines (e.g., `pandoc policy.pdf -t markdown | forge convert --strategy catalog`)
- Future reconsideration is possible if a high-quality Rust PDF parsing crate emerges, but this is not planned

#### Traceability

- **Vision Goals affected:** Removed old G-2 (PDF/DOCX Ingestion); renumbered G-3→G-2, G-4→G-3, G-5→G-4
- **Roadmap items removed:** WI-26 through WI-33, MS-5 (PDF/DOCX Ingestion), RR-1
- **Theme updated:** T-4 renamed from "Format Expansion" to "Output Format Expansion" (XML/YAML output only)
- **Documents updated:** `docs/FORGE_PRODUCT_VISION.md`, `docs/FORGE_PRODUCT_ROADMAP.md`
