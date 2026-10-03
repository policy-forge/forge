# Select workspace API v2

API v2 adds explicit lifecycle and framework-impact registrations, versioned
metadata bundles and captured read views. For a build publishing contract 2.2.0,
it also adds acknowledged metadata export and complete index-replacement
preparation. It is selected once at launch. API v1 remains the default, with its
original seven roles and contract 1.2.0. These are unreleased implementations;
contract 2.2.0 does not announce a product release. The initial 2.0.0 foundation is
retained in PR #193, and the 2.1.0 captured inspection delivery in PR #194, with
their historical versions and verification receipts.

## Launch and negotiate

```sh
forge workspace --project ./example --api-major 2
forge workspace --project ./example --api-major 2 --read-only --no-open
forge workspace --project ./example --api-major 2 --machine-session --read-only
python3 scripts/workspace_client.py --forge ./target/debug/forge --project ./example --api-major 2
```

A launch serves only its selected `/api/v1/` or `/api/v2/` namespace. Restart to
select another major; credentials and retained previews belong to the original
session. For a build publishing 2.2.0, API v2 reports `api_major: 2` and
`api_version: "2.2.0"` in the sensitive machine descriptor, and the same major and exact `contract_version` in Session.
Keep the descriptor and capability out of logs. The bundled shell and maintained
client verify this negotiation before project operations. Foreign-major paths
return a typed 404 before authorization, replay, resource reads or effects.

Both majors retain loopback-only binding, exact Host checks, browser Origin and
Fetch Metadata checks, scoped capabilities, read-only enforcement, confinement,
bounded capture and explicit preview/confirmation. API v1 support is retained
for at least one stable FORGE minor release, with migration guidance before
removal, under the [compatibility policy](compatibility.md).

## Choose the index version explicitly

API v2 reads `forge.workspace/1` and `forge.workspace/2`; launching never rewrites
the index. API v1 accepts only index1 and rejects index2 before startup credentials
or listening, and before subsequent registered-resource captures.

In **Policies & Artifacts**, select a role and project-relative file path. The
**Index version for this registration** control defaults to preserving the
current version. Choose **Explicitly use workspace index /2** to create index2 or
combine its migration with a registration. **Preview migration to workspace index
/2** migrates an existing index1 without registering another resource.

Each action prepares an ordinary `workspace-index-update` preview. Inspect its
target, exact proposed bytes/hash, inputs and validation, then confirm that exact
preview. Upload remains a separate confirmed write followed by registration.
Cancelling or merely launching changes no project file. Repeated activation while
preparation is pending sends no duplicate request and keeps the trigger focused.

For scripted preparation, use the same selected-major operation:

```python
from scripts.workspace_client import Workspace

with Workspace("./target/debug/forge", "./example", read_only=False, api_major=2) as client:
    prepared = client.request("POST", client.api_path("/resources/register"), {
        "role": "lifecycle-source", "path": "evidence/source.bin", "key": "source-pin",
        "index_schema_version": "forge.workspace/2",
    }, idempotency_key="register-source-example")
    preview = prepared["preview"]
    # Inspect the exact preview before a separate explicit commit.
    # Retain the original request body and key to recover a lost preparation reply.
```

The migration-only body is
`{"migration":{"from":"forge.workspace/1","to":"forge.workspace/2"}}`.
Omitting `index_schema_version` preserves the current index format; new roles
require current index2 or the explicit selector before the unregistered file is
read. Omitted or null registration keys retain their original behavior. Mixing
migration and registration alternatives, null selectors, or unsupported versions
returns 400. Migration with no index or an already-index2 project returns 422.
Changed index or source bytes invalidate a preview with the usual 409 conflict;
re-preview before confirming. Read-only sessions cannot migrate or register.

## Interpret registration admission

Index2 keeps the original seven roles and adds eight closed role values:

| Role | Admission profile | What admission establishes |
|---|---|---|
| `lifecycle-record` | `lifecycle-record-structure` | Intrinsic lifecycle record structure |
| `lifecycle-source` | `opaque-fingerprint-bytes` | Exact bounded bytes, including empty or non-UTF-8 content |
| `oscal-profile-artifact`, `oscal-ssp-artifact` | `native-oscal-schema` | Native model schema validity |
| `framework-impact-manifest` | `framework-impact-manifest` | Intrinsic manifest structure |
| `successor-map` | `successor-map` | Intrinsic successor-map structure |
| `framework-impact-report` | `framework-impact-prior-admission` | Limited prior-report header, object and finding-ID admission |
| `framework-impact-dispositions` | `framework-impact-dispositions` | Intrinsic disposition structure |

Resource responses expose the applicable optional `validation_profile`. A valid
registration does not establish referenced dependency closure, current domain
computation, freshness, reviewer identity or human approval. A prior impact report
is not validated as a complete native/current report. Opaque lifecycle source
registration grants no source excerpt access. The captured lifecycle and impact reads perform their own complete dependency and domain checks; see [the inspection guide](../workspace-lifecycle-impact.md).

## Inspect version-paired bundles

API v2 previews bundle1 for index1 and bundle2 for index2. Both retain exactly the
metadata profile `index-and-hashes`, normalized index hash and ordered complete
resource pins. Bundle1 requires index1 and its original seven roles; bundle2
requires index2 and its fifteen roles. Mixed version pairs fail admission.

The Trace & Reports panel and `bundle_preview()` / `verify_bundle(bundle)` consume
these pairs. Both queries work in read-only sessions. API v2 can compare a supplied
bundle1 against current index2 registrations: expected fingerprints, current-only
resources and whole-index equality remain separate facts. Supplied unregistered
paths are never opened. These read queries disclose no source content and grant
no import authority.

See [the metadata query guide](../workspace-index-bundles.md) for local download,
normalized hashing and query bounds. Contract 2.2.0 separately introduces the
[confirmed metadata and index-replacement workflow](../workspace-bundle-receipts.md):
acknowledged export preparation, authenticated committed JSON download and a
direct 200 index-replacement preview. Only subsequent explicit confirmation writes
the export target or the complete index. Numeric `target_index_schema_version`
1 or 2 selects the replacement format; migration from index1 to index2 is explicit,
and downgrade is refused. No source files are restored or deleted.

Read-only sessions retain preview/comparison queries but cannot prepare these
writes. The write prerequisite uses a complete 100-file/50 MiB consumed union,
including the present raw index, current registrations, incoming files and any
distinct existing destination, plus the existing conservative 20 MiB retention
cap. It does not increase the 1,000-registration query limit or truncate an
over-cap write to a prefix. Full source-content and multi-file S-6, full S-3
acceptance, native accessibility, platform, security, human acceptance and final
integrated documentation review remain open where recorded.

## Contract and packaging artifacts

The [v2 OpenAPI document](forge-workspace-v2.openapi.yaml),
[v2 capability matrix](capability-matrix-v2.md), [v2 fixtures](fixtures-v2/README.md)
and [index2 schema](../../schemas/forge.workspace-2.schema.json) are independently
versioned alongside the unchanged v1 family. API1/1.2.0 declares 39 operations.
API2/2.2.0 declares 51: the initial 39, nine captured read queries introduced in
2.1.0, and three metadata receipt operations introduced in 2.2.0. The maintained
client preserves matching numeric API-major-2 descriptor/Session bootstrap
negotiation. The inspection methods and navigation explicitly admit 2.1.0 or
2.2.0; the new metadata write/download surfaces require exactly 2.2.0. An accepted
future major-2 bootstrap version alone does not grant either feature surface.
Historical 2.0.0/39 and 2.1.0/48 receipts remain bound to those contracts.

[release-artifacts.json](release-artifacts.json) declares both API families and
the asset paths required in binary archives. Offline contract tests check their
existence, versions, fixtures and matrix relationships. Release packaging stages
current assets outside the cached build tree. A nonpublishing archive rehearsal
checks membership and bytes; it does not establish a published release, supported
Windows runtime or final signed release provenance.
