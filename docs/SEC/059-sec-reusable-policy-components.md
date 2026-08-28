# 059-sec-reusable-policy-components

> **Document Type:** Security Review Packet
> **Status:** Draft — security approval required
> **Last Updated:** 2026-08-27
> **Parent PRD:** [059-prd-reusable-policy-components](../PRD/059-prd-reusable-policy-components.md)
> **Architecture:** [059-ar-reusable-policy-components](../AR/059-ar-reusable-policy-components.md)

## Trust boundaries and data

All processing is local and offline. Composition manifests, component sidecars,
Markdown, and optional provenance are untrusted inputs. Assembled Markdown
inherits the sensitivity of every component and supplied value. Locks, provenance,
and impact reports contain identifiers, relative file labels, and hashes; they do
not contain parameter values. They can still reveal policy structure and should
receive the same access control as the composed policy.

## Security and privacy audit

| Risk / requirement | Control | Residual concern |
|--------------------|---------|------------------|
| Traversal / absolute / Windows-drive / ADS paths | lexical rejection plus canonical root prefix | platform filesystem semantics still require hosted Windows/macOS CI |
| Symlink or special-file substitution | every existing path component is inspected; final inputs/outputs must be regular and non-symlink | filesystem races are reduced, not eliminated without descriptor-relative APIs |
| Hard-link or path alias overwrite | existing files compared by device/inode on Unix and canonical identity elsewhere | Windows identity relies on canonical paths in this module |
| Component drift | raw bytes checked against lowercase SHA-256 before render/write | SHA-256 establishes identity, not author trust or approval |
| Template/code execution | one reserved token; no expressions, includes, env, network, or second pass | future grammar expansion requires a new security review |
| Markdown structure injection | single-line values; structural punctuation escaped; unsafe contexts rejected | renderer intentionally supports a narrow text context |
| Secret disclosure | secret-like parameter names warn and fail; value hashes only in artifacts | name matching cannot identify a secret placed under a benign name; user guidance remains necessary |
| Partial output replacement | render/stage first, same-directory backups, atomic renames, rollback | sudden process/host failure is not a journaled multi-file transaction |
| Stale provenance confusion | provenance output hash must match trace source before use | consumers other than FORGE must perform the same check |
| Memory / denial of service | manifest, string, component, depth, parameter, instance, and span bounds | assembled output grows with allowed instances and component sizes |
| Information disclosure in diagnostics | parameter values omitted; bounded names and relative labels used in deterministic artifacts | CLI errors can include caller-supplied filesystem paths, consistent with other local commands |

## No new supply-chain or service exposure

The feature adds no crate, external service, subprocess, network request,
registry, package installation, environment lookup, secret-manager call, or
remote schema resolution. Optional validation calls only FORGE's in-process
existing Catalog pipeline.

## Review gate

Security must approve the accepted substitution contexts and escaping rule,
path/race posture, secret-pattern policy, resource bounds, provenance disclosure,
and coordinated-write residual risk. This draft records implementation evidence;
it is not that approval.
