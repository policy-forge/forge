# Workspace API v2 development verification

The explicit API v2 registration and metadata foundation is implemented against
stacked base `80aa2869ea8823e85638f42345b8ca8e491029ae`. Default launches remain
API v1/1.2.0; `--api-major 2` selects only API v2/2.0.0. This is an unreleased
implementation checkpoint. Full PRD062 S-3/S-6 and the broader roadmap remain open.

See [migration guidance](api/migration-v2.md) for the actual launch, registration,
confirmed index migration, role-specific admission and version-paired bundles.
The [source and coverage audit](plans/2026-10-02-f19-api-v2-verification.json)
records exact source, command, log and raw-export fingerprints. The separate
[selected-function audit](plans/2026-10-02-f19-api-v2-selected-function-verification.json)
binds the final independent documentation and function-entry census to current
source. The source snapshots precede the verification documents and enabled commit hook. Delivery
and hosted outcomes must be bound separately to the eventual immutable commit.

## Executed checks — 2026-10-02

| Check | Observed outcome | Scope |
|---|---:|---|
| Final instrumented Rust run | 1,958 passed, 0 failed | 1,886 library, 19 contract, 32 workspace CLI/HTTP, 21 existing framework-impact CLI tests |
| Maintained-client controls | 59 passed, 0 failed | 25 new version controls and 34 carried extension controls |
| Browser source controls | 150 passed, 0 failed | 105 original controls, 44 version/registration controls and one pending-registration focus/repeat regression |
| Real maintained-client CLI | 2 successful campaigns | Default API1 and explicit API2, owned empty projects, real Session negotiation/context shutdown |
| Bootstrap regression | 1 passed | Both majors' exact metadata, content-addressed asset bytes, security headers and rejected asset requests |
| Strict Clippy | Passed | All targets and existing features; the enabled commit hook repeats default-feature lint on final source |
| Unix packaging rehearsal | 122 exact file members | Current assets plus a development binary; poisoned cached assets excluded; unrelated runner state preserved |

The separate final coverage run preserves the original 1,935-pass run and its
then-unobserved bootstrap/disposition entries. The new bootstrap unit exercises
actual response functions; existing impact CLI tests exercise the preserved
prior-report/disposition consumer. Neither check is browser or human acceptance.

## Documentation coverage

The root Rust census has **68 distinct documented functions**: 28 production,
two cfg helpers, 37 integration functions and one added bootstrap unit. The
earlier 67-function census remains unchanged; exact removal of the documented
unit reconstructs its prior HTTP source byte-for-byte. The producer's separate
Rust/Python/JavaScript cohort overlaps the root cohort and is not added to it.

The complete maintained client has a documented module, both classes and all
**12 functions**, including its nested bootstrap reader. The selected browser
JavaScript changes have **9 of 9** adjacent documentation comments. These are
lexical/AST documentation checks. Unchanged legacy doc gaps remain explicit;
these counts do not claim repository-wide documentation or semantic acceptance.

The combined selected Rust cohort contains **105 of 105 documented functions**
and **105 of 105 observed positive entries**: 56 production, 12 cfg-test and
37 integration functions. The root and producer censuses overlap by three
functions. No selected name is unmapped; positive entry does not establish
complete body or branch coverage. All 5,486 zero raw function records remain
in the retained export and are counted separately.

## Measured execution coverage

The final LLVM export reports **4,922 of 6,395 functions**, **51,762 of 65,648
lines**, and **88,571 of 111,107 regions** positive for the selected Rust run.
All 16 recorded Rust source files match the final run. Reported line/function
aggregates differ from distinct physical source projections and raw alias or
instantiation records. The complete raw export retains zero and generated
records. Branch and MC/DC instrumentation was not requested.

Python stock trace observes **131/142** executable client lines in the 59-control
run, **95/142** in the default CLI case and **107/142** in the API2 CLI case. Their
qualified union is **140/142**; lines 162–163 in shutdown exception handling remain
zero. The tracer marks imports/declarations and installs thread hooks, but its
cover files do not identify threads or establish complete function bodies or
branch coverage. The mocked descriptor-reader observation remains separate from
the real CLI campaigns.

V8 records **128 of 152** production function entries positive and retains all
**24 zero entries**. All nine selected documented JavaScript declarations have a
positive entry. The remaining source-harness gaps include draft and initialization
forms outside this campaign. Anonymous/binding records are separate from named
declaration doc counts. VM/fake-DOM execution proves neither native layout nor
keyboard, assistive-technology, offline OS confinement or human acceptance.

## Packaging and review limits

The offline inventory drift test binds both families to exact contract versions,
39-operation namespaces, fixture/matrix paths and ordered index-schema assets.
Release packaging copies current metadata and the built binary into a fresh
uncached stage. The exact Unix block was exercised in an isolated rehearsal and
all archive file names and bytes matched declared assets, with stale cache inputs
deliberately present. The binary was a development build. Windows packaging has
source review only; PowerShell execution, hosted release builds, signing,
reproducibility, final-byte provenance and publication are not established.

Independent bounded source/data reviews found no remaining material defect in
the selected-major boundary, actual wire replay keys, every public capture and
commit recapture, root busy/focus guard, protected v1 artifacts or final normative
and migration guidance. Reviewers' own authored-control/source exclusions remain
recorded. Passing tests do not supply independent human approval.

Initial compile/type, contract-helper, lint and browser-focus failures remain in
versioned evidence. The clean browser successor preserves original predicates;
temporary diagnostic assertions and incomplete/terminated runs receive no final
control credit. Historical API1 and earlier evidence bytes remain unchanged.

The nine captured lifecycle/impact read queries, complete S-3 views, S-6 confirmed
export/import, current-head hosted platform/CI outcomes, native accessibility,
security/product/pilot and human acceptance, dependency audit/owner decisions,
release provenance and **full final integrated documentation review** remain open.
