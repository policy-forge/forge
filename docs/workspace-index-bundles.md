# Workspace index bundle inspection

Unreleased API contract **1.2.0** adds a read-only preview of the explicit workspace index and a comparison of supplied fingerprints with currently registered resources. The maintained Python client exposes `bundle_preview()` and `verify_bundle(bundle)`. This slice supplies API/headless inspection; the bundled browser has no bundle action yet.

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

This metadata inspection has no browser action, bundle file publication, source-content export or confirmed writable import. Full PRD 062 S-6 still needs the browser export/preview and explicit confirmed import workflow, reviewed batch input binding and publication/retention capacity. The effect input cap is unchanged. No new index role, archive format, schema relaxation, dependency or approval policy is introduced.

All six PRD 062 Should-Have acceptance gates, security/privacy, real browser/keyboard/assistive-technology, supported-platform/interoperability, pilot/release and human acceptance remain open where recorded. Focused API/client tests and fixture inventories do not establish those outcomes. The user's final full integrated documentation review/update remains open.
