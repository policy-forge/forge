# F19 prerequisite: workspace index bundle queries

This slice adds two authenticated read-only API consumers and maintained Python client methods for the existing explicit resource-index workflow. It previews the complete metadata/fingerprint inventory and compares a supplied bundle only with current registered captures. It is a partial PRD 062 S-6 prerequisite; full browser export and confirmed writable import remain open.

Root-provided tracking: parent #23 (ID 1418), child #24 (ID 1419), branch `codex/f19-workspace-bundle-queries`. The implementation is stacked on PR177/F12 base `5a12daad1f31f33827cde5b50ad6f0915d9e12ec`. The attached draft manifest records observed working-tree SHA/byte pins, not an immutable final candidate. This plan records no executed test, coverage, hook, CI, release or human acceptance result.

## Frozen scope

- Additive unreleased API contract 1.2.0: `GET /api/v1/project/bundle-preview` / `getProjectBundlePreview`, and `POST /api/v1/project/bundle-verifications` / `verifyProjectBundle`.
- Closed `forge.workspace-index-bundle/1`, fixed `index-and-hashes` profile; the unchanged `forge.workspace/1` index plus normalized index SHA-256 and one original-resource-byte pin per registration in authored order.
- Read-only production HTTP query dispatch and `Workspace.bundle_preview()` / `Workspace.verify_bundle(bundle)`. No effect, receipt, idempotency replay, destination, publication or browser action.
- Existing intrinsic index/path rules, complete expected counters, missing-index priority, exact key/role/path comparison, current-only keys and whole-index equality. Fingerprints and observed valid/stale/invalid domain state remain separate.
- No file read attributable to a supplied unregistered path. Ordinary snapshot registered-resource/dependency capture retains its existing authority and limits.
- Labels, keys, paths and stable hashes remain sensitive metadata. This profile has no source content, absolute root, token, approval assertion or timestamp.

The [bundle guide](../workspace-index-bundles.md) specifies the field-order/two-space UTF-8 JSON/final-LF normalized index hash, original resource-byte hashes, absence versus explicit empty state, comparison counts and errors. All requests and replies remain governed by the normative OpenAPI, capability matrix, fixture index and existing runtime validator. The nested OpenAPI index is a composed mirror of the standalone index schema, not a separate bundle validator or relaxed index contract.

## Verification required before delivery

Executed local test, docstring and coverage evidence is recorded separately in the [bundle verification packet](../workspace-bundle-verification.md). Its measured scope and remaining gates do not change the preparation record above.

The current artifact inventory has 13 added schema fixture entries and 53 fixture entries in total. Those are file counts, not executed tests. Root will bind authentic verification and adjacent documentation/production coverage to final source bytes before the delivery packet/PR.

Meaningful checks must exercise actual authenticated HTTP query dispatch and the maintained client in a read-only machine session; preserve original index/source bytes; reconcile all ordered expected items at 0, 101 and 1,000 registrations; reject 1,001 and oversized encodings; distinguish absent versus present-empty indexes; demonstrate fresh retry captures; and retain the no-effect/no-new-unregistered-read boundary. Mixed match/conflict/unregistered/hash/size cases, extra current keys, label/order-only index differences and matching stale/invalid metadata need exact results rather than aggregate success alone.

Reject decoded duplicate keys, unknown content/approval fields, wrong version/profile/hash encoding, pin order/count/key errors, bad normalized hashes, unsafe paths, raw request-envelope/depth/separator excess and declared/captured byte excess. Keep bearer/Host/Origin/Fetch Metadata/JSON rules, reject undeclared query/idempotency headers and preserve read-only rejection for register/upload/commit. Verify the composed index schema against its standalone source and confirm every new helper is consumed in production without dead-code allowances. Root owns Cargo, actual client/server runs, final named source/doc inventories, exact production LCOV/cfg(test) scope, mandatory hook and hosted evidence.

Bounds remain 1 MiB normalized index, 1 MiB bundle, 1 MiB entire POST wrapper, 1,000 resources, 10 MiB each resource, 50 MiB captured inputs and 50 MiB declared bundle aggregate including normalized index, 4 MiB response, depth 64 and 100,000 structural separators. General strings are 64 KiB and existing nested index strings 4 KiB. Query inventories use no 100-input effect subset. The future writable import still requires reviewed batch binding and retention/publication capacity; neither cap is relaxed here.

## Open gates

S-1 static reports, S-2 guided authoring, S-3 lifecycle/impact views, S-4 long operations, S-5 launcher and S-6 full bundle export/import retain their complete acceptance gates. This statement preserves gate status and does not mean all earlier implementation is absent. S-6 browser export/metadata preview, confirmed writable import and source-content opt-in/capacity remain separate work.

Security/privacy and authority decisions, independent browser/keyboard/AT/WCAG work, supported-platform/interoperability, authentic pilot outcomes, release/public API compatibility and human acceptance remain open. The user's final full integrated documentation review/update remains open. No new dependency, OSCAL schema asset, index role, signing/identity decision or domain approval is introduced. Historical F12 and earlier receipts remain unchanged; future executed evidence must name its actual source and outcome.
