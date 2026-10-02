# Assessment Results baseline boundary verification

This F10 prerequisite rejects a native baseline containing multiple result epochs before adding review findings or publishing an artifact/report. The current manifest and report identify one epoch; a schema-valid plural baseline is unsupported input and exits `2`, including with `--fail-on never`. Existing files and source bytes remain unchanged. This does not implement PRD 063 S-3/S-4.

The source baseline is `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`; Veans #20 is under F10 #19 and epic #2. [The versioned JSON packet](assessment-baseline-verification.json) binds complete raw counts, exact source bytes, failed attempts and preserved executables. Worktree pins are recording-time identities; retained snapshots remain valid after branch reuse.

## Reproduction and checks

The first attempt failed to compile three ambiguous empty-slice assertions; it establishes no runtime defect. The corrected unchanged-production run passed 14 and failed 2 tests. It reached only the first populated/forward negative variant: the CLI returned 1 instead of 2 and the public API accepted it. Both negative tests loop and stop on failure, so unreached cases receive no red credit. The unchanged one-result control passed.

The candidate and instrumented integration runs each passed 16/16, with 0 failed/ignored. Four common helper tests are included in that harness denominator. Three new tests cover 32 negative CLI combinations (populated/empty additional epoch, both orders, any/never gate, existing/absent files and file/stdout artifact mode), four direct API negative variants with whole prepopulated-report equality, one positive API comparison and two positive CLI gates. Every plural artifact passes the complete pinned official schema first. Fixtures are synthetic model outputs; they are not authenticated assessor epochs or workflow acceptance.

Strict default all-target Clippy passed. The complete Rust suite passed 2,695, with 0 failed and 3 ignored across 69 summary lines. Checks use Rust 1.98.1 on local macOS ARM64, not an MSRV or hosted-platform qualification. The required commit hook must also run; its final completion is recorded with the exact PR head. No bypass is used.

## Documentation and coverage

Adjacent Rustdoc is present for 9/9 selected named declarations: two production functions and seven new test/helper functions. Whole baseline-module documentation is 2/11; nine private legacy helpers remain undocumented. Whole integration-file documentation is 7/22; all 22 test/helper functions are excluded from the production denominator. CLI baseline-field help and README/usage/assessment contracts describe the one-epoch rejection; fields, types and anonymous closures are outside the named-function audit.

After cleaning old workspace objects, current-only production LCOV covers 306/349 lines (87.68%), retaining 43 uncovered lines. All five added guard entries execute; there are no blank/comment entries or test declarations in this production denominator. The first export mixed old zero-count and current source mappings; its gap-region/comment-filter attribution was incorrect. All earlier attempts are retained as historical artifacts and receive no current-source module coverage credit. The cleaned export contains one current source symbol family. Native LLVM summary denominators are retained separately. No branch/MCDC, statement, whole-project or platform coverage claim is made.

## Remaining gates

S-3 still needs the accepted F08 authority/selection contract and reviewed-risk handoff. S-4 still needs plural manifest/version/identity/graph/context/baseline semantics and real workflow validation. Owner terminology/subset approvals, sanitized assessor workflows, independent downstream interoperability and pilot/release acceptance remain open. No dependencies, schemas, source-selection readers or authoritative remediation contracts change.

The full integrated documentation review/update remains an end-of-roadmap gate, covering all completed work and the CLI, API, examples, architecture, schemas, dependencies, provenance, platform verification and requirement/acceptance evidence. This focused contract update does not close that gate. Nothing here authorizes a merge, release or human acceptance.
