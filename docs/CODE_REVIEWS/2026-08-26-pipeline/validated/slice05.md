# slice05 — validated verdicts (compact recovery)

Recovered from ValSlice05 scout context 2026-08-26 after provider-quota interruption.
Format: `F####|status|locus|fix directive`. V=valid, P=partial, I=invalid, D=duplicate.
Slice composition: 60 medium findings. Original texts: `../all_findings.json` (by id).

Status counts: 47 valid, 9 partial, 4 invalid, 3 duplicate.
Notable invalids: F0646/F0647 (tracing format-string claims — edition-2024 captures
interpolate correctly), F0629/F0627 (intentional documented SSP design).

```
F0545|V|src/model/trace.rs:18-19|Change section_title to Option<String>; update EC-4 tests
F0544|V|src/model/trace.rs:81-85|Store link indices in by_requirement; resolve via links
F0562|V|src/oscal/assessment_plan.rs:222-224|Return ForgeError::Validation on empty controls; update ec1_zero_controls test
F0560|V|src/oscal/assessment_plan.rs:235-237|Seed with sanitized href plus length-prefixed control IDs
F0564|V|src/oscal/assessment_plan.rs:454-457|Share subjects instead of cloning per activity
F0622|P|src/oscal/back_matter.rs:173|Reject control-char pseudo-schemes; url crate strips tab/newline already
F0619|V|src/oscal/back_matter.rs:283|Error on duplicate citation id, not silent overwrite
F0612|V|src/oscal/catalog.rs:253-254|Document four-segment collision-suffix control ID form
F0611|P|src/oscal/catalog.rs:358-364|Buffer trace links; commit only on full success
F0610|V|src/oscal/catalog.rs:473-476|Reword docs: deterministic only for fixed section order
F0601|V|src/oscal/component_definition.rs:124-135|Fix doc: _trace_links ignored; wire or correct
F0605|P|src/oscal/component_definition.rs:148-157|Keep source_file Option end-to-end; omit empty prop
F0606|V|src/oscal/component_definition.rs:244-283|Dedup back-matter resources by UUID, not citation id
F0602|V|src/oscal/component_definition.rs:187-189|Preserve back-matter error variant; add boundary context
F0589|V|src/oscal/implemented_requirements.rs:128-129|Seed control-impl UUID from original source_profile path
F0591|P|src/oscal/implemented_requirements.rs:155-157|Replace no-stable-id sentinel with content fingerprint
F0586|P|src/oscal/implemented_requirements.rs:233-238|Fail on missing stable_id like catalog builder
F0588|V|src/oscal/implemented_requirements.rs:92|Return Vec directly or propagate real errors
F0594|V|src/oscal/metadata.rs:56-58|Correct docs: profile.rs uses timestamp_override in production
F0593|V|src/oscal/metadata.rs:70|Fix field count; describe override contract accurately
F0592|P|src/oscal/metadata.rs:95-97|Typed error for empty title/version; update T017
F0583|V|src/oscal/mod.rs:14-15|Add flat re-exports for implemented_requirements, trace_embedding
F0582|V|src/oscal/mod.rs:3-4|Replace hardcoded v1.2.0 doc with OSCAL_VERSION 1.2.3
F0595|V|src/oscal/profile.rs:253-254|Length-prefix seed parts to prevent UUID collisions
F0625|V|src/oscal/ssp.rs:370-373|Build emitted entry list first; compute length
F0629|I|src/oscal/ssp.rs:591-593|Intentional documented title-anchored deterministic UUID design
F0627|I|src/oscal/ssp.rs:674-678|Intentional template placeholder; CLI passes unwrap_or("")
F0628|P|src/oscal/ssp.rs:712-717|Drop dead serde(skip) system_id assignment at build_ssp
F0623|V|src/oscal/test_utils.rs:13-18|Panic on non-string remarks; fail loud
F0634|V|src/oscal/trace_embedding.rs:120-123|Emit warn on unknown-file fallback path
F0635|V|src/oscal/trace_embedding.rs:133-135|Debug-log when group children span sections
F0632|V|src/oscal/trace_embedding.rs:55|Percent-encode all RFC 3986 reserved path characters
F0637|V|src/oscal_cli/detector.rs:119-122|Bound version check with timeout; kill child
F0642|V|src/oscal_cli/detector.rs:136-145|Require semver-shaped token; empty output means failed
F0641|V|src/oscal_cli/detector.rs:37-40|Surface configured path and canonicalize error detail
F0640|V|src/oscal_cli/detector.rs:56-61|Preserve version-check failure detail for users
F0639|V|src/oscal_cli/detector.rs:86-91|Gate candidates on is_file; restrict extensions
F0646|I|src/oscal_cli/invoker.rs:106|Edition-2024 implicit capture interpolates context correctly
F0647|I|src/oscal_cli/invoker.rs:114|Edition-2024 implicit capture interpolates context correctly
F0648|V|src/oscal_cli/invoker.rs:159-160|Stream-sniff root key instead of full parse
F0649|V|src/oscal_cli/invoker.rs:163-172|Surface parse errors instead of collapsing via ok()
F0644|V|src/oscal_cli/invoker.rs:77-83|Capture and warn partial stderr on timeout
F0652|V|src/oscal_cli/mod.rs:17-20|Model detection outcome as enum, validated constructor
F0658|V|src/parameter/matchers.rs:102-105|Prepend \b to qualifier alternatives both thresholds
F0660|V|src/parameter/matchers.rs:169|Guard left edge against semi-/bi- compound matches
F0659|V|src/parameter/matchers.rs:95-97|Constrain value suffix; reject hyphenated token absorption
F0655|V|src/parameter/mod.rs:201-203|Track processed state explicitly; add placeholder tripwire
F0656|V|src/parameter/mod.rs:205-209|Extract oscal_base_id helper; reject duplicate param ids
F0709|P|src/parse/atomize.rs:138-152|Clone input requirement; preserve citations/modality/parameters
F0710|V|src/parse/atomize.rs:203-205|Derive subject from first split boundary position
F0664|V|src/parse/clauses.rs:211-214|Allow-list paragraph scope instead of deny-list
F0665|V|src/parse/clauses.rs:376-379|Handle HardBreak alongside SoftBreak in paragraph/table
F0705|V|src/parse/mod.rs:143-149|Push space on SoftBreak/HardBreak inside headings
F0684|V|src/parse/modality.rs:143-146|Drop requirement text from WARN records; SEC-1
F0688|D-of-F0684|src/parse/modality.rs:143-146|Identical warn-text leak as F0684
F0728|V|src/pipeline.rs:142-151|Extract shared finalize/serialize/AP helpers both pipelines
F0731|V|src/pipeline.rs:238-244|Validate assessment plan or fail until supported
F0732|P|src/pipeline.rs:328-330|Fall back to unknown-file, never full path
F0733|D-of-F0731|src/pipeline.rs:397-403|Same unvalidated assessment-plan gap; fix with F0731
F0726|V|src/pipeline.rs:55-60|Include bounded validation error details in message
```
