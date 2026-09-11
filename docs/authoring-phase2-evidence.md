# PRD-061 Phase 2 acceptance evidence

This record maps every row of the
[Phase 2 interface freeze and acceptance matrix](authoring-phase2-plan.md#acceptance-matrix)
to named executable cases merged into `main`. It is technical evidence only:
public Rust API/semver, product, compliance, legal/content, design-partner,
readiness, pilot and release gates remain open (see
[authoring gates](authoring-gates.md)).

## Verification commands (2026-09-11)

| Command | Observed result |
|---|---|
| `cargo fmt --check` | Clean |
| `cargo clippy --all-targets -- -D warnings` | Clean |
| `cargo test --locked` | 2,337 passed; 0 failed; 3 ignored (58 suites) |
| `git diff --check` | Clean |

The merged Phase 2 candidate recorded 2,331 locked tests (three ignored). This
closeout adds six tests for S-1 only: three `src/authoring/scaffold.rs` unit
cases and three `tests/authoring_cli_test.rs` scaffold cases. No frozen `/1` or
`/2` contract semantics changed.

Test names below use `tests/<file>::<name>` for integration tests and
`src/authoring/<file>.rs mod tests::<name>` for unit tests.

## Acceptance matrix → executable cases

| Matrix row | Test module/file | Exact test function names | Coverage notes |
|---|---|---|---|
| Legacy behavior | `tests/authoring_cli_test.rs`, `tests/authoring_phase2_cli_test.rs`, `src/authoring/output.rs` | `complete_gap_accounting_and_dependent_only_blocking`, `supplied_answer_is_hash_only_in_reports_and_never_interpolated`, `invalid_stale_expired_and_explicit_no_answer_remain_local_blockers`, `age_expiry_and_optional_unanswered_context_are_explicit`, `optional_context_pinned_by_a_clause_uses_a_truthful_blocking_marker`, `exact_source_fingerprint_mismatches_fail_without_output`, `pinned_fabricated_and_filtered_reports_are_rejected`, `assignment_and_deferral_conflicts_are_rejected`, `checked_in_synthetic_example_keeps_its_exact_hash_chain_and_published_contracts`, `repeated_and_cross_directory_builds_have_identical_complete_bytes`, `assignments_without_a_selected_policy_remain_unresolved`, `multiple_topics_and_families_do_not_double_count_gaps`, `all_accounted_gaps_and_available_context_exit_zero`, `generated_reports_use_only_authoring_outcome_terminology`; `output.rs mod tests::identical_generations_have_identical_bytes_across_directories`; phase2 `opt_in_components_html_exact_provenance_and_cross_directory_bytes`, `handoff_is_explicit_exact_draft_only_self_contained_and_no_overwrite`, `plan_only_html_uses_validated_legacy_and_component_reports_without_build_artifacts` | Existing Phase 1 CLI suite unchanged; exact repeated/cross-directory artifacts compare `first`/`second` trees and the checked-in `examples/authoring/*` hash chain; opt-in cases assert no new default artifacts. |
| Closed contracts | `src/authoring/manifest.rs`, `components.rs`, `handoff.rs`, `html.rs`, `impact.rs` | `manifest.rs::adversarial_and_forward_version_fixtures_fail_closed`, `duplicate_decoded_key_fixture_reports_the_duplicate_key`, `paths_reject_cross_platform_ambiguity_before_io`, `valid_contract_fixtures_also_validate_against_published_schemas`, `published_schemas_reject_invalid_text_and_timestamp_spellings`, `standalone_schemas_have_only_reachable_and_consistent_shared_definitions`, `schema_timestamp_guard_preserves_runtime_spellings_and_precision`, `malformed_json_bounds_and_canonical_identifier_rules_are_enforced`, `constraints_are_typed_bounded_and_cannot_supply_defaults`; `components.rs::components_closed_schema_rejects_forward_unknown_duplicate_null_and_graph_aliases`, `literal_binding_variant_rejects_unknown_fields_and_null_sensitivity`, `generated_newline_provenance_obeys_remaining_budget_before_push`; `handoff.rs::closed_manifest_rejects_forward_duplicate_unknown_null_and_implicit_governance`, `schema_and_runtime_agree_on_closed_governance_and_identity_contracts`, `output_budget_is_enforced_during_serialization`; `html.rs::raw_values_and_unsupported_or_duplicate_contracts_are_rejected`, `escaping_stops_before_allocating_beyond_the_limit`, `imported_report_strings_keep_the_upstream_limit_and_remaining_byte_budget`; `impact.rs::closed_contract_schema_and_runtime_reject_forward_null_unknown_and_duplicate_keys`, `impact_schema_and_runtime_reject_control_character_path_and_hash_suffixes`, `impact_utf8_byte_bounds_remain_distinct_from_schema_character_preflight`, `snapshot_roles_must_be_distinct_but_old_and_new_may_repeat_exact_pins`, `effective_impact_output_budgets_are_reported_and_leave_writer_unchanged`; `plan.rs::evidence_expansion_is_bounded_before_repeated_cloning`; `impact.rs::no_gap_edges_cannot_consume_budget_reserved_for_later_component_edges`, `scope_records_reserve_remaining_budget_before_allocation` | Schema/runtime agreement and unknown/decoded-duplicate/null/forward-version rejection are paired; bounded arrays, strings and dependency graphs are exercised. Fixtures live in `tests/fixtures/authoring/contracts/`. |
| Components | `src/authoring/components.rs`, `tests/authoring_phase2_cli_test.rs` | `components.rs::component_source_and_sidecar_drift_fail_even_when_context_is_blocked`, `component_defaults_never_satisfy_missing_explicit_context`, `missing_stale_invalid_expired_and_no_answer_block_only_the_selected_section`, `component_parameter_constraints_block_invalid_answers_without_literal_fallback`, `changed_answer_pin_blocks_only_dependent_section_and_preserves_both_hashes`, `answer_key_bound_to_the_wrong_question_is_an_invalid_contract`, `protected_values_and_secret_names_fail_before_rendering`, `repeated_instances_reuse_exact_capture_and_preserve_separate_identity`, `ordered_span_cursor_handles_long_unicode_lines_zero_width_and_crlf_exactly`, `fragment_provenance_partitions_utf8_and_retains_exact_source_and_answer_pins`, `topic_heading_uses_phase1_punctuation_escaping_exactly`, `one_pass_substitution_retains_literal_placeholder_text_and_empty_value_provenance`, `component_sources_reject_unsafe_markdown_unknown_grammar_and_outline_repair`, `component_paths_reject_nonportable_alias_spellings_and_binding_bounds`; phase2 `opt_in_components_html_exact_provenance_and_cross_directory_bytes`, `component_defaults_never_fill_missing_explicit_bindings_and_blocked_sources_still_validate`, `optional_unresolved_marker_stays_after_its_component_heading`, `component_extension_reformat_is_visible_and_drift_is_unverified`, `plan_only_html_uses_validated_legacy_and_component_reports_without_build_artifacts` | One case per named sub-clause: source/sidecar drift, blocked-section drift, explicit defaults, missing/stale/invalid/expired/no-answer dependencies, confidential and secret names, repeated instances, Unicode, one-pass substitution, unsupported Markdown and heading mismatch. |
| Provenance | `tests/authoring_cli_test.rs mod provenance_contract`, `tests/authoring_phase2_cli_test.rs`, `src/authoring/render.rs`, `components.rs`, `plan.rs` | `provenance_contract::published_provenance_binds_every_output_byte_to_complete_input_and_human_evidence`, `supplied_answer_is_hash_only_in_reports_and_never_interpolated`, `imported_applicability_manifest_above_two_mib_keeps_its_exact_byte_pin`, `nested_applicability_manifest_can_reference_its_contained_parent_framework`, `nested_baseline_dependencies_keep_portable_labels_on_every_platform`; phase2 `opt_in_components_html_exact_provenance_and_cross_directory_bytes`, `impact_html_is_deterministic_across_directories_and_keeps_answer_values_private`, `outside_component_pins_are_classified_consistently_across_snapshot_locations`; `render.rs::every_output_byte_has_exact_origin_and_clause_bytes_are_unchanged`, `repeated_render_is_byte_identical_and_contains_no_generated_verdict`, `readable_unicode_clauses_preserve_joiners_and_exact_source_bytes`, `blocked_section_omits_clause_while_unrelated_section_renders`; `components.rs::fragment_provenance_partitions_utf8_and_retains_exact_source_and_answer_pins`, `repeated_instances_reuse_exact_capture_and_preserve_separate_identity`, `generated_newline_provenance_obeys_remaining_budget_before_push`; `plan.rs::explicit_human_clause_yields_draft_state_without_disclosing_answer_values` | Exact contiguous UTF-8 partition (`span.output.end` chain equals `byte_length`), source/parameter/generated-newline spans, complete input hash set, repeated-instance distinction, and no raw-value disclosure. |
| M-13 | `src/authoring/impact.rs`, `tests/authoring_phase2_cli_test.rs` | `impact.rs::no_op_has_identical_bytes_dependencies_and_no_findings`, `report_hash_churn_retains_gap_correspondence_and_exact_old_new_ids`, `imported_long_control_supports_no_op_report_churn_and_gap_membership_changes`, `unrelated_frameworks_cannot_match_identical_control_ids_or_uuid`, `reviewed_exact_correspondence_targets_only_changed_control_dependencies`, `empty_and_partial_correspondence_cannot_hide_surviving_control_changes`, `optional_exact_framework_correspondence_requires_complete_surviving_coverage`, `reviewed_partial_pairs_allow_true_absent_side_control_additions_and_removals`, `gap_addition_and_removal_use_applicability_membership_not_report_hash`, `imported_long_control_additions_and_removals_keep_authored_correspondence_bounds`, `assignment_review_and_deferral_changes_preserve_reasons`, `answer_value_changes_are_private_and_do_not_affect_unrelated_sections`, `answer_review_changes_are_provenance_only`, `review_time_without_expiry_dependency_is_only_provenance`, `stale_and_expired_answers_report_state_and_block_only_dependencies`, `definition_and_explicit_expiry_changes_have_distinct_categories`, `unused_question_change_keeps_unrelated_content_and_bytes_unaffected`, `pack_reviewer_metadata_is_global_provenance_not_substantive_content`, `human_clause_changes_are_exact_and_targeted`, `policy_addition_removal_and_empty_policies_have_truthful_results`, `moved_component_marks_old_and_new_destinations`, `explicit_control_topic_dependencies_remain_visible_without_current_gaps`, `component_changes_bind_instance_identity_and_preserve_provenance_axis`, `component_expiry_block_and_unblock_do_not_invent_substantive_changes`, `component_extension_reformatting_is_global_provenance_with_unaffected_local_content`, `incomplete_drift_comparison_never_claims_unaffected_or_reflects_errors`; phase2 `impact_noop_independent_directories_report_churn_and_unverified_inputs`, `component_extension_reformat_is_visible_and_drift_is_unverified`, `outside_component_pins_are_classified_consistently_across_snapshot_locations`, `failed_old_capture_does_not_refund_unknown_bytes_to_the_new_snapshot` | One case per named category: no-op, report reformat, unrelated frameworks, reviewed correspondence, added/removed gaps, assignments/deferrals, answer value/definition/expiry, unused question, pack, component content/binding/drift, human clause, policy additions/removals, moved dependencies, unaffected sections. M-13's checkbox is technical evidence only. |
| HTML | `src/authoring/html.rs`, `tests/authoring_phase2_cli_test.rs` | `html.rs::every_label_is_text_and_unicode_and_offline_bytes_are_deterministic`, `raw_values_and_unsupported_or_duplicate_contracts_are_rejected`, `escaping_stops_before_allocating_beyond_the_limit`, `imported_report_strings_keep_the_upstream_limit_and_remaining_byte_budget`; phase2 `plan_only_html_uses_validated_legacy_and_component_reports_without_build_artifacts`, `impact_html_is_deterministic_across_directories_and_keeps_answer_values_private`, `outside_component_pins_are_classified_consistently_across_snapshot_locations`, `html_views_accept_validated_upstream_control_ids_beyond_authored_input_string_limits`, `opt_in_components_html_exact_provenance_and_cross_directory_bytes` | Injection across title, rationale, filename and component metadata; Unicode; bounded expansion; no active content or raw values; HTML carries the same validated report data as JSON. See partial-coverage note. |
| Handoff | `src/authoring/handoff.rs`, `tests/authoring_phase2_cli_test.rs` | `handoff.rs::exact_bytes_and_provenance_and_complete_unique_selection_are_required`, `closed_manifest_rejects_forward_duplicate_unknown_null_and_implicit_governance`, `schema_and_runtime_agree_on_closed_governance_and_identity_contracts`, `records_are_deterministic_drafts_with_exact_retained_sources_and_receipt`, `aliases_and_duplicate_lifecycle_identity_are_rejected`, `output_budget_is_enforced_during_serialization`, `published_records_pass_normal_lifecycle_check_and_existing_history_is_preserved`; phase2 `handoff_is_explicit_exact_draft_only_self_contained_and_no_overwrite`, `handoff_tampering_any_policy_rejects_whole_set`, `handoff_publishes_complete_drafts_with_exit_one_when_drafting_work_remains` | Exact-byte tampering, supplied identity/dates/parties/approval rules, deterministic two-policy draft records, empty history, normal `lifecycle check` acceptance, retained source references, unchanged existing records. |
| Safe I/O | `tests/authoring_cli_test.rs`, `tests/authoring_phase2_cli_test.rs`, `src/authoring/input.rs`, `output.rs`, `manifest.rs`, `components.rs`, `handoff.rs` | phase1 `symlink_and_hard_link_inputs_and_output_parents_are_rejected`, `canceled_symlink_dependency_is_rejected_even_when_the_external_bytes_match`, `overwrite_and_stale_source_preserve_existing_complete_generation`, `invalid_output_paths_and_input_aliases_fail_closed`, `noncanonical_baseline_dependencies_fail_even_with_a_real_matching_report`, `windows_manifest_root_aliases_fail_before_input_reads`; phase2 `component_symlink_hardlink_and_traversal_fail_before_publication`, `cli_components_paths_and_html_without_destination_are_rejected`, `handoff_is_explicit_exact_draft_only_self_contained_and_no_overwrite`; `input.rs::captures_reject_case_collisions_before_opening_an_alias`, `captured_identity_rejects_unicode_aliases_on_normalizing_filesystems`, `baseline_dependencies_reject_noncanonical_base_directory_spellings`, `baseline_dependencies_reject_non_utf8_without_lossy_conversion`, `remaining_source_budget_limits_each_read_before_capture`, `exhausted_source_budget_is_reported_before_reading_another_input`, `restricting_below_captured_bytes_reports_the_effective_reduced_budget`; `output.rs::failures_before_rename_leave_no_destination_or_staging_data`, `destination_is_complete_at_the_only_publication_point`, `process_interruption_exposes_either_no_generation_or_the_complete_generation`, `interrupted_publication_child`, `existing_empty_directory_and_concurrent_destination_are_never_replaced`, `rejects_symlink_parent_and_destination_without_following_them`, `rejects_portable_path_aliases_and_file_directory_collisions`, `unsupported_platform_fails_without_creating_output`; `manifest.rs::paths_reject_cross_platform_ambiguity_before_io`; `components.rs::component_paths_reject_nonportable_alias_spellings_and_binding_bounds`; `handoff.rs::aliases_and_duplicate_lifecycle_identity_are_rejected` | Symlink/hard-link/case/Unicode aliases, traversal, overwrite, aggregate budget, interruption before/after the single publish, and multi-output failure atomicity. See partial-coverage note for missing-parent. |
| Delivery | Cargo/CI; no Rust test module | — | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --locked`, `git diff --check`; independent complete-diff review, exact-commit OCR and exact-head CI/thread disposition are process artifacts recorded on the PR, not tests. |

## Partial coverage and non-test evidence

- **HTML injection is representative, not exhaustive.** `every_label_is_text_and_unicode_and_offline_bytes_are_deterministic`
  injects into title, rationale, filename and component metadata and asserts the
  absence of active elements; CLI cases assert no `<script` in emitted views.
  There is no field-by-field sweep of every label/path/metadata surface, because
  the views consume bounded validated types whose labels are already escaped by
  one shared encoder.
- **Safe I/O missing parent** is folded into
  `invalid_output_paths_and_input_aliases_fail_closed` (a non-existent parent
  destination exits 2) rather than a dedicated named case. The S-1 scaffold adds
  a direct no-replace publication case below.
- **Platform-gated cases:** `unsupported_directory_publication_fails_before_any_output`
  and `output.rs::unsupported_platform_fails_without_creating_output` are
  `#[cfg(not(any(target_os = "linux", target_os = "macos")))]`;
  `windows_manifest_root_aliases_fail_before_input_reads` is `#[cfg(windows)]`;
  symlink/hard-link cases are Unix-only. Authoring publication fails closed off
  Linux/macOS.
- **No matrix row lacks real coverage.** Every row maps to executable cases; the
  two notes above are depth caveats, not absent tests.

## S-1 scaffold evidence

`forge author scaffold --manifest <forge.author-project/1>` writes an empty
`forge.authoring-pack/1` template to the project's pinned `authoring_pack` path.
It captures and exactly regenerates the pinned framework inventory, binds the
project baseline, creates no topics/questions/families/assignments, and emits the
required reviewer provenance fields empty. The output is intentionally not a
usable pack until a human supplies `reviewers`, `content_rights` and the drafting
records; no approval evidence or lifecycle state is created.

| Case | Purpose |
|---|---|
| `src/authoring/scaffold.rs mod tests::empty_pack_binds_the_project_baseline_and_creates_no_records` | Baseline/key binding; empty collections and empty reviewer provenance |
| `src/authoring/scaffold.rs mod tests::rendered_scaffold_is_deterministic_and_closed` | Byte determinism; exactly the closed schema field set |
| `src/authoring/scaffold.rs mod tests::rendered_scaffold_requires_reviewer_provenance_before_use` | Intentional fail-until-reviewed contract |
| `tests/authoring_cli_test.rs::scaffold_creates_an_empty_pack_without_assignments_or_reviewer_provenance` | CLI end-to-end, determinism, existing destination refused |
| `tests/authoring_cli_test.rs::scaffold_requires_an_exactly_valid_framework_inventory` | Stale report pin fails with exit 2 and writes nothing |
| `tests/authoring_cli_test.rs::scaffold_never_follows_a_symlinked_destination` | No-follow, atomic no-replace publication |

S-1 adds a public `AuthorCommand` variant, so it feeds the pending public
Rust API/semver and migration gate.

## S-4 shared-gap evidence

M-4 already admits "one or more policy topics" per gap, and the pack allows one
topic to belong to multiple policy families (`validate_pack` dedupes only the
exact `(control_id, topic_key)` and `(topic_key, policy_family_key)` pairs).
`plan.rs::build_gaps` materializes every (control, topic, family) combination
while gap counts stay per-gap. No new code was needed; the existing
`tests/authoring_cli_test.rs::multiple_topics_and_families_do_not_double_count_gaps`
case now also asserts that one gap's assignments carry both
`access-policy` and `operations-policy` and that both policies list the gap in
their sections, proving multiple policy families share one gap with per-edge
responsibility boundaries preserved.

## Must-Have requirement evidence

Every PRD-061 Must Have maps to named executable cases merged on `main`. Where a
requirement spans both phases, the Phase 2 cases are included; M-13's full
comparison matrix is the M-13 row above. This mapping satisfies the
Definition-of-Ready item "Every Must Have maps to an executable acceptance test"
(technical evidence only).

| ID | Requirement | Executable cases (representative) |
|---|---|---|
| M-1 | `forge author plan` / `build` commands with text/JSON reports and output-directory controls | `complete_gap_accounting_and_dependent_only_blocking`, `all_accounted_gaps_and_available_context_exit_zero`, `repeated_and_cross_directory_builds_have_identical_complete_bytes`, `checked_in_synthetic_example_keeps_its_exact_hash_chain_and_published_contracts`, `generated_reports_use_only_authoring_outcome_terminology` (text), `plan_only_html_uses_validated_legacy_and_component_reports_without_build_artifacts` |
| M-2 | Closed bounded inputs; reject unknown/duplicate/unsupported | `manifest.rs::valid_contract_fixtures_also_validate_against_published_schemas`, `duplicate_decoded_key_fixture_reports_the_duplicate_key`, `adversarial_and_forward_version_fixtures_fail_closed`, `malformed_json_bounds_and_canonical_identifier_rules_are_enforced`, `constraints_are_typed_bounded_and_cannot_supply_defaults`, `components.rs::components_closed_schema_rejects_forward_unknown_duplicate_null_and_graph_aliases` |
| M-3 | Exact PRD-056 baseline binding | `exact_source_fingerprint_mismatches_fail_without_output`, `pinned_fabricated_and_filtered_reports_are_rejected`, `noncanonical_baseline_dependencies_fail_even_with_a_real_matching_report`, `plan.rs::filtered_and_duplicate_baselines_are_rejected`, `plan.rs::captured_fingerprints_must_match_the_exact_project_pins` |
| M-4 | Gap accounting reconciles (assigned / deferred / unresolved) | `complete_gap_accounting_and_dependent_only_blocking`, `assignments_without_a_selected_policy_remain_unresolved`, `multiple_topics_and_families_do_not_double_count_gaps`, `all_accounted_gaps_and_available_context_exit_zero`, `assignment_and_deferral_conflicts_are_rejected`, `plan.rs::one_gap_with_multiple_destinations_is_counted_once`, `plan.rs::gaps_without_selected_policies_remain_unresolved_and_deferrals_are_explicit` |
| M-5 | Reviewer/time/rationale provenance on assignments | `complete_gap_accounting_and_dependent_only_blocking` (assignment review binding), `manifest.rs::hashes_are_domain_separated_and_bind_complete_records`, `manifest.rs::cross_references_deferrals_and_exact_answer_pins_fail_closed`, `plan.rs::missing_context_blocks_only_dependent_sections_and_preserves_assignment_review`, `manifest.rs::source_labels_allow_human_punctuation_and_reject_absolute_local_paths` |
| M-6 | Bounded typed questions with required/owner/sensitivity/expiry/constraints | `manifest.rs::constraints_are_typed_bounded_and_cannot_supply_defaults`, `missing_stale_expired_invalid_and_no_answer_are_explicit`, `provided_answers_require_supported_values_and_no_answer_requires_absence`, `explicit_empty_values_are_distinct_from_no_answer_when_constraints_allow_them`, `text_contracts_reject_blank_format_sequences_and_preserve_unicode_joiners` |
| M-7 | Block only dependent sections; no hidden substantive defaults | `invalid_stale_expired_and_explicit_no_answer_remain_local_blockers`, `plan.rs::missing_context_blocks_only_dependent_sections_and_preserves_assignment_review`, `plan.rs::invalid_stale_and_expired_answers_block_only_their_section`, `plan.rs::optional_unavailable_context_is_visible_and_only_explicit_clause_dependency_blocks`, `supplied_answer_is_hash_only_in_reports_and_never_interpolated`, `age_expiry_and_optional_unanswered_context_are_explicit`, `components.rs::component_defaults_never_satisfy_missing_explicit_context` |
| M-8 | Deterministic Markdown skeletons with visible unresolved markers | `render.rs::repeated_render_is_byte_identical_and_contains_no_generated_verdict`, `render.rs::every_output_byte_has_exact_origin_and_clause_bytes_are_unchanged`, `render.rs::blocked_section_omits_clause_while_unrelated_section_renders`, `render.rs::readable_unicode_clauses_preserve_joiners_and_exact_source_bytes`, `optional_unresolved_marker_stays_after_its_component_heading` |
| M-9 | Only supplied clauses or hash-pinned components; never synthesized prose | `render.rs::clause_grammar_rejects_outline_changes_html_fences_and_template_syntax`, `render.rs::clauses_reject_whitespace_and_invisible_format_characters_alone`, `invalid_pinned_clause_grammar_fails_plan_and_build_before_output`, `changed_clause_answer_pin_fails_plan_and_build_without_output`; component branch: `components.rs::component_sources_reject_unsafe_markdown_unknown_grammar_and_outline_repair`, `opt_in_components_html_exact_provenance_and_cross_directory_bytes` |
| M-10 | Machine-readable provenance graph to every input hash | `provenance_contract::published_provenance_binds_every_output_byte_to_complete_input_and_human_evidence`, `render.rs::every_output_byte_has_exact_origin_and_clause_bytes_are_unchanged`, `components.rs::fragment_provenance_partitions_utf8_and_retains_exact_source_and_answer_pins`, `imported_applicability_manifest_above_two_mib_keeps_its_exact_byte_pin` |
| M-11 | Safe output: containment, alias rejection, atomic writes, bounds, no absolute paths | `symlink_and_hard_link_inputs_and_output_parents_are_rejected`, `invalid_output_paths_and_input_aliases_fail_closed`, `overwrite_and_stale_source_preserve_existing_complete_generation`, `output.rs::failures_before_rename_leave_no_destination_or_staging_data`, `output.rs::rejects_symlink_parent_and_destination_without_following_them`, `input.rs::captures_reject_case_collisions_before_opening_an_alias`, `manifest.rs::paths_reject_cross_platform_ambiguity_before_io` |
| M-12 | No wall-clock/locale/location in identity or outputs | `repeated_and_cross_directory_builds_have_identical_complete_bytes`, `plan.rs::semantic_ordering_and_repeated_reports_are_stable`, `output.rs::identical_generations_have_identical_bytes_across_directories`, `checked_in_synthetic_example_keeps_its_exact_hash_chain_and_published_contracts` |
| M-13 | Baseline/dependency impact distinction | See the M-13 acceptance row above: `impact.rs` full comparison matrix plus `impact_noop_independent_directories_report_churn_and_unverified_inputs`, `component_extension_reformat_is_visible_and_drift_is_unverified`, `outside_component_pins_are_classified_consistently_across_snapshot_locations`, `failed_old_capture_does_not_refund_unknown_bytes_to_the_new_snapshot`. M-13's checkbox records this technical evidence, not human acceptance. |
| M-14 | Never emit compliant/certified/implemented/effective/approved from mere completeness | `generated_reports_use_only_authoring_outcome_terminology`, `render.rs::repeated_render_is_byte_identical_and_contains_no_generated_verdict`, `html.rs::every_label_is_text_and_unicode_and_offline_bytes_are_deterministic` |
| M-15 | Tests cover gap reconciliation, missing context, stale packs, component drift, provenance, safe paths, deterministic rebuilds | The union above plus the Safe I/O and Components rows: `exact_source_fingerprint_mismatches_fail_without_output`, `components.rs::component_source_and_sidecar_drift_fail_even_when_context_is_blocked`, `provenance_contract::published_provenance_binds_every_output_byte_to_complete_input_and_human_evidence`, `repeated_and_cross_directory_builds_have_identical_complete_bytes` |

The names are representative per requirement; the full per-row map is the
acceptance matrix above, and the exact source of record is the test suite itself.

## Review follow-up verification

Every actionable follow-up in [Phase 2 review dispositions](authoring-phase2-review.md)
is landed; the remaining entries are explicitly deferred optional refactors.

| Item | Verdict |
|---|---|
| Direct extension-schema links in the authoring guide | Landed: `docs/authoring.md` links component, impact and handoff schemas |
| Changelog link to the complete Phase 2 contract table plus a `Changed` entry | Landed: `CHANGELOG.md` links `docs/authoring-phase2.md` and records the `AuthorCommand` change; publication-novelty and public/internal substitution text is in the `Added` entry |
| Impact schema end anchors and byte/cardinality documentation | Landed in `schemas/authoring-impact.schema.json` |
| CLI help descriptions, handoff exit-1 documentation, PRD revision row | Landed |
| Pure renderer/caller boundary and adapter capture notes | Landed in `docs/authoring-phase2.md` |
| Product/compliance gates retained in both roadmap views | Landed |
| Constructor and positional-budget refactors | Deferred by design (optional); tests protect effective bounds and bytes |
| Component failure-phase `Option::map`, qualified imported-string constant, extra Unicode fixture assertion, HTML DOM streaming guard | Deferred by design (optional/non-finding) |
| Comment-anchor probe, component heading override, `ErrorKind` formatting, duplicate component index, repeated HTML serialize/parse advice | Non-findings / duplicates, no source defect |
