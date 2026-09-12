//! Offline, local-only suggestion pipeline contracts (PRD-066).
//!
//! Five explicit steps — prepare, run, validate, review, promote — move
//! untrusted model output into a quarantine bundle and, at most, into a
//! proposal a human still has to apply. Every contract here is closed and
//! bounded, and runtime validation is authoritative over the published JSON
//! Schema in `schemas/`.
//!
//! The model boundary is local only. Nothing in this module opens a network
//! connection, resolves a provider, or reads a credential; the adapter is a
//! local child process or a recorded response.

pub mod adapter;
pub mod bundle;
pub mod consent;
pub mod disposition;
pub mod prepare;
pub mod promotion;
pub mod redact;
pub mod request;
pub mod response;
pub mod review;
pub mod run;
pub mod run_record;
mod shared;
pub mod task;
pub mod validate;

pub use bundle::{EvidenceSupport, Suggestion, SuggestionsBundle};
pub use consent::ConsentToken;
pub use disposition::{DispositionManifest, DispositionRecord, DispositionStatus};
pub use promotion::{DestinationKind, PromotionEntry, PromotionProposal, PromotionStatus};
pub use request::{AdapterTarget, ContextUnit, Sensitivity, SourceRef, SuggestRequest};
pub use response::SuggestResponse;
pub use run_record::{RunMode, RunRecord};
pub use task::{Citation, SuggestionBody, TaskIdentity, TaskKind};

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{bundle, consent, disposition, promotion, request, response, run_record, task};

    const SCHEMAS: [(&str, &str); 9] = [
        (
            include_str!("../../schemas/forge.suggest-request-1.schema.json"),
            "https://policy-forge.github.io/schemas/suggest-request/1",
        ),
        (
            include_str!("../../schemas/forge.suggest-consent-1.schema.json"),
            "https://policy-forge.github.io/schemas/suggest-consent/1",
        ),
        (
            include_str!("../../schemas/forge.suggest-task-mapping-1.schema.json"),
            "https://policy-forge.github.io/schemas/suggest-task-mapping/1",
        ),
        (
            include_str!("../../schemas/forge.suggest-task-drafting-1.schema.json"),
            "https://policy-forge.github.io/schemas/suggest-task-drafting/1",
        ),
        (
            include_str!("../../schemas/forge.suggest-response-1.schema.json"),
            "https://policy-forge.github.io/schemas/suggest-response/1",
        ),
        (
            include_str!("../../schemas/forge.suggestions-1.schema.json"),
            "https://policy-forge.github.io/schemas/suggestions/1",
        ),
        (
            include_str!("../../schemas/forge.suggest-dispositions-1.schema.json"),
            "https://policy-forge.github.io/schemas/suggest-dispositions/1",
        ),
        (
            include_str!("../../schemas/forge.suggest-promotion-1.schema.json"),
            "https://policy-forge.github.io/schemas/suggest-promotion/1",
        ),
        (
            include_str!("../../schemas/forge.suggest-run-1.schema.json"),
            "https://policy-forge.github.io/schemas/suggest-run/1",
        ),
    ];

    fn instances() -> Vec<Value> {
        vec![
            request::fixture_request_json(),
            consent::fixture_consent_json(),
            json!({
                "task_schema_version": task::mapping::TASK_SCHEMA_VERSION,
                "mapping_subjects": [task::mapping::fixture_subject_json()],
                "mapping_candidates": [task::mapping::fixture_candidate_json()],
            }),
            json!({
                "task_schema_version": task::drafting::TASK_SCHEMA_VERSION,
                "drafting_sections": [task::drafting::fixture_section_json()],
                "draft_clauses": [task::drafting::fixture_clause_json()],
            }),
            response::fixture_response_json(),
            bundle::fixture_bundle_json(),
            disposition::fixture_dispositions_json(),
            promotion::fixture_promotion_json(),
            run_record::fixture_run_json(),
        ]
    }

    #[test]
    fn every_published_schema_is_compilable_closed_and_versioned() {
        for (source, id) in SCHEMAS {
            let schema: Value = serde_json::from_str(source).expect("published schema is JSON");
            assert_eq!(schema["$id"], id);
            assert_eq!(schema["additionalProperties"], json!(false), "{id}");
            jsonschema::validator_for(&schema)
                .unwrap_or_else(|error| panic!("{id} is not valid JSON Schema: {error}"));
        }
    }

    #[test]
    fn every_runtime_fixture_validates_against_its_published_schema() {
        let cases = SCHEMAS.iter().zip(instances());
        for ((source, id), instance) in cases {
            let schema: Value = serde_json::from_str(source).expect("published schema is JSON");
            let validator = jsonschema::validator_for(&schema)
                .unwrap_or_else(|error| panic!("{id} is not valid JSON Schema: {error}"));
            validator
                .validate(&instance)
                .unwrap_or_else(|error| panic!("{id} rejects its own fixture: {error}"));
        }
    }

    #[test]
    fn published_schemas_reject_an_unknown_field_like_the_runtime() {
        for ((source, id), instance) in SCHEMAS.iter().zip(instances()) {
            let mut closed = instance.clone();
            closed
                .as_object_mut()
                .expect("every fixture is an object")
                .insert("unexpected_key".to_string(), json!(true));
            let schema: Value = serde_json::from_str(source).expect("published schema is JSON");
            let validator = jsonschema::validator_for(&schema)
                .unwrap_or_else(|error| panic!("{id} is not valid JSON Schema: {error}"));
            assert!(
                validator.validate(&closed).is_err(),
                "{id} must reject an unknown top-level field"
            );
        }
    }
}
