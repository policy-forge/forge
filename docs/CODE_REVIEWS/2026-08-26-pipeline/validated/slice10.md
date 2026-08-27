# slice10 — validated verdicts (compact recovery)

Recovered from ValSlice10 scout context 2026-08-26 after provider-quota interruption.
Format: `F####|status|locus|fix directive`. V=valid, P=partial, I=invalid, D=duplicate.
Slice composition: 60 low findings. Original texts: `../all_findings.json` (by id).

Status counts: 55 valid, 5 partial, 0 invalid, 0 duplicate.

```
F0500|V|src/mapping/model.rs:330-333|Aggregate unmapped statement ids into GapSummary or document controls-only
F0499|V|src/mapping/model.rs:610|Hoist gap-summary kind literals into shared consts both sites
F0519|V|src/migration/engine.rs:18-24|Assert or reject duplicate stable_ids when building new_by_id
F0521|V|src/migration/engine.rs:427-437|Key grouping maps by borrowed str slices, not clones
F0522|V|src/migration/engine.rs:434|Replace or_insert_with(Vec::new) with or_default()
F0526|V|src/migration/formatter.rs:20-22|Comment that writeln discards are safe on String sink
F0525|V|src/migration/formatter.rs:67-69|Stream evidence join into buffer without intermediate Vec
F0502|V|src/migration/inventory.rs:16-18|Check input_format before running prepare_document
F0514|V|src/migration/inventory.rs:39-40|Add sorted debug_assert or HashSet-based uniqueness check
F0523|V|src/migration/inventory.rs:55-57|Return UnsupportedFormat variant; align exit code and help text
F0517|V|src/migration/inventory.rs:78|Escape slashes in titles or record structured section paths
F0528|V|src/migration/mod.rs:39|Wrap successor::load errors with map path context
F0529|V|src/migration/mod.rs:6|Privatize successor; re-export only used public items
F0559|V|src/migration/successor.rs:182-183|Deserialize approved_at as DateTime, stop parse-then-discard
F0557|V|src/migration/successor.rs:222-227|Reject ids with surrounding whitespace or control characters
F0558|V|src/migration/successor.rs:92-97|Add path.display() to symlink and regular-file rejections
F0578|V|src/model/assemble.rs:120-124|Use partition_point on source_line-sorted list_items
F0580|V|src/model/assemble.rs:117,169|Document usize::MAX sentinel or model optional end explicitly
F0579|V|src/model/assemble.rs:181-183,220,244|Derive fallback id from content hash, warn on fallback
F0577|V|src/model/assemble.rs:34-75|Document Preamble rule; optionally unify via one helper
F0572|V|src/model/frontmatter.rs:48|Return Absent/Malformed/Data enum instead of Option
F0570|P|src/model/frontmatter.rs:55-64|Strip trailing CR from CRLF-matched YAML slice
F0554|V|src/model/mod.rs:121-124|Add debug-checked enrichment postconditions or typestate wrappers
F0553|P|src/model/mod.rs:99-100|Validate heading_level while accommodating Preamble's level 0
F0547|V|src/model/trace.rs:107-109|Document append-only invariant; consider checked index access
F0546|V|src/model/trace.rs:25-26|Derive PartialEq and Eq on TraceLink
F0548|V|src/model/trace.rs:75-76|Document element-id-only uniqueness in record() docs
F0566|V|src/oscal/assessment_plan.rs:112-113|Fix doc: Assess prefix, 77 chars, ellipsis
F0567|V|src/oscal/assessment_plan.rs:168-173|Model include-all/include-subjects as exclusive enum
F0563|V|src/oscal/assessment_plan.rs:232-233|Preserve assemble_metadata error source in AssessmentPlanBuild
F0565|V|src/oscal/assessment_plan.rs:349-353|Apply empty-text trim guard to activity title/description
F0621|V|src/oscal/back_matter.rs:161-163|Use parsed_url canonical form for href, check snapshots
F0607|P|src/oscal/component_definition.rs:137-160,228-231|Add content hash to component UUID seed inputs
F0603|V|src/oscal/component_definition.rs:263-283|Signal cap truncation; break loops without skipping children
F0604|V|src/oscal/component_definition.rs:275-277|Check seen membership before cloning citation id
F0587|V|src/oscal/implemented_requirements.rs:220|Fix doc: REQ fallback is 1-based global position
F0590|V|src/oscal/implemented_requirements.rs:227-239|Take Option<&str> stable_id instead of has_stable_id bool
F0584|V|src/oscal/mod.rs:46-47|Document or rename duplicate catalog OscalMetadata placeholder
F0596|V|src/oscal/profile.rs:237-247|Fold mode into UUID seed only when ids present
F0600|V|src/oscal/profile.rs:242-243|Sort/dedup control_ids once; reuse, drop clone
F0597|V|src/oscal/profile.rs:242-259|Emit sorted with_ids matching UUID seed order
F0599|P|src/oscal/profile.rs:308-311|Validate tokens cautiously to avoid rejecting legitimate ids
F0598|V|src/oscal/profile.rs:65-66,199|Update doc: href is sanitized filename, not as-is
F0630|V|src/oscal/ssp.rs:222,243,275,421,494|Model enumerated OSCAL states as typed enums
F0626|V|src/oscal/ssp.rs:588-589|Preserve assemble_metadata error source in SspBuild
F0633|V|src/oscal/trace_embedding.rs:40-57|Cap and warn on post-encoding href length
F0636|V|src/oscal/trace_embedding.rs:62-96|Document filename-only precondition or centralize normalization
F0650|V|src/oscal_cli/invoker.rs:25-29,49-54|Log found vs absent allowlisted env vars
F0645|V|src/oscal_cli/invoker.rs:65-70|Read stderr lossy byte-wise instead of read_to_string
F0651|V|src/oscal_cli/invoker.rs:78-103|Document synchronous blocking contract on OscalCliInvoke
F0653|V|src/oscal_cli/mod.rs:37-44|Enforce canonical paths; document positional argument contract
F0654|V|src/oscal_cli/mod.rs:89-90|Drop false default wording or add timeout constant
F0661|V|src/parameter/matchers.rs:208-210|Restrict QUANTITY units to count nouns, update tests
F0662|V|src/parameter/matchers.rs:63-66|Replace expect() with graceful skip on missing groups
F0657|V|src/parameter/mod.rs:211-216|Preserve error source or simplify to non-Result contract
F0711|V|src/parse/atomize.rs:317-322,360-396|Share MAX_SECTION_DEPTH; align counting with atomize cap semantics
F0713|P|src/parse/atomize.rs:68-72|Salt preliminary_id with section context; pipeline overwrites ids
F0712|V|src/parse/atomize.rs:77-80,218,228|Defend char-boundary slicing with get() or debug_assert
F0666|V|src/parse/clauses.rs:211-218|Assert exclude_depth pairing before saturating_sub
F0667|V|src/parse/clauses.rs:402-423|Drop dead Result or add real failure path
```
