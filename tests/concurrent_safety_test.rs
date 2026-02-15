//! H-9: Concurrent execution safety verification.
//!
//! Verifies that core pipeline functions are safe to call from multiple threads
//! simultaneously. All global state in forge uses `LazyLock<Regex>` which is
//! thread-safe, but this test explicitly confirms no data races or panics
//! under concurrent load.

use std::sync::Arc;
use std::thread;

use forge::model::{DocumentMetadata, PolicyDocument, PolicyRequirement, PolicySection};

// ─── Helpers ───────────────────────────────────────────────────────────────

fn make_req(text: &str, source_line: usize) -> PolicyRequirement {
    PolicyRequirement {
        stable_id: Some(format!("conc-req-{source_line}")),
        text: text.to_string(),
        source_line,
        nesting_depth: 0,
        atom_index: 0,
        parent_text: None,
        citations: vec![],
    }
}

fn make_req_no_id(text: &str, source_line: usize) -> PolicyRequirement {
    PolicyRequirement {
        stable_id: None,
        text: text.to_string(),
        source_line,
        nesting_depth: 0,
        atom_index: 0,
        parent_text: None,
        citations: vec![],
    }
}

fn make_test_doc(sections: Vec<PolicySection>) -> PolicyDocument {
    PolicyDocument {
        id: "concurrent-test".to_string(),
        metadata: DocumentMetadata {
            title: "Concurrent Test Policy".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            date: None,
            source_path: std::path::PathBuf::from("test.md"),
            content_hash: None,
        },
        sections,
    }
}

fn make_section(title: &str, reqs: Vec<PolicyRequirement>) -> PolicySection {
    PolicySection {
        title: title.to_string(),
        heading_level: 1,
        source_line: 1,
        body_text: None,
        children: vec![],
        requirements: reqs,
    }
}

// ─── Concurrent Tests ──────────────────────────────────────────────────────

/// C-1: Concurrent atomization of different requirements produces correct results.
///
/// Spawns N threads, each atomizing a unique compound requirement. Verifies
/// that all results are independent and correct (no data races on LazyLock<Regex>).
#[test]
fn concurrent_atomization_produces_correct_results() {
    let thread_count = 16;
    let handles: Vec<_> = (0..thread_count)
        .map(|i| {
            thread::spawn(move || {
                let text =
                    format!("System-{i} must enforce control-A and must implement control-B");
                let req = make_req_no_id(&text, i * 10);
                let result = forge::parse::atomize_requirement(&req)
                    .unwrap_or_else(|e| panic!("Thread {i} atomize failed: {e}"));

                assert!(result.was_split, "Thread {i}: should split compound statement");
                assert_eq!(
                    result.requirements.len(),
                    2,
                    "Thread {i}: should produce exactly 2 requirements"
                );
                assert!(
                    result.requirements[0].text.contains(&format!("System-{i}")),
                    "Thread {i}: should preserve subject"
                );
                result
            })
        })
        .collect();

    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.join().unwrap_or_else(|_| panic!("Thread {i} panicked"));
        // Verify each thread produced distinct requirements
        assert_eq!(result.requirements.len(), 2);
    }
}

/// C-2: Concurrent citation extraction produces correct, independent results.
///
/// Verifies LazyLock<Regex> patterns (URL, bibliographic, cross-ref, scheme-less)
/// are safe under concurrent access.
#[test]
fn concurrent_citation_extraction_produces_correct_results() {
    let thread_count = 16;
    let handles: Vec<_> = (0..thread_count)
        .map(|i| {
            thread::spawn(move || {
                let text = format!(
                    "Comply with https://example.com/policy-{i}, \
                     per NIST SP 800-53 Rev 5 and www.standards.org/ref-{i}, \
                     see Section 3.{} for details",
                    i % 10
                );
                let (cleaned, citations) =
                    forge::citation::extract_citations_from_text(&format!("req-{i}"), &text)
                        .unwrap_or_else(|e| panic!("Thread {i} citation failed: {e}"));

                // Should find: 1 URL + 1 bibliographic + 1 scheme-less URL + 1 cross-ref = 4
                assert_eq!(
                    citations.len(),
                    4,
                    "Thread {i}: should extract 4 citations. Got: {}",
                    citations.len()
                );
                assert!(
                    !cleaned.contains("https://"),
                    "Thread {i}: cleaned text should not contain URLs"
                );
                (cleaned, citations)
            })
        })
        .collect();

    for (i, handle) in handles.into_iter().enumerate() {
        let (_, citations) = handle.join().unwrap_or_else(|_| panic!("Thread {i} panicked"));
        assert_eq!(citations.len(), 4);
    }
}

/// C-3: Concurrent document-level operations (atomize_document + assign_stable_ids).
///
/// Tests that full document processing pipelines run correctly in parallel
/// without interference.
#[test]
fn concurrent_document_processing_is_independent() {
    let thread_count = 8;
    let handles: Vec<_> = (0..thread_count)
        .map(|i| {
            thread::spawn(move || {
                let doc = make_test_doc(vec![make_section(
                    &format!("Section-{i}"),
                    vec![
                        make_req_no_id(
                            &format!(
                                "System-{i} must enforce MFA and must require complex passwords"
                            ),
                            i * 100 + 1,
                        ),
                        make_req_no_id(&format!("System-{i} must implement logging"), i * 100 + 2),
                    ],
                )]);

                // Atomize
                let atomized = forge::parse::atomize_document(&doc)
                    .unwrap_or_else(|e| panic!("Thread {i} atomize_document failed: {e}"));

                // Assign IDs
                let with_ids = forge::uuid::assign_stable_ids(atomized);

                // Verify structure
                assert_eq!(with_ids.sections.len(), 1, "Thread {i}: should have 1 section");
                assert_eq!(
                    with_ids.sections[0].requirements.len(),
                    3,
                    "Thread {i}: should have 3 requirements (2 from split + 1 atomic)"
                );

                // Verify all IDs assigned
                for req in &with_ids.sections[0].requirements {
                    assert!(
                        req.stable_id.is_some(),
                        "Thread {i}: all requirements should have stable_id"
                    );
                }

                with_ids
            })
        })
        .collect();

    // Collect results and verify each thread's output is internally consistent
    let results: Vec<_> = handles
        .into_iter()
        .enumerate()
        .map(|(i, h)| h.join().unwrap_or_else(|_| panic!("Thread {i} panicked")))
        .collect();

    // Verify cross-thread: different threads produced different IDs
    // (since input text differs, UUID v5 should produce different IDs)
    let all_ids: Vec<String> = results
        .iter()
        .flat_map(|doc| doc.sections[0].requirements.iter().filter_map(|r| r.stable_id.clone()))
        .collect();

    let unique_count = {
        let mut sorted = all_ids.clone();
        sorted.sort();
        sorted.dedup();
        sorted.len()
    };
    assert_eq!(unique_count, all_ids.len(), "All requirement IDs across threads should be unique");
}

/// C-4: Concurrent citation extraction + document extraction end-to-end.
///
/// Runs the full citation extraction pipeline on complete documents in parallel,
/// verifying that results are correct and independent.
#[test]
fn concurrent_full_citation_pipeline() {
    let thread_count = 8;
    let handles: Vec<_> = (0..thread_count)
        .map(|i| {
            thread::spawn(move || {
                let doc = make_test_doc(vec![make_section(
                    &format!("Compliance-{i}"),
                    vec![
                        make_req(
                            &format!("Comply with https://example.com/policy-{i} requirements"),
                            i * 100 + 1,
                        ),
                        make_req(
                            "Follow NIST SP 800-53 Rev 5 and ISO 27001 standards",
                            i * 100 + 2,
                        ),
                    ],
                )]);

                let result = forge::citation::extract_citations(doc)
                    .unwrap_or_else(|e| panic!("Thread {i} extract_citations failed: {e}"));

                // First requirement: 1 URL citation
                assert_eq!(
                    result.sections[0].requirements[0].citations.len(),
                    1,
                    "Thread {i}: first req should have 1 URL citation"
                );

                // Second requirement: 2 bibliographic citations
                assert_eq!(
                    result.sections[0].requirements[1].citations.len(),
                    2,
                    "Thread {i}: second req should have 2 biblio citations"
                );

                result
            })
        })
        .collect();

    for (i, handle) in handles.into_iter().enumerate() {
        handle.join().unwrap_or_else(|_| panic!("Thread {i} panicked"));
    }
}

/// C-5: Stress test — high concurrency with mixed operations.
///
/// 32 threads running atomization, citation extraction, and ID assignment
/// simultaneously to detect any subtle race conditions.
#[test]
fn stress_test_mixed_concurrent_operations() {
    let barrier = Arc::new(std::sync::Barrier::new(32));
    let handles: Vec<_> = (0..32)
        .map(|i| {
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                // Synchronize all threads to start simultaneously
                barrier.wait();

                match i % 3 {
                    0 => {
                        // Atomization
                        let req = make_req_no_id(&format!("Thread-{i} must do X and must do Y"), i);
                        let result = forge::parse::atomize_requirement(&req).unwrap();
                        assert!(result.was_split);
                    }
                    1 => {
                        // Citation extraction
                        let (_, citations) = forge::citation::extract_citations_from_text(
                            &format!("req-{i}"),
                            &format!("Per https://example.com/{i} and RFC 2119 standards"),
                        )
                        .unwrap();
                        assert_eq!(citations.len(), 2);
                    }
                    _ => {
                        // Full document pipeline
                        let doc = make_test_doc(vec![make_section(
                            "S",
                            vec![make_req_no_id(&format!("System-{i} must enforce controls"), i)],
                        )]);
                        let atomized = forge::parse::atomize_document(&doc).unwrap();
                        let with_ids = forge::uuid::assign_stable_ids(atomized);
                        assert!(with_ids.sections[0].requirements[0].stable_id.is_some());
                    }
                }
            })
        })
        .collect();

    for (i, handle) in handles.into_iter().enumerate() {
        handle.join().unwrap_or_else(|_| panic!("Thread {i} panicked in stress test"));
    }
}
