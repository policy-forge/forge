# 051-sec-project-configuration

> **Document Type:** Security Review (Implementation-Phase)
> **Audience:** LLM agents, human reviewers
> **Status:** Approved
> **Last Updated:** 2026-02-21 <!-- @auto -->
> **Reviewer:** Brian Luby
> **Risk Level:** Medium → Mitigated

---

## Review Tier Legend

| Marker | Tier | Speckit Behavior |
|--------|------|------------------|
| 🔴 `@human-required` | Human Generated | Prompt human to author; blocks until complete |
| 🟡 `@human-review` | LLM + Human Review | LLM drafts → prompt human to confirm/edit; blocks until confirmed |
| 🟢 `@llm-autonomous` | LLM Autonomous | LLM completes; no prompt; logged for audit |
| ⚪ `@auto` | Auto-generated | System fills (timestamps, links); no prompt |

---

## Severity Definitions

| Level | Label | Definition |
|-------|-------|------------|
| 🔴 | **Critical** | Immediate exploitation risk; data breach or system compromise likely |
| 🟠 | **High** | Significant risk; exploitation possible with moderate effort |
| 🟡 | **Medium** | Notable risk; exploitation requires specific conditions |
| 🟢 | **Low** | Minor risk; limited impact or unlikely exploitation |

---

## Linkage ⚪ `@auto`

| Document | ID | Relationship |
|----------|-----|--------------|
| Parent PRD | [051-prd-project-configuration.md](../PRD/051-prd-project-configuration.md) | Feature under review |
| Implementation | `src/config.rs`, `src/cli/config_check.rs`, `src/cli/mod.rs` | Reviewed code |
| Tests | `tests/project_config_test.rs`, `src/config.rs` unit tests | Mitigation evidence |
| Dependency review | `supply-chain/config.toml`, verified by `cargo vet --locked` | Supply-chain gate |

---

## Purpose

The PRD requires a security review because **automatic config discovery changes
the behavior of FORGE commands based on repository-controlled content** —
including file-write destinations. A cloned repository from an untrusted source
must not gain the ability to read arbitrary files, write outside the project,
or execute programs through `.forge.toml`.

This review answers:
1. What does an attacker-controlled `.forge.toml` expose?
2. Which trust boundaries does the implementation enforce?
3. What residual risk remains?

---

## Threat Model and Findings 🟢 `@llm-autonomous`

### Assets and boundaries

| Asset | Boundary | Trust |
|-------|----------|-------|
| Host filesystem outside the project root | Write boundary | Must never be writable via config |
| Files inside the project root | Read boundary (explicit references only) | Readable when named by reviewed, committed paths |
| Environment (`$FORGE_CONFIG`, `$FORGE_JOBS`) | Input boundary | Operator-controlled; stricter validation applies |
| Process execution | Execution boundary | Config must never enable or select executables |

### Threats T-1 … T-9

| ID | Threat | Severity | Mitigation (code) | Evidence (tests) |
|----|--------|----------|-------------------|------------------|
| T-1 | Write outside project root via lexical traversal (`../`) | 🟠 High | Absolute values rejected; lexical normalization + root containment in `resolve_inside_root` | `traversal_outside_root_rejected`, `absolute_output_rejected`; AC-14 integration tests |
| T-2 | Write/read escape via symlinks that point outside the root | 🟠 High | Symlink-aware containment: deepest existing ancestor canonicalized; rebuilt path must stay within canonical root | `symlinked_output_path_escape_is_rejected`, `symlinked_input_reference_escape_is_rejected` |
| T-3 | Win32 device redirection via reserved names (`CON`, `NUL`, `COM1-9`, `LPT1-9`) | 🟡 Medium | `reject_windows_device_name` on every config path value, enforced on all platforms | `windows_reserved_device_names_rejected_cross_platform` |
| T-4 | Code execution via discovered config (executables, hooks, round-trip enabling) | 🔴 Critical | Schema v1 is closed and contains no executable-path or process-enabling keys (`deny_unknown_fields` plus explicit key allowlists) | `unsupported_security_sensitive_keys_rejected_ac19`; M-16 test matrix |
| T-5 | Secret/credential exfiltration through `${VAR}`/shell interpolation | 🟠 High | No interpolation anywhere in config values; documented prohibition | Docs state no expansion; parser treats `$` as ordinary text |
| T-6 | Resource exhaustion via oversized/pathological TOML | 🟡 Medium | 1 MiB pre-parse limit; closed schema bounds all numerics (`jobs ≤ 256`, `max-size-mb ≤ 51200`, `timeout ≤ 3600`) | `oversized_config_rejected`, range tests |
| T-7 | Config swap/symlink TOCTOU between selection and use | 🟡 Medium | Selection rejects symlinks up front (`symlink_metadata`); referenced inputs re-stat as regular files at load | `symlinked_config_rejected`, `source_profile_must_exist_as_regular_file`. Residual gap noted below. |
| T-8 | Discovery bypassing unsafe nearest candidate into ancestor config | 🟡 Medium | Discovery selects any existing nearest candidate; loading fails closed under M-10 rather than skipping | `discovery_selects_unsafe_nearest_candidate_instead_of_bypassing` |
| T-9 | Diagnostics leaking absolute local paths into artifacts or logs | 🟢 Low | Diagnostics prefer project-relative rendering; generated OSCAL content never embeds config paths | `display_relative` usage; PRD artifact-limitation note |

### Explicitly out of threat scope

- Malicious *content* of referenced input files (existing ingestion sanitization owns that surface).
- Attacks requiring local attacker control of the operator's environment variables.
- Byte-level artifact drift (functional limitation, not a security issue).

---

## Residual Risks Accepted 🟡 `@human-review`

1. **TOCTOU window** — between load-time validation and command execution, an
   attacker with write access to the repository could swap a validated path's
   symlink target. Window is milliseconds-long and requires local write access
   to a tree the operator chose to run in. Accepted for v1; revisit if a
   long-running watch mode lands (W-3/G-7 territory).
2. **In-root symlink chains** — a committed symlink that stays inside the root
   is permitted and may be retargeted later by a branch change to another
   in-root location. Containment is re-verified per invocation, so impact stays
   within the trust boundary.
3. **Environment trust** — `FORGE_CONFIG`/`FORGE_JOBS` are trusted more than
   repository content because only the operator can set them.

---

## Verification Evidence ⚪ `@auto`

- Full suite: 1560+ tests green including adversarial-config fixtures; zero
  filesystem side effects asserted for invalid configurations.
- Strict gates: `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`.
- Supply chain: `cargo vet --locked` passes (280 exemptions incl. the TOML
  parser stack: `toml 0.9.12`, `toml_datetime`, `toml_parser`, `serde_spanned`,
  `winnow`); `cargo deny check` advisories predate this feature.
- Cross-platform: CI matrix executes the full suite on Ubuntu, macOS, and
  Windows; Windows-specific hazards covered cross-platform by design
  (device-name rejection runs everywhere).

---

## Sign-off

| Item | Decision | Owner |
|------|----------|-------|
| Automatic discovery + path restrictions approved | ✅ Approved (this review) | Brian Luby |
| Schema v1 key list approved | ✅ Approved (see PRD Definition of Ready) | Brian Luby |
| Q-2 ceiling decision recorded | ✅ Keep `max-size-mb = 1..=51200` (PRD Decision Log) | Brian Luby |
