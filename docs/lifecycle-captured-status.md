# Captured lifecycle status prerequisite

The existing `forge lifecycle check`, `status`, and `queue` commands read their record and
required artifacts through the existing confined CLI capture wrapper. Their status calculation
now uses a crate-internal projector over a validated record and captured facts. This prerequisite
introduces no workspace route, index role, browser view, or public library API.

`status_from_captured(record, current, as_of)` revalidates the record's intrinsic contract and
the captured inventory. Source and generated hashes must be lowercase SHA-256 strings.
Generated hashes must be unique, sorted by the exact declared path strings, and cover every
declared generated artifact exactly once. Identity-change paths must be a unique subset of
those paths; their supplied order is preserved. The record contract bounds generated artifacts
to 128. Hash differences remain observations of drift. These structural checks cannot
authenticate captured bytes; the capture wrapper reads and classifies actual artifacts.

The projector receives an explicit optional date and performs no filesystem reads, clock
sampling, process execution, or network access. Record validation retains its lexical path
checks. `None` keeps `check` schedule-neutral. Due-soon is inclusive, overdue begins after the
review date, and approved drift takes priority over schedule status. Date overflow returns a
lifecycle error. Status `/1`, queue `/1`, serialized field order, event order, sorted impact
references, recorded approval fingerprints, and the unauthenticated-authority qualification
are retained. Missing or invalid artifacts fail before report publication.

`check` emits its report and exits 1 when approved artifacts have drifted or an artifact identity
has changed. Explicit `status/queue --gate none` emits structurally valid reports without the
action-required exit; it still rejects invalid records and failed artifact reads. The publication
gate retains its existing approval, drift, identity, and overdue rules.

Eight new unit tests exercise complete reports and immutable inputs, malformed captures, record
validation, date boundaries and overflow, drift priority, raw path spelling, and latest approval
history. Two new CLI tests cover complete repeated reports and missing-artifact rejection before
publication. The focused eight tests and all 26 lifecycle CLI tests passed. Separate instrumented
runs passed 19 lifecycle library tests and 26 CLI tests using one fresh LLVM profile directory.
These are synthetic development fixtures; their declared actors and events do not establish
human approval.

A preserved prior debug build and the corrected build produced identical complete stdout,
stderr, and expected exit codes for 24 selected JSON report comparisons across draft, approved,
source drift, and identity drift. The earlier comparison harness stopped on the expected drift
exit 1; its partial outputs are preserved and are not credited as a passing run. Strict private
Rustdoc link checks passed separately for the library and binary. Exact source, docstring and
executable coverage denominators, zero-count records, and limitations are recorded in the
[root validation audit](plans/2026-10-02-f19-lifecycle-captured-status-checks-v2.json).

The separate [portable Lifecycle review exchange](lifecycle-review-exchange.md)
is registered in the open draft stack and consumes its own native closure. Its
four review commands remain separate from the status `/1` report and its
qualification; review quorum grants no native Lifecycle transition or reviewer
authority. The linked guide distinguishes draft-source availability from
acceptance and release.

See the [projector source](../src/lifecycle/status.rs),
[CLI regression tests](../tests/lifecycle_cli_test.rs), and
[delivery plan](plans/2026-10-02-f19-lifecycle-captured-status.md).

The open draft stack also provides [read-only Lifecycle & Impact inspection](workspace-lifecycle-impact.md)
over registered captured inputs. That separate view does not turn this prerequisite
into accepted PRD 062 Should S-3. Human, platform and release acceptance, and the
final all-work integrated documentation review, remain open. Commit, enabled
repository-hook and draft delivery evidence are recorded separately from the
pre-commit validation audit.
