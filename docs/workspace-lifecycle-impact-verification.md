# Lifecycle and framework-impact development verification

API2/2.1.0 adds nine authenticated read queries to the initial API2/2.0.0 foundation in draft PR #193. The selected namespace now has 48 operations; default API1/1.2.0 retains 39. Product release target 2.0.0 is separate from these unreleased contract versions. The basis is `41a2698cc44b84bfb20081c67a3484ce9875c12a`.

The [usage guide](workspace-lifecycle-impact.md) describes explicit dates, captured input closure, filters, paging and authority. The [source and coverage record](plans/2026-10-03-f19-lifecycle-impact-verification.json) binds exact source, retained logs, exports, selected names and zero entries. This is a pre-commit development checkpoint; the enabled commit hook, immutable commit and hosted delivery need separate readback.

## Executed checks

| Check | Outcome | Scope |
|---|---:|---|
| Fresh final instrumented Rust run | 2,032 passed; no failures or ignored tests | 1,928 library, 22 contract, 35 workspace HTTP, 21 impact CLI, 26 lifecycle CLI |
| Python controls | 117 passed | 25 version, 34 extension, seven new lifecycle/impact, 51 receipt controls |
| Browser source controls | 178 passed | Final asset bytes; Node VM/fake DOM harness |
| Actual maintained client | Nine methods in each of two scopes | Native local HTTP; read-only and writable; same capture; no project-file changes |
| Actual installed Chrome | Nine S3 GET operations, three focus checks | Synthetic read-only fixture, no substituted responses; zero page errors or non-loopback requests |
| Strict Clippy | Passed | Existing all-target/all-feature configuration, warnings denied |

The native regressions cover absent, legacy and empty indexes, real nonempty Mapping/applicability findings, five-filter AND semantics, exact current disposition IDs, complete hidden blockers, cursor binding and missing registered dependency refusal. Inspection queries preserve the seeded project bytes. Synthetic actors remain declared assertions.

## Documentation and coverage

The final nine-file Rust cohort has **162/162 documented functions** and **159/162 positive direct entries**, with zero unmapped names. Its retained zero entries are `crate::workspace::impact::internal`, `crate::workspace::impact::tests::Stopped::interruption`, `crate::workspace::inspection::tests::Stopped::interruption`. Positive entry is weaker than complete function-body or branch coverage. Generated records and aliases remain separate.

The LLVM successor reports **55,112/68,717 lines (80.20%)**, **5,218/6,686 functions (78.04%)** and **95,200/117,417 regions (81.08%)** for the five selected suites plus gracefully terminated native sessions. It retains all 12,212 raw function records, including 5,555 zeros. These are reported aggregates, not unique physical line or repository-wide coverage. Branch and MC/DC instrumentation were not requested. The original final test export and earlier source reports remain unchanged.

The selected Python cohort has **12/12 docstrings and positive entries**. The full maintained client has **23/23 documented functions**: 20 positive entry-lines in mocked controls and a qualified native union of 22/23. The nested `read_descriptor` remains unobserved because this tracer does not install thread tracing. The new test file has **15/15 documented functions and 14 positive entry-lines**; its `kill` helper remains zero. Other suite files have retained after-run trace hashes; only the client and new test file have that generic job's before/after guards. Entry-line observations are not exact invocation counts or full-body coverage.

The bounded JavaScript cohort has **40/40 adjacent documentation comments** and **37/40 positive V8 entries**. `resumeInspectionPage`, `resumeInspectionSelection` and `resumeInspectionView` remain zero. The whole asset includes **199/231 positive first entries**, retaining 32 zeros and all anonymous records; engine ranges are **910/1,087 positive**. Eighteen unchanged bounded lexical documentation gaps remain. This is not a complete JavaScript AST/method census or native browser coverage.

The appended browser harness has **11/11 documented named helpers and positive V8 entries**. Twenty commented test-call sites expand through four loops into 28 added cases. These declaration, syntax-site and executed-case denominators remain distinct; other callbacks are retained separately and do not receive named-helper documentation credit.

## Native observations and review limits

Chrome 154.0.8037.98 used the actual compiled server, current asset bytes and keyboard Enter. The browser run precedes the seven added cfg-test controls; every prior production byte remains an exact prefix in the two affected modules, and other recorded source bytes match. Inventory date submission, filtered changes and initial credential focus passed. Measured page widths were exactly 640 and 320 pixels at those viewports. Owned fixture file hashes stayed unchanged; shutdown closed both listener ports. Screenshots show synthetic project data. This does not establish native assistive technology, complete WCAG, Windows/Linux browser parity, OS-wide network denial or human acceptance.

Independent bounded source, DTO, documentation and evidence arithmetic reviews retain authors' own-code exclusions. Root execution corrected test-only JSON macro syntax, an overbroad `rationale` substring assertion and an incorrect six-file provenance denominator: the native engine consumes five closure inputs. The Mapping embeds its policy identity. Original failed logs remain preserved. Early browser focus-probe failures retained a detached DOM handle; the passing successor observes the current focused region without changing product focus behavior.

Platform/hosted parity, authentic interoperability and evaluation, independent security/privacy, human accessibility and pilot acceptance, dependency audit and owner decisions, S6 full import/export, release provenance and the **full final integrated documentation review across all completed roadmap work** remain open. Passing development checks and this focused guide do not close those gates.
