# 059-ar-reusable-policy-components

> **Document Type:** Architecture Review
> **Status:** Proposed — human approval required
> **Last Updated:** 2026-08-27
> **Parent PRD:** [059-prd-reusable-policy-components](../PRD/059-prd-reusable-policy-components.md)

## Proposed decision

Use two duplicate-key-safe, closed JSON contracts and a flat composition graph.
Component sidecars are adjacent to their UTF-8 Markdown source; composition
manifests declare a project root, ordered stable instances, and three output
paths. All local inputs are lexically checked, symlink-walked, canonicalized,
prefix-checked, and compared by file identity. Raw source bytes are SHA-256
pinned before rendering.

Substitution recognizes only `{{forge:param:<ascii-kebab-name>}}`. Values are
strongly typed, validated, Markdown-escaped, inserted once, and never reparsed.
The output provenance model uses half-open, one-based line/character spans.
Literal spans identify the component source bytes and line/columns; parameter
spans additionally identify the declaration name and value hash. Values never
appear in locks or provenance.

The three outputs are fully rendered and staged before existing destinations
move to same-directory backups. Replacement uses atomic renames with rollback
on a failed replacement. This is a coordinated local filesystem transaction,
not a cross-filesystem or crash-recovery protocol.

## Contract choices requiring approval

| Decision | Proposed contract | Rationale |
|----------|-------------------|-----------|
| Component source base | Sidecar directory, still bounded by composition project root | Makes standalone checking and adjacent sidecars portable |
| Span coordinates | One-based Unicode scalar columns, half-open end | Stable for UTF-8 and unambiguous for substitutions |
| Accepted substitution contexts | Paragraph/list text only; headings, code, links/URLs, and HTML rejected | Avoids Markdown structure injection in MVP |
| Escaping | Backslash-escape Markdown structural punctuation | Preserves data without a second template/Markdown interpretation pass |
| Parameter hashing | SHA-256 of canonical compact typed JSON | Distinguishes strings, numbers, booleans, and lists without disclosure |
| Conversion validation | Optional temporary-file pass through existing Catalog pipeline | Reuses production ingestion/conversion without changing composition bytes |

## Requirement audit

| Requirement | Implementation evidence | Executable evidence |
|-------------|-------------------------|--------------------|
| M-1 | `policy compose`, `policy component check`, `policy compose check` CLI tree | `policy_components_cli_test` read/write tests |
| M-2 | `manifest.rs`, strict JSON parser, `deny_unknown_fields`, schema constants | duplicate/unknown/version unit tests |
| M-3 | `resolve_root`, `resolve_input_from`, `resolve_output`, identity registry | traversal unit tests; symlink/output-alias CLI tests |
| M-4 | raw SHA-256 checked before render | pin-drift no-replacement test |
| M-5 | flat structs; unknown include/recursive keys rejected; reserved grammar closed | component contract and placeholder grammar unit tests |
| M-6 | generated H1, required component H2, component H1 rejection, LF separators | composition/heading tests |
| M-7 | typed values plus type-specific constraints/defaults | valid four-type fixture and invalid type/range tests |
| M-8 | context scanner, `escape_markdown`, one render pass | fenced-context and placeholder-as-data tests |
| M-9 | declaration/value/placeholder cross-checks | missing, unknown, unused, duplicate tests |
| M-10 | unique instance keys; exact sidecar reuse permitted | duplicate component fixture and duplicate-instance test |
| M-11 | staged outputs, backups, atomic rename, collision checks | staged-failure unit test and alias CLI test |
| M-12 | generated/component/parameter span origins; values represented only by hashes | provenance completeness invariant test |
| M-13 | ordered vectors/BTree maps, LF normalization, no time/env/absolute path | repeated and cross-directory byte comparison tests |
| M-14 | `--validate` calls existing Catalog pipeline on temporary assembled bytes | end-to-end compose with `--validate` |
| M-15 | secret-like names warn and fail; no interpolation source exists | sensitive-name unit test and documentation |
| M-16 | unit and CLI suites cover requested adversarial classes | `cargo test policy`; `policy_components_cli_test` |
| S-1/S-2 | explicit-manifest `component impact` index/report; read-only pins | drifted impact report test verifies lock unchanged |
| S-3 | optional `trace --composition-provenance` hash-validates and appends origins | OSCAL conversion/trace bridge test |
| S-4 | adjacent, hash-pinned, always-draft `component scaffold` | scaffold and subsequent check test |

## Compatibility and scope

No new dependency, canonical OSCAL type, schema registry, existing ID seed, or
default conversion output changes. Existing trace output is unchanged unless
`--composition-provenance` is supplied. Component lifecycle status is preserved
but not authenticated or transitioned. Remote access, nesting, code execution,
environment interpolation, conditional logic, secret retrieval, approval, and
coverage claims remain out of scope.

## Approval gate

Engineering must approve the manifest fields, source-base rule, span coordinates,
escaping/context restriction, typed hash contract, and coordinated-write boundary
before this proposed architecture is considered accepted.
