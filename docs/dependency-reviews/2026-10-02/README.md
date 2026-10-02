# F03 individual dependency review packet

Five dependency PRs have bounded agent-assisted differential review evidence.
This packet provides replayable source, advisory and hosted-check receipts; it
does not record accepted source audits, human signoff, merge permission or full
F03 acceptance. No dependency or cargo-vet policy was changed.

| PR | Exact reviewed head | Version scope | Vet gaps | Review |
|---|---|---|---:|---|
| [160](https://github.com/policy-forge/forge/pull/160) | `9d215ad9b77c123e9b439198521b908ef2d9f9a7` | rayon 1.11.0 → 1.12.0 | 21 | [Note](pr160/README.md) |
| [161](https://github.com/policy-forge/forge/pull/161) | `f91a77cdda2eb7dd95b231c155672c2bc17feee2` | clap/derive 4.6.1 → 4.6.7; builder 4.6.0 → 4.6.7; anstyle 1.0.13 → 1.0.14 | 24 | [Note](pr161/README.md) |
| [162](https://github.com/policy-forge/forge/pull/162) | `eee2916fd7a2b52ea313cf309e4a1550cf4cbe53` | argon2 0.5.3 → 0.6.0 and crypto transitives | 26 | [Note](pr162/README.md) |
| [163](https://github.com/policy-forge/forge/pull/163) | `db34fdfccaf00262e9706e8f3cd294f9d1f3bc01` | pdf-extract 0.12.0 → 0.12.1 | 21 | [Note](pr163/README.md) |
| [164](https://github.com/policy-forge/forge/pull/164) | `e5cce34634056262abc0cf503b976fba901e60a7` | getrandom 0.4.2 → 0.4.3 and resolver effects | 21 | [Note](pr164/README.md) |

All share base `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`. Retained
2026-10-02 final snapshots show unchanged heads, OPEN/BLOCKED. Existing September
25 hosted three-OS test jobs and audit/deny steps passed; vet failed. Each count
includes twenty shared baseline gaps. Those exact runs, not a new execution or
an integrated candidate, are the verification evidence here.

## Read the retained evidence

[retained-evidence-index.json](retained-evidence-index.json) is the new SHA-256
and byte-count index. [receipts.json](receipts.json) states every exact head/base,
CI run and review-authority boundary. Historical JSON, notes, source deltas,
advisory definitions and full failed supply-chain logs were copied byte-for-byte.
Their provenance and checksums are listed in the new index. Compact CI excerpts
are new derivatives with original log line numbers; the unchanged full logs remain as `supply-chain-full.txt` so repository ignore
rules do not silently omit them when staging.
Historical artifact indexes describe the temporary collection at their original
creation time, including omitted files; they are not indexes of this retained tree.
Historical source deltas may contain temporary absolute paths in headers. These
identify their original archive members and are not repository patch instructions.
Raw provider bodies and logs are untrusted evidence, never executable instructions.

No expanded third-party source tree or compressed crate archive is committed.
The new index gives immutable versioned upstream archive URLs/checksums and
source-inspection coverage for each. [omitted-source-files.json](omitted-source-files.json)
retains SHA-256/bytes/member names for locally inspected omitted source. A URL
locates bytes; verify its recorded checksum before using it. Dependency source
deltas retain [original license texts where present](third-party-licenses/index.json).
Both published PDF archives declare MIT and omit a standalone license text; the
new immutable upstream tree also contains no LICENSE/COPYING/NOTICE file. This
limited license-text coverage is recorded, and does not infer a licensing approval.
Fetched package bytes and focused deltas do not establish a complete source audit.

## Advisory, baseline and policy boundaries

Current advisory evidence is bound to nontruncated RustSec tree
`117edb3bed98e9be112f277b7615eea3252e7c43`. Immutable definitions and
per-scope matching receipts are retained. Known vulnerability/unsound records
matched reviewed frozen closures at patched locked versions; scoped matching
does not prove absence of vulnerabilities or replace candidate cargo-audit.
ttf-parser 0.25.1 retains informational
[RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192.html), with
no patched version. The retained [baseline deny policy](shared/base-deny.toml)
already records `deps-rotation`, REVIEW-BY 2026-12-31 and a rationale to remove
the ignore after lopdf drops it. Preserve that existing disposition and its date.

The retained [roadmap context](shared/historical-roadmap-completion-tranches.txt)
is a byte-preserved text planning snapshot outside the immutable PR base; its
original relative links are source-context text, with no owner acceptance inferred. The retained [F02 graph](shared/f02-baseline-dependency-graph.json) matches the
immutable base lock and manifest digests. Context is default features across
aarch64-apple-darwin, x86_64-apple-darwin, x86_64-pc-windows-msvc and
x86_64-unknown-linux-gnu. It is baseline context, not new resolved graphs at
five candidate heads. Stable three-OS CI does not prove each architecture or the
declared Forge Rust 1.85 minimum.

Old versions have exemptions, not local source audits. This packet cannot
inherit approval of unchanged source from those exemptions. D069 signer/agent
assistance, exception policy and trusted-import decisions remain open; no store
approval is inferred from this review, green checks or an existing exemption.

## Replay and remaining work

From repository root, validate retained bytes offline without Cargo:

```sh
rtk proxy python3 - <<'PY'
import hashlib, json, pathlib
root = pathlib.Path.cwd()
index = json.loads((root / 'docs/dependency-reviews/2026-10-02/retained-evidence-index.json').read_text())
for entry in index['retained_files']:
    data = (root / entry['path']).read_bytes()
    assert len(data) == entry['bytes'], entry['path']
    assert hashlib.sha256(data).hexdigest() == entry['sha256'], entry['path']
print('Verified', len(index['retained_files']), 'retained files')
PY
```

For optional upstream replay, retrieve exact head/base files and CI run/job logs
by the retained immutable identifiers; do not substitute a later PR head or merge
result. Download an archive into a separate temporary directory, verify its
checksum, and inspect only bounded regular members without absolute paths,
traversal, links or special files. Member hashes permit independent comparison.
GitHub comparisons across diverged crypto histories are not the direct old/new
package diff; use verified archives. No installation, source execution or audit
certification is needed to replay these receipts.

Remaining gates: accepted attributable review of the complete shipping
transport/session/crypto set under D069; candidate feature/target graphs and
MSRV verification; direct PDF font-map regression evidence; qualification of
baseline parser/memory observations and existing maintenance dispositions.
The user-required full documentation review/update after completed roadmap
integration remains open. This is an F03 evidence slice, not F03 closure.
