# Workspace index bundle inspection

Unreleased API contract **1.2.0** supplies complete explicit-index metadata
preview and registered fingerprint comparison. The maintained Python client
exposes `bundle_preview()` and `verify_bundle(bundle)`. The **Trace & Reports**
browser panel also previews metadata, requires sensitivity acknowledgment before
a local JSON download, and compares a chosen file's original bytes. Both queries
are available in read-only sessions; this panel does not publish a project
bundle, register supplied paths or import data.

## Inspect and compare

Use an existing project containing `forge.workspace.json` and registered resources that the workspace can capture safely. Run this example from the repository root with a build containing API 1.2.0:

```sh
python3 - <<'PYTHON'
import json
from scripts.workspace_client import Workspace

with Workspace("./target/debug/forge", "./example", read_only=True) as client:
    preview = client.bundle_preview()
    comparison = client.verify_bundle(preview["bundle"])
    print(json.dumps({
        "state": comparison["state"],
        "expected_resources": comparison["expected_resources"],
        "matched_resources": comparison["matched_resources"],
        "current_only_resources": comparison["current_only_resources"],
        "expected_index_matches_current": comparison["expected_index_matches_current"],
    }, indent=2))
PYTHON
```

The client starts a read-only machine session, keeps its capability in memory and closes the session when the context ends. Each call captures current state again, so a comparison can differ from the preceding preview if project bytes changed. Inspect the returned bundle in memory before choosing whether to retain or share its metadata. The example prints only comparison state and counts. It creates no exported file, write preview, receipt, registration or import.

The operations are `GET /api/v1/project/bundle-preview` (`getProjectBundlePreview`) and `POST /api/v1/project/bundle-verifications` (`verifyProjectBundle`). The POST body is exactly `{"bundle": ...}`. Both require the ordinary session capability and are available in read-only sessions. They accept no selection, cursor or idempotency key. Browser clients retain the existing Origin, Fetch Metadata and JSON requirements.

## Browser preview, download and comparison

In **Trace & Reports**, use **Preview metadata** to inspect every registered
resource key, role, project-relative path, SHA-256 fingerprint and byte length.
This read is available in writable and read-only sessions. Labels, paths, keys
and stable hashes can reveal project information even though source content is
excluded. Acknowledge that sensitivity before **Download metadata bundle**, which
requests a local `forge-workspace-index-and-hashes.json` file. It creates no
project file, server publication or receipt. A fresh preview or view/session
retirement clears acknowledgment for the old preview.

To compare an existing file, use **Choose a metadata bundle JSON file**, then
**Compare registered fingerprints**. The file may contain at most **1,048,565
bytes**: its original bytes and fixed 11-byte JSON wrapper must fit the 1 MiB
request bound. The browser sends those bytes without decoding, stripping a BOM,
removing duplicate keys or rewriting JSON. The server validates the input and
compares only current registered captures; supplied unregistered paths are not
opened. An explicit retry captures current state again.

Queries preserve complete denominators through the existing 1,000-registration
and byte bounds, with no 100-input preparation prefix. Missing versus explicitly
empty current index, expected fingerprint agreement versus current-only
registrations, whole-index equality and observed valid/stale/invalid content
remain distinct. Matching hashes or the local sensitivity acknowledgment
establish neither domain approval nor import readiness or later byte stability.

The local download contains the observed bundle alone, encoded as compact UTF-8
JSON within 1 MiB. Its file hash describes those downloaded bytes. That encoding
is separate from the normalized index hash and original resource fingerprints
described below. No source content or server receipt is added.

## What the bundle contains

`forge.workspace-index-bundle/1` with content profile `index-and-hashes` contains the existing closed `forge.workspace/1` index, its `index_sha256`, and one ordered pin per index resource. Each pin supplies the exact resource key, lowercase SHA-256 and byte length. Pins and index entries have the same order and keys, with no missing, duplicate or extra pin.

The preview covers the complete explicitly registered inventory. It returns the project label, resource keys, typed roles, project-relative paths, fingerprints and byte lengths. Labels, keys and paths can reveal names; stable hashes can support correlation. Handle that metadata according to the project's sharing rules. Source content, credentials, absolute project roots, timestamps and cached approval/validation assertions are outside this bundle profile. `source_content_included` is false.

### Two hash inputs

Resource `sha256` and `size_bytes` describe each resource's **original captured bytes**. Whitespace and line-ending changes alter that fingerprint. `index_sha256` describes **normalized index bytes**, rather than the original formatting of `forge.workspace.json`.

The normalized index algorithm is the existing `Index::bytes()` encoding:

1. Serialize the validated index fields in order: `schema_version`, `label`, `resources`. Serialize each resource's fields in order: `key`, `role`, `path`.
2. Preserve the authored resource array order and all accepted string values. Roles use their existing lowercase kebab-case wire names.
3. Use the `serde_json` pretty encoder with two-space indentation, UTF-8 and ordinary JSON escaping for quotes, backslashes and control characters. Non-ASCII characters remain UTF-8; no sorting or Unicode normalization is added.
4. Append exactly one LF byte after the closing object, then compute lowercase SHA-256 over the entire encoding, including that LF.

Changing the label, resource order or registration metadata changes the normalized index hash. Reformatting an otherwise identical source index does not. Preserve this exact encoding when independently reproducing the hash; hashing arbitrary JSON serialization or the source index file will give a different result. `snapshot_version` describes the observed capture, including local identity effects; it is not a portable identity or a guarantee of later byte stability.

## Interpret the comparison

Verification resolves each expected entry by exact **key, role and project-relative path**, then compares its fingerprint and byte length with the current registered capture. Supplying an unregistered path does not cause that path to be opened. The workspace still performs its ordinary registered-resource and domain-dependency capture; the supplied bundle adds no file-discovery or unregistered-read authority.

| Item status | Meaning | Reason codes |
|---|---|---|
| `matched` | Registration, SHA-256 and size agree | Empty |
| `not-registered` | No current registration has the supplied key | `registration-not-found` |
| `mismatched` | The current key has another role/path, or its bytes differ | `registration-conflict`, or `sha256-mismatch` and/or `size-mismatch` |

The response retains one item per expected index entry, in supplied order. The complete denominator is:

`matched_resources + unregistered_resources + mismatched_resources = expected_resources = items.length = bundle.index.resources.length`.

`current_resources` describes the current registration count. `current_only_resources` counts current keys absent from the expected index. Extra current registrations stay in place and do not prevent every supplied expected entry from matching. `expected_index_matches_current` separately compares the normalized **whole index**, including its label and array order. A matching expected subset can therefore have `state: matched`, extra current resources and whole-index equality false.

| Current index | Expected bundle | Result |
|---|---|---|
| Absent | Any intrinsically valid bundle | `missing-index` takes priority; source-index-present is false, expected entries are `not-registered`, and whole-index equality is false |
| Explicitly present and empty | Empty index with the same label | `matched`, all counts zero, whole-index equality true |
| Explicitly present | Every expected fingerprint agrees | `matched`; extras and whole-index equality remain separate |
| Explicitly present | Any expected entry is absent, conflicts or differs | `mismatched`, with complete item results |

GET on an absent index returns the existing safe 404 setup error; it does not invent a bundle from the default empty setup value. An explicit empty index is valid. Labels still participate in whole-index equality, including for empty indexes.

Each item also reports `observed_resource_validation_state`: `valid`, `stale`, `invalid` or `not-registered`. It describes current workspace domain metadata separately from fingerprint agreement. A byte-matching historical report can remain `stale`, and invalid registered content can still have matching fingerprints. Comparison establishes no domain approval, currentness, import eligibility or permission to publish.

## Bounds and errors

| Boundary | Limit |
|---|---:|
| Existing index encoding | 1 MiB including its final LF |
| Encoded bundle | 1 MiB |
| Browser local metadata download | 1 MiB |
| Browser selected comparison file | 1,048,565 bytes (plus 11-byte wrapper) |
| Entire raw POST JSON body | 1 MiB, including `bundle`, braces, escapes and whitespace |
| Registered/index/pin/item count | 1,000 |
| Each resource and each declared pin size | 10 MiB |
| Current snapshot capture | 50 MiB, including the original captured index bytes |
| Intrinsic bundle declared aggregate | 50 MiB, including normalized index bytes plus all declared resource sizes |
| API response | 4 MiB |
| JSON nesting / structural separators | 64 levels / 100,000 separators |

General request/bundle strings are bounded at 64 KiB; the nested existing index parser additionally limits strings to 4 KiB and retains its narrower field/path rules. The independent bundle and request limits both apply: a bundle near 1 MiB can exceed the POST envelope limit after wrapping or re-encoding. The Python client's general 14 MiB guard does not increase this route's 1 MiB server limit.

Within these byte and capture limits, 101 or 1,000 registrations are complete query inventories. The existing 100-consumed-input effect limit does not apply because these operations prepare no effect. If a complete bundle or response exceeds a limit, the request fails explicitly; no 100-entry prefix or other successful subset is substituted.

Malformed/unsupported bundle or request shape, pin-order/bijection errors and an incorrect normalized index hash return the existing safe `invalid-request` error. A declared pin size above 10 MiB also returns `invalid-request` (400). Lexical paths rejected by the API schema return 400 before the index parser; portable device names or trailing-dot segments that pass that pattern then violate intrinsic containment and return `resource-containment` (403). Oversized requests/encodings and a checked declared aggregate above 50 MiB return `payload-too-large` (413). A structurally valid bundle with registered fingerprint mismatches instead returns the complete comparison result. Missing or unsafe current registered files follow existing snapshot errors. Retrying captures new current state; there is no replay receipt or promise that earlier results stay unchanged.

## Remaining workflow and acceptance

This metadata consumer supplies browser preview, an acknowledged local metadata
download and registered comparison. Receipt-backed server export/publication,
source-content export, confirmed writable import, reviewed batch input binding
and retention/capacity qualification remain open. Full PRD 062 S-6 remains open.
The 100-consumed-input effect cap and existing dependency/approval boundaries
are unchanged. API v1 and bundle1 retain their seven-role closed contract.

All six PRD 062 Should-Have acceptance gates, security/privacy, supported-platform
and interoperability, human keyboard/assistive-technology/WCAG, pilot/release and
human acceptance remain open where recorded. The scoped
[browser metadata verification](workspace-bundle-browser-verification.md) is
development evidence. The final full integrated documentation review/update
remains open.

## Explicit API v2

With `--api-major 2` (or `Workspace(..., api_major=2)`), the same queries use
`/api/v2`. A current index1 produces bundle1; a current index2 produces
`forge.workspace-index-bundle/2` paired with `forge.workspace/2` and fifteen roles.
The server, maintained client and UI reject mismatched version pairs. V2 can
compare a supplied bundle1 with current index2 registrations while retaining
separate expected fingerprints, current-only counts and whole-index equality.
API v1 accepts only index1 and bundle1. The five metadata fields, complete ordered
pin bijection, normalized hashing, sensitivity acknowledgment, download/POST/capture
bounds and registered-only comparison remain the same. New-role admission profiles
grant no freshness or approval. See [v2 migration guidance](api/migration-v2.md).

API2 contract 2.2.0 separately offers [confirmed metadata export and complete index replacement](workspace-bundle-receipts.md). Those acknowledged preparations require a writable session and an exact receipt; they use their own whole-effect bounds. These existing queries retain their read-only semantics and do not restore source contents. Full source-content and multi-file import remain required S-6 work.
