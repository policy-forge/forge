# Supply-chain posture

`cargo vet --locked` gates CI (`.github/workflows/ci.yml`, `audit` job).

- `config.toml` imports the Mozilla, Embark-Studios, and (historically)
  community cargo-vet audit registries; `imports.lock` pins them.
- Most of the graph is still covered by blanket `safe-to-deploy` exemptions
  rather than real audits. That is a deliberate transitional posture, not an
  endorsement: exemptions attest author-side review only.
- Conversion backlog, highest priority first (F1049): crates that parse
  attacker-controlled bytes at runtime — `pdf-extract`, `lopdf`,
  `cff-parser`, `type1-encoding-parser`, `postscript`, `quick-xml`, `zip`,
  `unsafe-libyaml`, `wasmparser`, `wit-parser`, `wit-component`,
  `fancy-regex`, `nom`. Imported registries already cover some of these;
  run `cargo vet prune` after dependency bumps to drop exemptions that
  imported audits now satisfy.
- Application-side defense is independent of vet status: ingest enforces
  `max_size_bytes` (`src/ingest/mod.rs`), and DOCX decompression is bounded.


## Offline dependency inventory (F02, proposed policy)

Use Python 3.11+ and the standard library. No crate or Python package is added.
The committed `dependency-policy.json` remains **proposed** while the PRD-069
owner decisions are unresolved. Native Cargo files and audit history remain
intact. Real exception and review sidecars are empty; fixture signoffs are tests.

```sh
python3 scripts/dependency_inventory.py inventory --as-of 2026-10-02
python3 scripts/dependency_inventory.py gate --as-of 2026-10-02
python3 scripts/dependency_inventory.py gate --strict --as-of 2026-10-02
python3 scripts/dependency_inventory.py validate-store --as-of 2026-10-02
```

Consumption reads only this checkout's bounded committed inputs. It never invokes
Cargo, downloads data, reads the Cargo cache, writes bytecode, or refreshes the
store. Specify an evaluation date explicitly; the same bytes and date produce
identical JSON at another checkout path. `--format text` escapes terminal controls.
The JSON names every locked identity once, source and checksum, classification,
status, proof chain, auditor or exception owner, age, current/retired exemptions,
new runtime gaps, and exact count/age denominators. Invalid input still produces
a report with invalid phases; unknown class or empty proof earns no credit.

Exit 0 means clean under an approved **incremental** policy; 1 means action is
required (new runtime identity, proposed policy, stale exception); 2 means invalid
input or unsupported store semantics. `--strict` also rejects every bare legacy
runtime exemption. `validate-store` rejects each native exemption missing valid
owned sidecar metadata, including retired entries. Merely returning 0 does not
establish PRD-069 acceptance, a complete audit or release readiness.

### Pinning and independently checking the graph

The graph is derived from default-feature locked Cargo metadata for each literal
release target. Source-less first-party status is verified against workspace
manifests; external path dependencies do not inherit it. Runtime closure follows
normal edges, stops proc macros and wins over test/build use. All lock identities,
including target-only and unused candidates, remain visible. Edges are checked
against the lock's possible dependencies and workspace normal dependencies.

Input hashes detect drift; they do not authenticate the graph's edges or remote
proc-macro flags. CI separately regenerates and compares complete canonical graph
bytes. Both that check and the policy gate must pass. A standalone offline report
labels graph qualification pending. Graph preparation requires Cargo and cached
manifests, but does not compile dependencies:

```sh
# Explicit preparation; may fetch locked manifests first if the cache is empty.
cargo fetch --locked
python3 scripts/dependency_graph.py check
python3 scripts/dependency_graph.py refresh --output supply-chain/dependency-graph.json
```

Refresh is a reviewable change; it never changes the frozen legacy baseline.
Initial baseline candidates require explicit `--baseline-output`, refuse existing
files, and grant no audit credit. The policy pins exact baseline bytes. Graph
publication uses no-follow POSIX directory descriptors; Windows can consume the
inventory and run fixtures, but preparation output publication is unsupported.
The Windows reader rejects drive/stream/reserved aliases and reparse points; it
validates a committed checkout and is not a sandbox against concurrent writers.

Build scripts and procedural macros can generate deployed code. Cargo-vet's
[`safe-to-deploy` criterion](https://mozilla.github.io/cargo-vet/built-in-criteria.html)
covers that generated code. PRD-069's runtime versus build class rule needs an
owner disposition before total release dependency assurance; the strict runtime
denominator alone does not settle that question. Metadata can overapproximate
feature-unified/dev dependencies and is not proof of final binary contents.

### Recording reviews and differential audits

Record the actual source examination and notes in `audits.toml`. Then bind its
canonical native record digest in `dependency-reviews.json`, naming the exact
source, method, version checksums and genuine signoff. A full audit binds its
version checksum. A delta binds both old and new checksums, requires a full-rooted
reviewed base and matches each local chain endpoint. A native exemption cannot
seed a delta. Agent-assisted records require the proposed explicit human signoff;
`human-only` policy rejects that method. No signoff may be future-dated.

The sidecar is an asserted attribution, not authentication of the named person or
certification of code safety. Do not enter synthetic reviewers or derive human
approval from a passing test. Unknown criteria, criteria remapping, violations,
wildcard/trusted entries and dependency overrides currently fail closed rather
than being silently ignored. Tests exercise full and differential paths with
explicitly synthetic records; actual runtime audits belong to F03.

### Registering temporary exceptions

An exception binds an exact locked name, version, source and non-null content
checksum. Record owner, rationale, created date, review date, expiry and approval
reference in `dependency-exceptions.json`; keep the native cargo-vet exemption
only while needed for that same identity/criteria. All criteria must be covered.
The proposed limit is 90 days, with creation <= review <= expiry. Dates equal to
review/expiry remain current; later dates are action required. A changed checksum,
version or source receives no legacy credit. External path/Git content binding is
unsupported. Retired/duplicate entries and absent metadata invalidate store checks.
A stale exception always requires action even when an audit also exists.

### Imports, cadence and remaining gates

`dependency-policy.json` proposes exact existing Embark/Mozilla source names and
URLs, pinning the complete `imports.lock` digest. No proposal becomes trusted
before owner disposition. Imports prove crates.io identities only and start at a
full audit; pinned deltas may cross the explicitly approved sources. Refresh
imports explicitly, inspect the diff and full-rooted chain, and update pins through
normal review. A source/URL/pin change is invalid until reconciled.

On every lock/manifest/release-target change, explicitly refresh and qualify the
graph, review new identities and version deltas, and run the policy gate. CI names
new versions and uploads reports even on failure. Trends are machine readable in
the report; release-by-release evidence, actual reduction decisions, schema and
release provenance reconciliation, reminders beyond CI, and release artifact
publication remain F03/F22 acceptance work. None is inferred from this mechanism.

```sh
python3 -B scripts/test_dependency_inventory.py
```
