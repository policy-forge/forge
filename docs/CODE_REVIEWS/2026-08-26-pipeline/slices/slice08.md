# Validation slice slice08 — 61 findings
Severity mix: low×61


══════ F0003 │ examples/component-based/output/ssp.json:92-95 │ [maintainability · low] ══════
[maintainability · low] `reason` is not a permitted sibling of `state` in an OSCAL `status` object
(it holds only `state`, an enumerated value, plus optional prose/remarks). While spelled correctly,
this non-standard extension will be dropped or rejected by OSCAL tooling during round-trips; the
explanatory text belongs in prose form (e.g., `remarks`). Note also that top-level user fields like
`authorized-date` follow the same non-standard pattern, so the users blocks should be validated
against the OSCAL SystemUser model before publication.

            "status": {
-             "state": "active",
-             "reason": "automated system process, no interactive login"
-           }
+             "state": "active"
+           },
+           "remarks": "Automated service account for ingestion; no interactive login."


══════ F0892 │ scripts/ci-local.sh:18-18 │ [maintainability · low] ══════
[maintainability · low] Clippy runs only over the default target set, so warnings in integration
tests, benches, and examples (e.g. benches/pipeline_benchmark.rs) are not gated by '-D warnings'.
Use --all-targets so the whole tree passes CI consistently; otherwise local benches/tests can carry
lints that later break an extended pipeline.

- run_step "cargo clippy -- -D warnings" cargo clippy -- -D warnings
+ run_step "cargo clippy --all-targets -- -D warnings" cargo clippy --all-targets -- -D warnings


══════ F0891 │ scripts/ci-local.sh:3-3 │ [maintainability · low] ══════
[maintainability · low] Identical gap in scripts/ci-local.sh and scripts/pre-commit.sh: under set -e
the first failing step kills the script immediately after the last label echo, leaving developers to
guess which gate broke. Install an ERR trap (or handle the status inside run_step) in BOTH scripts
that prints a prominent 'FAILED: <step>' banner and exits with the original step status.

  set -euo pipefail
+ trap 'rc=$?; echo "[ci-local] FAILED at previous step (exit code ${rc})" >&2' ERR


══════ F0896 │ scripts/install-hooks.sh:19-20 │ [bug · low] ══════
[bug · low] The installer never verifies its target: if `scripts/pre-commit.sh` does not exist or is
not executable at the time the hook fires, every future commit fails with a confusing bash error
long after installation succeeded — and this installer still reports success. Additionally, failures
of the `cat >`/`chmod +x` sequence abort (thanks to `set -e`) without a diagnostic message,
potentially leaving a truncated/dead hook behind. Validate the delegate script upfront and emit
errors on the write steps.

- REPO_ROOT="$(git rev-parse --show-toplevel)"
- exec "${REPO_ROOT}/scripts/pre-commit.sh" "$@"
+ # Before installing, verify the delegate script is usable so
+ # misconfiguration fails here instead of at the next commit.
+ PRE_COMMIT_SCRIPT="${REPO_ROOT}/scripts/pre-commit.sh"
+ if [[ ! -f "${PRE_COMMIT_SCRIPT}" ]]; then
+     echo "[install-hooks] ${PRE_COMMIT_SCRIPT} not found; refusing to install a dead hook." >&2
+     exit 1
+ fi
+ if [[ ! -x "${PRE_COMMIT_SCRIPT}" ]]; then
+     chmod +x "${PRE_COMMIT_SCRIPT}" || { echo "[install-hooks] cannot chmod ${PRE_COMMIT_SCRIPT}" >&2; exit 1; }
+ fi


══════ F0015 │ sonar-project.properties:14-14 │ [performance · low] ══════
[performance · low] No exclusion for the Cargo build directory. On a Rust project the scanner will
otherwise crawl target/ (incremental build state, build-script OUT_DIR artifacts, generated/copied
.rs and other files), which inflates scan time and can surface duplicate/generated-code findings.
Add target/** to sonar.exclusions; for deterministic detection you may also want to declare explicit
sonar.sources / sonar.tests roots instead of relying on defaults.

- sonar.exclusions=.specify/**,scripts/**,tests/fixtures/**
+ sonar.exclusions=.specify/**,scripts/**,tests/fixtures/**,target/**


══════ F0311 │ src/applicability/mod.rs:391-397 │ [performance · low] ══════
[performance · low] Inside the edges x targets x sources triple loop, every (source-subject, target)
pair allocates fresh Strings for raw_sha256, source_id, and id_ref just to form a RelationshipKey —
even on pure lookup probes where the entry already exists. Cost scales quadratically with map width
(sources x targets) per edge and repeats per reviewed_at edge across all collections. Intern the
shared components once per edge/source_resource (e.g., Rc<str>/Arc<str>, or build the source-side
key prefix outside the target loop) so per-pair construction copies only short ids, or restructure
lookup so the borrowed (&str-based) view probes the map and cloning happens solely on first insert.

+                 // Interned once per edge/resource; key assembly becomes cheap handle clones.
                  let relationship_key = (
-                     source_resource.raw_sha256.clone(),
+                     std::rc::Rc::clone(&shared.resource_sha256),
                      *source_type,
-                     source_id.clone(),
+                     std::rc::Rc::clone(source_id),
                      target.subject_type,
                      target.id_ref.clone(),
                  );


══════ F0309 │ src/applicability/mod.rs:816-816 │ [security · low] ══════
[security · low] Several interpolated fields bypass escape_html: schema_version (the report's own
manifest declares forge.applicability/1 and here renders the report const), root_uuid,
raw_sha256/manifest_sha256, and item.reason_code. Each is currently provably constrained ('static
consts, Uuid::parse_str-checked UUIDs, computed/lowercase-hex-validated digests), so there is no
exploitable injection today. However, the renderer mixes escaped and unescaped interpolations side
by side, so any future field added here that carries manifest-derived text will silently inherit the
raw pattern. Prefer routing everything through escape_html as cheap defense-in-depth for a
stored-markup-injection surface fed by third-party OSCAL artifacts.

          "<p>Framework: {} <code>{}</code>, root UUID <code>{}</code>, version <code>{}</code>, OSCAL <code>{}</code>, SHA-256 <code>{}</code>.</p>",
+         report.framework.resource_type.as_str(),
+         escape_html(&report.framework.href),
+         escape_html(&report.framework.root_uuid),
+         escape_html(&report.framework.document_version),
+         escape_html(&report.framework.oscal_version),
+         escape_html(&report.framework.raw_sha256)


══════ F0310 │ src/applicability/mod.rs:965-971 │ [maintainability · low] ══════
[maintainability · low] The overdue-deferred CI gate fails open on unparsable revisit_date values:
is_ok_and treats a malformed date as 'not overdue'. Today this branch is unreachable because
manifest::parse rejects non-YYYY-MM-DD revisit_dates and requires them for deferred decisions
(src/applicability/manifest.rs), but nothing in this function expresses or enforces that
precondition — if validation in manifest.rs is ever relaxed or a second construction path for
PreparedAnalysis appears, the gate silently weakens. Fail closed instead: propagate a parse error,
or better, carry the validated NaiveDate (typed during manifest parsing) so this gate cannot drift
from the manifest validator.

              manifest.decisions.iter().any(|decision| {
                  decision.state == manifest::DecisionState::Deferred
                      && decision.revisit_date.as_deref().is_some_and(|date| {
-                         chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
-                             .is_ok_and(|revisit| revisit < as_of)
+                         match chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
+                             Ok(revisit) => revisit < as_of,
+                             Err(_) => true, // fail closed: unparsable date is treated as overdue
+                         }
                      })
              })


══════ F0292 │ src/applicability/model.rs:130-132 │ [test · low] ══════
[test · low] Each variant's wire form exists twice: via the derived serde kebab-case rendering and
via the hand-written const as_str used by the text/emitters in applicability/mod.rs. Nothing ties
them together, so editing one without the other silently desynchronizes JSON and text output. Add a
small unit test asserting serialize(GapClassification) == as_str() for every variant.

- impl GapClassification {
-     #[must_use]
-     pub const fn as_str(self) -> &'static str {
+ // In the test module:
+ // #[test]
+ // fn classification_as_str_matches_serde() {
+ //     for value in [
+ //         Self::ApplicableMapped,
+ //         Self::ApplicableReviewedNoRelationship,
+ //         Self::ApplicableUnmapped,
+ //         Self::NotApplicable,
+ //         Self::Deferred,
+ //         Self::UnderReview,
+ //     ] {
+ //         assert_eq!(serde_json::to_value(value).unwrap(), value.as_str());
+ //     }
+ // }


══════ F0291 │ src/applicability/model.rs:213-217 │ [documentation · low] ══════
[documentation · low] These two implicit rules determine counts, report rows, and review-queue
membership, yet they are only discoverable by reading the guards: (1) when participation facts
contain BOTH positive mappings and reviewed no-relationship entries, the first guard wins and the
control is labeled applicable-mapped, hiding the contradictory no-relationship review from the
queue; (2) an absent manifest decision silently defaults to UnderReview (consistent with manifest's
'omitted controls have UnderReview semantics'). Capture both in a doc comment so future edits to the
precedence order are made knowingly.

+ /// Classify one control. Precedence among participation facts is fixed:
+ /// positive mappings beat reviewed no-relationship relationships, which beat
+ /// unmapped, so contradictory evidence resolves toward the mapped label and the
+ /// control leaves the review queue. Controls with no manifest decision are
+ /// treated as `UnderReview`.
  fn classify(
      decision: Option<&ControlDecision>,
      positive_mapping_count: usize,
      no_relationship_count: usize,
  ) -> GapClassification {


══════ F0276 │ src/batch/formatter.rs:20-24 │ [maintainability · low] ══════
[maintainability · low] Every `writeln!` result is discarded via `let _`. This is only safe because
the target is a `String`, whose `fmt::Write` impl is infallible — but that invariant lives entirely
in the reader's head and silently breaks if this is ever generalized to another sink (file,
io::stderr wrapper), where I/O errors would be swallowed. Document the invariant inline (preferred
here) or centralize the discard in one helper.

-     let _ = writeln!(
+     // Infallible: `String`'s `fmt::Write` implementation never returns `Err`,
+     // so ignoring the result here is intentional.
+     writeln!(
          buf,
          "Batch conversion complete: {} {files_label} ({} succeeded, {} failed) in {total_secs:.2}s",
          summary.total_files, summary.succeeded, summary.failed,
-     );
+     )
+     .expect("write to String cannot fail");


══════ F0278 │ src/batch/formatter.rs:28-31 │ [performance · low] ══════
[performance · low] `to_string_lossy().into_owned()` force-allocates a `String` for every successful
path even though the overwhelmingly common case is valid UTF-8, where `to_string_lossy()` already
returns a borrowed `Cow`. Keeping the `Cow` lets the format machinery borrow the slice and skips one
heap allocation per file; keep in mind batching can run over thousands of inputs.

-         let input_name = result.input_path.file_name().map_or_else(
-             || result.input_path.display().to_string(),
-             |n| n.to_string_lossy().into_owned(),
-         );
+         let input_name = match result.input_path.file_name() {
+             Some(name) => name.to_string_lossy(),
+             None => std::borrow::Cow::Owned(result.input_path.display().to_string()),
+         };


══════ F0279 │ src/batch/formatter.rs:64-71 │ [test · low] ══════
[test · low] Coverage gaps: no test exercises an empty result set (the "0 files" header plus
dangling blank line), an input path whose `file_name()` is absent (e.g., `/` or `..`, hitting the
full-path fallback), or a multi-line/control-character error message — exactly the branches most
likely to regress. All assertions are loose substring checks, so ordering across rows, the trailing
newline, and duplication of names would pass unnoticed; pin the exact expected output in at least
one test.

+     #[test]
+     fn empty_results_renders_header_only() {
+         let summary = BatchSummary::from_results(Vec::new(), Duration::from_millis(10));
+         let output = format_batch_summary(&summary);
+         assert_eq!(
+             output,
+             "Batch conversion complete: 0 files (0 succeeded, 0 failed) in 0.01s\n\n"
+         );
+     }
+
+     #[test]
+     fn multiline_error_is_rendered_on_one_line() {
+         let results = vec![FileResult::failure(
+             PathBuf::from("bad.md"),
+             "Parse error:\nunexpected token".to_string(),
+             Duration::from_millis(50),
+         )];
+         let summary = BatchSummary::from_results(results, Duration::from_millis(50));
+         let output = format_batch_summary(&summary);
+         assert_eq!(output.lines().count(), 3); // header + blank + one row
+         assert!(output.lines().last().unwrap().contains("bad.md"));
+     }
+
      #[test]
      fn all_success_format() {
          let results = vec![
              FileResult::success(
                  PathBuf::from("alpha.md"),
                  PathBuf::from("out/alpha.json"),
                  Duration::from_millis(450),
              ),


══════ F0303 │ src/batch/orchestrator.rs:121-125 │ [maintainability · low] ══════
[maintainability · low] The panic payload (message, location, backtrace) is discarded entirely here
— the `Err(_)` arm maps every unwind to a fixed string without even a `tracing::error!`. The default
panic hook normally prints to stderr, but under a custom hook/subscriber configuration common when
using `tracing` (or suppressed verbosity), the only surviving evidence is 'Internal error (panic
during conversion)', which makes field triage of pipeline panics nearly impossible. At minimum
capture the downcast message/location and emit them to logs (keeping the user-facing summary line
generic); consider also whether stripping the backtrace is intentional given this module's
fault-isolation goal.


══════ F0297 │ src/batch/orchestrator.rs:155-157 │ [test · low] ══════
[test · low] Test coverage stops at `validate_inputs`; neither of the two interesting control-flow
paths in this module is exercised: (1) the `catch_unwind` panic-isolation branch — a pipeline that
panics must yield `FileResult::failure` with the "Internal error" message while the rest of the
batch still completes; (2) the `jobs >= 1` custom-pool branch and its sequential fallback when pool
construction fails. Both are exactly the paths most likely to regress silently.


══════ F0296 │ src/batch/orchestrator.rs:56-59 │ [bug · low] ══════
[bug · low] This public entry point performs no input guarding, which contradicts the module's own
stated design: `validate_inputs`' docs declare it the *single owner* of the
empty-input/no-valid-path invariant and say callers must route through it. Here an empty
`path_pairs` slice silently yields an empty `BatchSummary` (total=0, succeeded=0,
has_failures()==false) — a successful-looking no-op — and nonexistent/non-file entries proceed
straight to per-work item failures long after validation could have caught them up front. Either
call `validate_inputs` (on the input half of the pairs) at the top of this function so the invariant
actually lives in one enforceable place, or relax the doc claim; today enforcement rests purely on
the convention that `execute_dispatch` remembers to call it.


══════ F0285 │ src/batch/output_naming.rs:12-12 │ [documentation · low] ══════
[documentation · low] Rule 4 underspecifies the contract. Two behaviors callers depend on are not
stated: (1) the bare name goes to whichever duplicate appears first in `input_paths` (later ones get
`_{n}`), i.e. assignment is order-dependent; (2) collision avoidance only considers paths minted
during this call — it does not consult the filesystem, so pre-existing files under `output_dir` are
not renamed around. State both in the doc header so refactorings/tests preserve them.


══════ F0288 │ src/batch/output_naming.rs:27-30 │ [bug · low] ══════
[bug · low] Stem derivation degrades poorly on edge-case paths. When `file_stem()` is `None` (empty
path, root `/`), the fallback reuses the entire path string as the stem, producing nonsense
single-component outputs like `/.json` or `<full-path>.json`. Additionally, `to_string_lossy()` maps
every invalid UTF-8 byte to U+FFFD, so two genuinely distinct non-UTF-8 stems collapse to one key in
`next_suffix`/`claimed`, yielding spurious `_2` suffixes and outputs named with replacement
characters. Prefer handling the `None` case explicitly (skip with an error, or hash the raw `OsStr`
bytes for uniqueness) rather than falling back to the lossy full-path text.


══════ F0283 │ src/batch/summary.rs:13-17 │ [maintainability · low] ══════
[maintainability · low] Flattening the failure to a bare `String` erases the original error value
and its source chain right at this boundary: downstream code can no longer match on the failure
cause programmatically (only via brittle string parsing), and richer logging context is lost. Keep
the typed error (e.g. `Arc<dyn Error + Send + Sync>` or the crate's own error enum) in the variant
and render the message only at display time.

      /// The file could not be converted.
      Failure {
-         /// Human-readable error message describing what went wrong.
-         error_message: String,
+         /// Underlying error, preserved for programmatic matching and
+         /// structured logging; format it for display only at output time.
+         cause: Box<dyn std::error::Error + Send + Sync>,
      },


══════ F0284 │ src/batch/summary.rs:178-179 │ [test · low] ══════
[test · low] The tie-breaker branch of the comparator (identical file names in different
directories) and componentless paths are untested — exactly the fragile parts of the sort logic. Add
cases asserting that `/dir1/x.md` vs `/dir2/x.md` orders deterministically and that nameless paths
get a defined position.

-         assert_eq!(names, vec!["alpha.md", "bravo.md", "charlie.md"]);
+     #[test]
+     fn tie_breaker_orders_same_name_by_full_path() {
+         let results = vec![
+             FileResult::success(
+                 PathBuf::from("/dir2/x.md"),
+                 PathBuf::from("x.json"),
+                 Duration::from_millis(1),
+             ),
+             FileResult::success(
+                 PathBuf::from("/dir1/x.md"),
+                 PathBuf::from("x.json"),
+                 Duration::from_millis(1),
+             ),
+         ];
+         let summary = BatchSummary::from_results(results, Duration::ZERO);
+         let paths: Vec<_> =
+             summary.results.iter().map(|r| r.input_path.clone()).collect();
+         assert_eq!(paths, vec![PathBuf::from("/dir1/x.md"), PathBuf::from("/dir2/x.md")]);
      }


══════ F0280 │ src/batch/summary.rs:72-73 │ [bug · low] ══════
[bug · low] `unwrap_or_default()` gives componentless paths (e.g. "/", "foo/..") an empty sort key,
silently collapsing all of them into one tie group that always sorts ahead of every real filename;
their distinguishing information is thrown away at the ordering boundary. Fall back to the full path
as the key instead so every entry keeps a meaningful, unique key.

-             let a_name = a.input_path.file_name().unwrap_or_default();
-             let b_name = b.input_path.file_name().unwrap_or_default();
+         results.sort_by(|a, b| {
+             let key =
+                 |p: &Path| p.file_name().unwrap_or_else(|| p.as_os_str());
+             let a_name = key(&a.input_path);
+             let b_name = key(&b.input_path);
+             a_name.cmp(b_name).then_with(|| a.input_path.cmp(&b.input_path))
+         });


══════ F0281 │ src/batch/summary.rs:74-74 │ [maintainability · low] ══════
[maintainability · low] `OsStr::cmp` compares raw byte sequences: it is case-sensitive ('B.md' sorts
before 'a.md') and platform-dependent (UTF-8 bytes on Unix, UTF-16 units on Windows), while the docs
promise "sorted by input filename" — most users will expect human-friendly, case-insensitive order.
Either normalize keys (e.g. compare case-folded, lossy representations) or state the byte-order
semantics explicitly in the field/type docs.

+             // Ordering is raw OsStr byte order: case-sensitive
+             // ('B.md' < 'a.md') and platform-dependent.
+             // Documented on `BatchSummary::results`.
              a_name.cmp(b_name).then_with(|| a.input_path.cmp(&b.input_path))


══════ F0301 │ src/citation.rs:217-229 │ [maintainability · low] ══════
[maintainability · low] strip_matches trusts callers to supply disjoint, char-boundary-safe ranges:
it sorts but never merges, so a nested/overlapping pair makes `last_end` move backwards and text
after the smaller range is emitted a second time (with wrong spacing). Today overlaps_any preserves
the disjointness invariant, but that guarantee lives three regex passes away and fails silently if
any future pattern forgets an overlap check. Merge the sorted intervals locally so the function is
self-contained; the regex-crate byte indices themselves are boundary-safe, but the function should
not depend on that indirectly either.

      let mut sorted = ranges.to_vec();
      sorted.sort_by_key(|r| r.start);
+
+     // Merge overlapping/nested ranges so `last_end` can never move backwards.
+     let mut merged: Vec<Range<usize>> = Vec::with_capacity(sorted.len());
+     for range in sorted {
+         match merged.last_mut() {
+             Some(last) if range.start <= last.end => last.end = last.end.max(range.end),
+             _ => merged.push(range),
+         }
+     }

      let mut result = String::with_capacity(text.len());
      let mut last_end = 0;

-     for range in &sorted {
+     for range in &merged {
          if range.start > last_end {
              result.push_str(&text[last_end..range.start]);
          }
          result.push(' ');
          last_end = range.end;
      }


══════ F0302 │ src/citation.rs:32-33 │ [other · low] ══════
[other · low] All four detectors are case-sensitive, so "HTTP://example.com", "WWW.Example.com",
"nist sp 800-53", and "SECTION 3.2" are silently ignored with no diagnostic — such citations fall
through neither matching nor stripping. EC-6 shows lowercase cross-references are excluded
deliberately, but that intent is documented nowhere for the other three families. Either widen the
patterns with `(?i)` (where the spec allows) or record the mixed-case exclusion in the module docs,
and consider a tracing::debug! near-miss signal so silent under-detection becomes observable during
triage.


══════ F0307 │ src/cli/config_check.rs:157-160 │ [security · low] ══════
[security · low] The fallback path silently prints the raw (typically absolute) path when
`strip_prefix(root)` fails — e.g., symlinked or non-canonical roots, case-insensitive filesystems,
or genuinely external inputs. Beyond hiding the fact that an input lies outside the project root
(useful diagnostic signal in a file-safety-focused checker), echoing absolute paths can leak
usernames/home-directory names into shared CI logs. Consider annotating unscoped paths explicitly,
e.g. prefixing with `<outside-project-root>` and/or canonicalizing before stripping.

  fn relative_or_self(root: &Path, path: &Path) -> String {
-     path.strip_prefix(root)
-         .map_or_else(|_| path.display().to_string(), |rel| rel.display().to_string())
+     match path.strip_prefix(root) {
+         Ok(rel) => rel.display().to_string(),
+         Err(_) => format!("<outside-project-root> {}", path.display()),
+     }
  }


══════ F0304 │ src/cli/config_check.rs:17-21 │ [documentation · low] ══════
[documentation · low] The documented exit-code contract is inaccurate: it states the sole source of
`ForgeError::Io` is an unreadable working directory on the no-config path, but the final
`write_output(&report, None)` also propagates `ForgeError::Io` for any non-BrokenPipe stdout
write/flush failure (see `crate::cli::output::write_output`), and `BrokenPipe` is deliberately
swallowed as success. Either adjust the wording (e.g., "...and report emission failures, which
surface as `ForgeError::Io`") or classify/write-report errors distinctly so the promised split
(Config=3 vs. Io=1-only-for-cwd) holds.

  /// Writes a diagnostic report to stdout (via [`crate::cli::output::write_output`])
  /// and exits 0 on success. Validation, selection, parsing, and schema failures
- /// return [`ForgeError::Config`] (exit code 3). The sole exception is an
- /// unreadable working directory on the no-config discovery path, which returns
- /// [`ForgeError::Io`] (exit code 1).
+ /// return [`ForgeError::Config`] (exit code 3). Failures that surface as
+ /// [`ForgeError::Io`] (exit code 1) include an unreadable working directory on
+ /// the no-config discovery path and, additionally, stdout write/flush errors.
+ /// (A closed/broken pipe is treated as success.)


══════ F0305 │ src/cli/config_check.rs:33-33 │ [bug · low] ══════
[bug · low] Redundant and inconsistently classified working-directory probe. Reaching the `None`
branch implies `config::load_selected` -> `select_path` already executed `std::env::current_dir()`
successfully and mapped a failure there to `ForgeError::Config`. This second probe maps the
identical failure to `ForgeError::Io`, so the same environmental problem classifies as exit 3 or
exit 1 depending on which syscall trips (and this Io arm is effectively unreachable except when the
cwd is removed mid-race). This undermines the deterministic exit-code contract documented above.
Prefer obtaining the discovery anchor once from the config layer (e.g., return
`(Option<ProjectConfig>, PathBuf)` or a small anchor struct) instead of re-querying the filesystem
here.

+         // TODO(config): plumb the discovery anchor from `load_selected`/
+         // `select_path` so the cwd is probed exactly once and every failure of
+         // that probe classifies identically (currently Config here, Io there).
          let cwd = std::env::current_dir().map_err(ForgeError::Io)?;


══════ F0306 │ src/cli/config_check.rs:41-45 │ [documentation · low] ══════
[documentation · low] The comment overstates what is enforced. `config::validate_cross_field` errors
whenever `strategy = "component"` lacks a config-level `source-profile`, yet its own contract
documents that command execution tolerates such configs because an explicit `--source-profile` on
the command line resolves the conflict (EC-9). Consequently `forge config check` exits 3 for
configurations that *can* run successfully, contradicting both this comment and the module doc's
'reject configurations that could never run successfully'. Either soften the wording to describe
config-layer conflicts, or render CLI-resolvable conflicts as warnings within the report instead of
hard failures.

-     // Cross-field constraints (M-13): reject configurations that could never
-     // run successfully, before printing any report.
+     // Cross-field constraints (M-13): reject settings that conflict at the
+     // config layer. Note some conflicts (e.g. component strategy without a
+     // config-level source-profile) remain resolvable via CLI flags, so these
+     // rejections are stricter than 'could never run successfully'.
      if let Some(convert) = &project.convert {
          config::validate_cross_field(convert)?;
      }


══════ F0318 │ src/cli/convert.rs:42-46 │ [maintainability · low] ══════
[maintainability · low] This traversal mirrors `crate::uuid::assign_stable_ids_to_section_inner`
(same `{section_path}/{title}` scheme) but drops its invariants: the UUID builder stops at
`MAX_SECTION_DEPTH = 50` and, above all, `HashMap::insert` here silently overwrites on locator
collisions — two requirements sharing `(section_path, source_line, atom_index)` would make the
last-seen stable_id win arbitrarily, quietly skewing the diff count rather than surfacing a
conflict. The correctness of `count_substantive_stable_id_changes` (and its symmetric smaller-map
branch) currently depends on these unseen assumptions. At minimum, detect duplicates during
collection (`insert` returning `Some`) and log/panic loudly, and mirror the depth cap so traversal
stays consistent with the ID generator.

      for child in &section.children {
          let child_path = format!("{section_path}/{}", child.title);
          collect_stable_ids_from_section(child, &child_path, map);
      }
  }
+ // NOTE: `collect_stable_ids_from_section` must stay keyed on the exact
+ // (section_path, source_line, atom_index) tuples used by the UUID generator,
+ // must enforce the same MAX_SECTION_DEPTH guard, and must never overwrite an
+ // existing locator entry — a conflicting insertion indicates corrupted input
+ // and should be reported, not silently replaced.


══════ F0317 │ src/cli/convert.rs:97-101 │ [bug · low] ══════
[bug · low] The whitespace-only guard rejects `p.trim().is_empty()`, but the valid branch forwards
the original untrimmed string: a value like `" profiles/x.json "` is stat-ed with surrounding spaces
(producing a confusing "does not exist (not found)" path containing invisible characters) or, if
such a file exists, leaks the raw padded string into the OSCAL profile href downstream. Trim once
and carry the trimmed value forward so the error messages, filesystem access, and serialized href
all agree.

          Some(p) if p.trim().is_empty() => {
              Err(ForgeError::Validation("--source-profile must not be empty".to_string()))
          }
          Some(p) => {
+             let p = p.trim();
              let profile_path = Path::new(p);


══════ F0319 │ src/cli/diff.rs:6-8 │ [documentation · low] ══════
[documentation · low] The `# Errors` section claims `execute` returns `ForgeError::DiffError` for
all failures, but the final `?` propagates the result of `write_output`, whose contract also yields
`ForgeError::Validation` (output parent dir missing) and `ForgeError::Io` (stdout write failure).
Downstream consumers (and this crate's CI contract tests around documented error variants) will see
these variants escape `execute`, contradicting the blanket promise. Update the doc to list all
variants: e.g. "Returns `ForgeError::DiffError` for comparison failures, and `ForgeError::Io` or
`ForgeError::Validation` if writing the report fails." Note the misleading-error exit-code mapping
this creates is intended per error.rs (`DiffHasChanges`→exit 1 vs `DiffError`→exit 2), but accurate
docs are needed for callers reasoning about those codes.

  /// # Errors
  ///
- /// Returns `ForgeError::DiffError` for file I/O, JSON parsing, and artifact comparison errors.
+ /// Returns `ForgeError::DiffError` for file I/O, JSON parsing, and artifact
+ /// comparison errors. Returns `ForgeError::Io` or `ForgeError::Validation`
+ /// if printing the formatted report to stdout fails.


══════ F0337 │ src/cli/drift.rs:36-37 │ [documentation · low] ══════
[documentation · low] The `# Errors` section is incomplete: besides the documented DiffError cases,
the write_output call below can propagate ForgeError::Io when writing the rendered status to stdout
fails (BrokenPipe is deliberately swallowed as success in output.rs; all other write/flush errors
become Io). Callers performing exit-code triage would misclassify such failures as comparison
failures unless the doc states this surface.

+ // Doc comment update:
+ /// Returns [`ForgeError::DiffError`] for file, JSON, model, or status-output
+ /// serialization errors, and [`ForgeError::Io`] when the rendered status
+ /// cannot be written to stdout.
  crate::cli::output::write_output(&output, None)?;
      Ok(comparison.has_drift())


══════ F0338 │ src/cli/drift.rs:40-45 │ [maintainability · low] ══════
[maintainability · low] This local match is the third independent string rendering of ArtifactType
in the tree: `Display` in src/diff/types.rs emits "Catalog"/"ComponentDefinition" (CamelCase) and
canonicalize() in src/diff/canonical.rs separately maps variants to the same kebab-case strings used
here. Because these literals are the machine-readable wire format consumed by CI automation, having
them authored in multiple places risks silent divergence across formats. Consider promoting a single
inherent `as_str()` (following the existing DriftStatus/OscalModelType pattern) on ArtifactType and
consuming it here, so both this renderer and the root-key mapping derive from one authoritative
definition.

- fn artifact_type_name(comparison: &DriftComparison) -> &'static str {
-     match comparison.artifact_type {
-         crate::diff::ArtifactType::Catalog => "catalog",
-         crate::diff::ArtifactType::ComponentDefinition => "component-definition",
+ // In src/diff/types.rs:
+ impl ArtifactType {
+     /// Stable kebab-case representation shared by all machine-readable output.
+     pub const fn as_str(self) -> &'static str {
+         match self {
+             Self::Catalog => "catalog",
+             Self::ComponentDefinition => "component-definition",
+         }
+     }
      }
+ // Then here:
+ fn artifact_type_name(comparison: &DriftComparison) -> &'static str {
+     comparison.artifact_type.as_str()
  }


══════ F0336 │ src/cli/drift.rs:64-66 │ [maintainability · low] ══════
[maintainability · low] The serde_json::Error is erased entirely by the `_` pattern. The
content-free contract (AR-052: no paths, IDs, titles, UUIDs, or excerpts) is not actually at risk
here: JsonOutput contains only &'static str literals and a const u8, so the serializer's own error
text cannot contain artifact content. Preserving the cause (e.g., map_err(|e| ... format!("unable to
serialize drift status output: {e}"))) keeps render failures diagnosable without weakening the
no-disclosure guarantee; as written, any future serialization regression surfaces as a generic,
undiagnosable message.

- let mut rendered = serde_json::to_string(&output).map_err(|_| {
-         ForgeError::DiffError("unable to serialize drift status output".to_string())
+ let mut rendered = serde_json::to_string(&output).map_err(|e| {
+         // Safe to include `{e}`: every field above is a static literal or a
+         // compile-time constant, so the error text carries no artifact content.
+         ForgeError::DiffError(format!("unable to serialize drift status output: {e}"))
      })?;


══════ F0332 │ src/cli/export.rs:136-139 │ [other · low] ══════
[other · low] Detection trusts only the first element's local_name; the namespace URI is discarded,
so a foreign-vocabulary document rooted at <catalog>, <component-definition>, or <profile> in any
namespace clears this gate, and the mismatch only surfaces later as a generic schema/deserialization
error with no pointer at the namespace. Resolve the root element's namespace (e.g. quick-xml's
read_resolved_event()/event_namespace()) and fail fast with a targeted 'unexpected namespace X for
<catalog>' diagnostic; relying wholly on downstream schema validation hides the real cause.

+             // Prefer read_resolved_event() so the namespace URI travels with
+             // the local name and foreign-namespace lookalikes are rejected
+             // with an explicit "unexpected namespace" error.
              Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                  let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                  return Ok(name);
              }


══════ F0334 │ src/cli/export.rs:228-232 │ [performance · low] ══════
[performance · low] Validation re-serializes the already-typed envelope back into a JSON Value on
every export — a second full serialization on top of the target-format output (and for XML/YAML
targets the model is serialized twice in two shapes). Beyond the cost, acceptance now depends on an
assumed-lossless envelope->Value round-trip: any field dropped/altered by a custom Serialize impl,
or lost during XML import, gets validated as-transformed rather than as-authored, so invalid
artifacts can validate cleanly. Document the losslessness invariant on this function (ideally
asserted by a round-trip test fixture), and consider caching/sharing the canonical Value when the
target format is Json to skip the duplicate serialization.


══════ F0331 │ src/cli/export.rs:299-301 │ [performance · low] ══════
[performance · low] Extension-based format detection requires zero I/O, yet it runs after the entire
file has been read and UTF-8 converted (up to MAX_FILE_SIZE bytes). An input with an unsupported or
missing extension pays the full read before hitting a cheap, deterministic rejection. Move
detect_format(input_path)? up right after the existence check (Steps 1-2), so rejectable inputs fail
instantly and the expensive read/UTF-8 work is only done for files that stand a chance of parsing.

-     // Step 5: Detect input format
+     // Step 1: Check file exists
+     if !input_path.exists() {
+         return Err(ForgeError::FileNotFound { path: input_path.to_path_buf() });
+     }
+
+     // Step 2 (cheap, no-I/O): Detect input format BEFORE paying for the read
      let source_format = detect_format(input_path)?;
      info!(source = ?source_format, target = ?target_format, "Format detection complete");


══════ F0335 │ src/cli/export.rs:51-54 │ [documentation · low] ══════
[documentation · low] The # Errors section claims ForgeError::Serialization is returned "if
format-specific parsing fails", but the JSON and YAML branches map every parse failure (malformed
syntax, bad envelope cast, unknown root key) to ExportInvalidOscal; at most the XML delegate can
surface a Serialization variant internally. Align the documented error list with what the branches
actually return, or normalize all parse failures to ExportInvalidOscal and drop the misleading
bullet.


══════ F0344 │ src/cli/migrate.rs:33-34 │ [maintainability · low] ══════
[maintainability · low] `write_output` failures are flattened through `error.to_string()` into
`MigrationError(String)`, discarding the structured `ForgeError` (e.g. `Io(io::Error)` with its
`ErrorKind` and source chain, or `Validation`). Beyond losing diagnostics for callers/tests that
match on variants, this silently re-categorizes the failure: `Io`/`Validation` exit with code 1
while `MigrationError` exits with code 2 (see `exit_code` in src/error.rs), so an ordinary disk-full
or bad-output-directory failure is reported as a migration analysis error. Since `format_json`
errors propagate unchanged above, the boundary is also internally inconsistent. Prefer propagating
the original error or wrapping it so the source chain is preserved (e.g. a `MigrationOutput` variant
carrying the inner `ForgeError`, or keeping `ForgeError::Io`).

-     crate::cli::output::write_output(&rendered, output)
-         .map_err(|error| ForgeError::MigrationError(error.to_string()))?;
+     crate::cli::output::write_output(&rendered, output)?;


══════ F0345 │ src/cli/migrate.rs:99-100 │ [test · low] ══════
[test · low] Test coverage for `reject_output_alias` is thin: there is no test for the successor-map
aliasing branch (`output == successor_map`), nor a positive-case test asserting that a legitimately
new output path next to the inputs is accepted (the second test exercises only the error-message
wording for a missing *input* parent). Both branches carry distinct, hand-written messages that
regress silently unless pinned.

+     #[test]
+     fn rejects_output_that_aliases_the_successor_map() {
+         let directory = tempfile::tempdir().unwrap();
+         let old = directory.path().join("old.md");
+         let new = directory.path().join("new.md");
+         let map = directory.path().join("successors.forge.md");
+         std::fs::write(&old, "# Old\n").unwrap();
+         std::fs::write(&new, "# New\n").unwrap();
+         let result = reject_output_alias(Some(&map), &old, &new, Some(&map));
+         assert!(matches!(result, Err(ForgeError::MigrationError(_))));
+     }
+
+     #[test]
+     fn accepts_fresh_output_path_adjacent_to_inputs() {
+         let directory = tempfile::tempdir().unwrap();
+         let old = directory.path().join("old.md");
+         let new = directory.path().join("new.md");
+         let output = directory.path().join("report.txt");
+         std::fs::write(&old, "# Old\n").unwrap();
+         std::fs::write(&new, "# New\n").unwrap();
+         assert!(reject_output_alias(Some(&output), &old, &new, None).is_ok());
+     }
+
      #[test]
      fn missing_input_directory_names_the_input_role() {


══════ F0339 │ src/cli/mod.rs:1-2 │ [documentation · low] ══════
[documentation · low] This single-line doc comment reads like the crate/root overview ('Convert
subcommand: policy document → OSCAL artifact.') but is attached to `mod config_check` (a subcommand
for validating .forge.toml), making the module documentation misleading. Rustdoc renders these as
different items, so `cargo doc` will show the wrong description for config_check. Either move the
intended root-level overview into a proper module doc (`//!`) at the top of the file, or replace it
with an accurate description such as 'Config check subcommand: inspect and validate .forge.toml.'

- /// Convert subcommand: policy document → OSCAL artifact.
+ /// Config check subcommand: inspect and validate the selected .forge.toml.
  pub mod config_check;


══════ F0342 │ src/cli/mod.rs:324-326 │ [maintainability · low] ══════
[maintainability · low] Same issue as `Lifecycle Transition --at`: the '--timestamp' contract
requires ISO 8601 but the flag accepts any arbitrary string (including empty), pushing format
validation into `profile::execute`/serialization and risking inconsistent acceptance rules across
commands. Parse to a typed instant (e.g. `chrono::DateTime<Utc>` or `jiff::Timestamp`) with a clap
value parser so reproducibility runs abort immediately on malformed input.

          /// Override the last-modified timestamp (ISO 8601) for reproducible output.
-         #[arg(long)]
-         timestamp: Option<String>,
+         #[arg(long, value_parser = clap::builder::ValueParser::new(parse_iso8601))]
+         timestamp: Option<chrono::DateTime<chrono::Utc>>,


══════ F0341 │ src/cli/mod.rs:498-500 │ [maintainability · low] ══════
[maintainability · low] The transition timestamp is captured as an opaque `String` despite being
specified as 'RFC 3339', deferring validation (and possibly accepting drift between spelling and
what's enforced) to `lifecycle::execute_transition`, deep inside write logic that is supposed to
append atomically. Parsing at the boundary would fail fast with clap's standard usage error instead
of surfacing a ForgeError mid-command, and keeps malformed state out of downstream code. Consider
`next_review`-style typed parsing, e.g. `value_parser = humantime::parse_rfc3339` /
`chrono::DateTime<Utc>: FromStr`, storing `chrono::DateTime<Utc>` (or `jiff::Timestamp`).

          /// Explicit RFC 3339 event timestamp
-         #[arg(long)]
-         at: String,
+         #[arg(long, value_parser = clap::builder::ValueParser::new(parse_rfc3339))]
+         at: chrono::DateTime<chrono::Utc>,


══════ F0328 │ src/cli/output.rs:10-15 │ [documentation · low] ══════
[documentation · low] Behavioral gap in the contract: on `BrokenPipe` the function swallows the
error and returns `Ok(())`, so a caller piping into e.g. `head` gets a zero exit status even though
the payload was truncated mid-stream. That is a reasonable CLI convention, but it should be stated
in the doc comment; likewise the `# Errors` section should mention that `ForgeError::Io` can
originate from the stdout branch, not only 'file write'.

  /// Handles `BrokenPipe` gracefully (e.g., when piped into `head`) instead of
- /// panicking like `print!` would.
+ /// panicking like `print!` would. Note this reports success (exit code 0)
+ /// even if the consumer disconnected before receiving the full output.
  ///
  /// # Errors
  /// * `ForgeError::Validation` if parent directory does not exist
- /// * `ForgeError::Io` if file write fails
+ /// * `ForgeError::Io` if the write to stdout or the target file fails


══════ F0329 │ src/cli/output.rs:33-36 │ [other · low] ══════
[other · low] Check-then-act race: `parent.exists()` can pass/fail between the probe and the
rename-based write, in which case the race manifests as a bare io::Error (ENOENT) from
`write_atomic` instead of this friendly `Validation` message. The pre-check is safe (the underlying
operation still fails atomically on a missing directory) but purely advisory — consider commenting
it as a fast-path UX nicety so future readers do not treat it as a correctness guard, mirroring the
`parent.filter(...).unwrap_or(Path::new("."))` normalization done inside `write_atomic`.

+             // Advisory fast-path: gives a clear "directory missing" error up front.
+             // Racy (dir can vanish before write_atomic runs) but safe: the write
+             // itself fails atomically with an io::Error in that case.
              if let Some(parent) = path.parent()
                  && !parent.as_os_str().is_empty()
                  && !parent.exists()
              {


══════ F0324 │ src/cli/profile.rs:122-122 │ [performance · low] ══════
[performance · low] The entire serialized document (JSON pretty-printed, XML, or YAML) is fully
materialized into a `String` before `write_output` runs, even when `output` points to a file. Peak
memory is roughly doubled (in-memory buffer plus buffered writer) and grows linearly with the source
catalog size. Consider streaming serializers straight into the destination writer
(`serde_json::to_writer_pretty`, quick-xml `Writer`, serde_yaml `to_writer`) when `output` is
`Some`, reserving the buffered path for stdout only.

-     let serialized = match format {
+     // Stream directly into the destination writer when a file target is given;
+     // buffer only for stdout so large profiles don't double peak memory.
+     if let Some(out) = output {
+         let file = std::fs::File::create(out)?;
+         let mut writer = std::io::BufWriter::new(file);
+         match format {
+             OutputFormat::Json => serde_json::to_writer_pretty(&mut writer, &root)
+                 .map_err(|e| ForgeError::Serialization(format!("Profile JSON serialization failed: {e}")))?,
+             OutputFormat::Xml | OutputFormat::Yaml => { /* streaming equivalent */ }
+         }
+         writer.flush()?;
+         return Ok(());
+     }


══════ F0325 │ src/cli/profile.rs:20-21 │ [documentation · low] ══════
[documentation · low] The `# Arguments` section documents every parameter except `timestamp`,
although it affects observable behavior (overrides the Profile's last-modified value and is
validated against RFC 3339 with its own `# Errors` entry listed as an ISO-string requirement). Add a
bullet for it so the public handler contract stays complete.

  /// * `set_params` — Flat `[id, value, id, value, ...]` slice from `--set-param` flags (WI-31).
  ///   Pass `&[]` when no `--set-param` flags are provided.
+ /// * `timestamp` — Optional RFC 3339 string overriding the Profile's last-modified value;
+ ///   parsed below and rejected with `ForgeError::InvalidArgument` if malformed.


══════ F0323 │ src/cli/profile.rs:75-80 │ [maintainability · low] ══════
[maintainability · low] The same warning is emitted twice — once via raw `eprintln!` and once via
`tracing::warn!` — so when a tracing subscriber writes to stderr (the common setup) users see
duplicate output, and the `eprintln!` copy bypasses log-level filtering and formatting entirely.
Emit only through `tracing::warn!` (or only `eprintln!` if the tracer is configured away) so
diagnostics stay consistent with the rest of the command, which uses `tracing::info!`.

-             eprintln!(
-                 "warning: --set-param specified without --include or --exclude; the Profile will have no control imports"
-             );
              tracing::warn!(
                  "--set-param specified without --include or --exclude; Profile will have no control imports"
              );


══════ F0349 │ src/cli/resolve.rs:152-153 │ [maintainability · low] ══════
[maintainability · low] Non-UTF-8 file stems are silently replaced by the literal fallback
"profile", producing an unexpected `<stem>-resolved.json` default output (e.g.
'/data/<lossy>-resolved.json' becomes '/data/profile-resolved.json') with no diagnostic — on
multi-user systems this could also collide with a concurrently produced default in the same
directory. Use to_string_lossy() to preserve the actual (lossily-rendered) stem and reserve the
"profile" fallback only for stems that are genuinely empty.

  fn derive_default_output_path(input: &Path) -> PathBuf {
-     let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("profile");
+     // Preserve non-UTF-8 stems (lossy) instead of silently renaming the output.
+     let stem = input
+         .file_stem()
+         .map(|s| s.to_string_lossy())
+         .filter(|s| !s.is_empty())
+         .unwrap_or_else(|| Cow::Borrowed("profile"));


══════ F0350 │ src/cli/resolve.rs:74-75 │ [performance · low] ══════
[performance · low] Detection here always runs a full `oscal-cli --version` preflight, and the very
next step spawns oscal-cli again for the real resolve — two JVM startups back-to-back add measurable
latency (often 1–2+ s each) to every resolve. The resolve attempt itself surfaces the same
broken-install failures, so consider short-circuiting/skipping the --version probe when a concrete
invocation follows immediately, or caching detection results across commands.

-     // Detect oscal-cli
+     // Detect oscal-cli (--version preflight costs a full JVM start; consider
+     // skipping it when a real resolve invocation follows immediately).
      let cli_info = detector.detect();


══════ F0361 │ src/cli/validate.rs:161-164 │ [maintainability · low] ══════
[maintainability · low] The size/read/empty/parse preamble here duplicates `execute()` almost line
for line (including the same `ForgeError` mappings), and the step comments have already drifted —
two consecutive blocks are both labeled 'Step 3' ('Read and parse original JSON' and 'Detect
oscal-cli'). Extract a shared `read_and_parse_artifact(input) -> Result<serde_json::Value,
ForgeError>` helper and renumber the steps to prevent further drift between the two entry points.

-     // Step 3: Read and parse original JSON
-     let content = std::fs::read_to_string(&canonical_input).map_err(|e| {
-         ForgeError::Validation(format!("Failed to read artifact file '{}': {e}", input.display()))
-     })?;
+     // Shared with `execute()`; keep this step list in sync via one helper.
+     let original_json = read_and_parse_artifact(input)?;


══════ F0362 │ src/cli/validate.rs:276-278 │ [bug · low] ══════
[bug · low] `format` is accepted by `execute_round_trip` but silently ignored whenever `--output` is
provided: every on-disk report goes through `write_divergence_log(result, output_path)`, which
always serializes pretty-printed JSON regardless of `ValidateOutputFormat`. A caller passing
`--format json` coincidentally gets JSON, but a `--format text` caller unexpectedly receives machine
JSON in the file (and no human-readable summary beyond the FAIL hint). Honor the requested format
(or reject unsupported combinations with an explicit error) so the CLI behaves predictably.

      match output {
          Some(output_path) => {
-             write_divergence_log(result, output_path)?;
+             // Serialize the report in the caller-requested format.
+             match format {
+                 ValidateOutputFormat::Text => {
+                     crate::cli::output::write_output(&render_round_trip_text_str(result), Some(output_path))?;
+                 }
+                 ValidateOutputFormat::Json => write_divergence_log(result, output_path)?,
+             }


══════ F0363 │ src/cli/validate.rs:30-31 │ [documentation · low] ══════
[documentation · low] Doc comment is stale: the success path no longer prints a bare "Valid" line —
it renders the full WI-20 report (text or JSON) via `write_output`. Update the description so
callers relying on stable stdout output don't get surprised.

  /// Uses `run_full_validation()` for enhanced error reporting (WI-20).
- /// On valid: prints "Valid" to stdout + exit 0.
+ /// On valid: renders the full report (text or JSON) to stdout/--output + exit 0.


══════ F0386 │ src/config.rs:0-0 │ [bug · low] ══════
[bug · low] On Windows, `Path::join`/`push` treats a pushed operand as either absolute (replaces the
base, caught here) or drive-RELATIVE (prefix like `C:` swaps the base yet keeps subsequent
components appended to the new prefix, NOT under project_root). The second shape slips past the
`is_absolute()` guard by luck (the resulting path simply isn't under root, giving a generic error).
Worse, both platforms silently drop information during normalization: `Path::new(a).join("b\\c")` on
Unix keeps `b\\c` as ONE component (the backslash is an ordinary character), while on Windows
`"generated\\out.json"` stays in-root; a config authored with native-looking separators works on
Windows but explodes on Unix with a confusing 'file not found'. This file already special-cases
Win32 quirks (reserved devices); portability-aware decoding deserves the same care.


══════ F0389 │ src/config.rs:0-0 │ [maintainability · low] ══════
[maintainability · low] Bug in the tie/win logic (`closest_key`): `ties += 1` fires on EVERY
subsequent equidistant candidate, but `best` is overwritten by `_` whenever `d < bd`, resetting ties
— correct so far — yet once ties > 0 a strictly closer candidate correctly resets. However, ties is
compared as `== 0` at the end, so the FIRST candidate equal-distance case when best==None is
miscounted: initial `_` arm sets best=(d,candidate) even when d>2 if every candidate matched
`Some((bd,_)) if d > bd` guard order incorrectly (e.g., candidates yielding distances 4 then 3:
first sets best=(4,x) even though d=4>2, second hits `_` and overwrites). Consequence:
distant-but-inconsistent best may suppress or produce wrong suggestions; also `ties` counts
duplicate distances only among d<=2, mixing semantics. Rewrite as explicit guarded pattern shown.


══════ F0390 │ src/config.rs:0-0 │ [bug · low] ══════
[bug · low] The missing-input diagnostic renders the RESOLVED absolute path via `display_relative`,
discarding the exact value the user wrote. For `source-profile = "nested/link.json"` the error says
`` references 'nested/link.json': No such file…`` only when the file is absent; but when the path
traverses a symlinked directory (allowed when contained), `resolved` is the JOINED lexical path
while the failure text implies it equals the configured value — and if `resolve_inside_root`
normalized segments, the printed suffix no longer matches what users can grep for in their
`.forge.toml`. Preserve the raw configured string in messages about that setting.


══════ F0391 │ src/config.rs:0-0 │ [maintainability · low] ══════
[maintainability · low] Two parallel sources of truth for the closed schema (Pre-scan focus #4):
these hand-maintained key lists mirror `RawFile`/`RawConvert`/`RawValidate`, which independently
carry `deny_unknown_fields`. Adding a field to one side but not the other degrades UX (generic serde
message instead of the tailored 'did you mean'), causes duplicate suggestion/wording maintenance,
and risks drift between discovery-time validation and post-parse enforcement. Derive one from the
other, or add a compile-time/debug assertion that they agree (e.g., serialize a unit instance of
each struct and diff the emitted key sets in a unit test).


══════ F0392 │ src/config.rs:0-0 │ [bug · low] ══════
[bug · low] `[convert]` / `[validate]` defined as scalars (e.g. `convert = "oops"`) pass
`check_unknown_keys` silently: the arm yields `sub_keys = None`, so their inner structure is never
inspected and the unknown-key machinery never flags the section — the failure surfaces later only as
a serde type-mismatch with no 'did you mean' affordance, diverging from M-6 diagnostics for every
malformed-table case. Require tables (or use `toml_edit`'s typed descent) instead of quietly
skipping non-tabular sections.


══════ F0375 │ src/diff/canonical.rs:110-115 │ [maintainability · low] ══════
[maintainability · low] Every diagnostic in `parse_artifact` identifies the failing side only by
role label ('committed'/'generated') and omits the path itself. On a mismatch a user with several
candidate files cannot tell which path failed to stat/read/parse, unlike the crate-wide convention
(e.g. `ForgeError::FileNotFound { path }`) of surfacing the path. Paths are CLI-supplied inputs, not
artifact content, so including them does not violate this module's content-free guarantee.

      let metadata = std::fs::metadata(path).map_err(|error| {
          ForgeError::DiffError(format!(
-             "unable to inspect {role_name} artifact ({:?})",
+             "unable to inspect {role_name} artifact '{}' ({:?})",
+             path.display(),
              error.kind()
          ))
      })?;


══════ F0374 │ src/diff/canonical.rs:150-153 │ [maintainability · low] ══════
[maintainability · low] This arm discards the concrete `ValidateError` returned by
`detect_model_type`, collapsing malformed shapes and ambiguous roots into one generic message. When
a contributor feeds a Profile-that-almost-parses or an ambiguous hybrid artifact, triage cannot
distinguish 'unexpected model' from a detection-layer defect. Preserve the cause via its `Display`
impl while keeping the same `ForgeError::DiffError` variant.

-         Err(_) => Err(ForgeError::DiffError(format!(
-             "{} artifact is not a recognized Catalog or Component Definition",
+         Err(error) => Err(ForgeError::DiffError(format!(
+             "{} artifact is not a recognized Catalog or Component Definition: {error}",
              role.as_str()
          ))),


══════ F0376 │ src/diff/canonical.rs:89-89 │ [documentation · low] ══════
[documentation · low] `Value` equality is stricter than documented: serde_json treats `1` (u64) and
`1.0` (f64) as distinct numbers, so an innocuous reserialization that changes a numeral's lexical
form (integer vs float, or overflow-promoted precision) surfaces as Drift even though every
human-visible field matches. Array-order significance is intentionally stated; the
numeric-representation sensitivity is not. Documenting it here — or normalizing number forms if the
generator may emit `1` vs `1.0` across releases — prevents false-positive drift reports being
misread as tampering.
