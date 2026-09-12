//! Confined, bounded capture of the corpus documents a report consumes.
//!
//! Reuses the authoring capture primitive so corpus files are subject to the
//! same containment, alias, budget and pin rules as every other authoring input.

use std::path::{Path, PathBuf};

use crate::ForgeError;
use crate::authoring::input::CaptureSet;
use crate::hashing::sha256_hex;

use super::corpus::{Corpus, MAX_DOCUMENT_BYTES};

/// One corpus document captured as the exact bytes its pin names.
#[derive(Debug)]
pub struct CapturedDocument {
    /// Stable corpus document key.
    pub key: String,
    /// Portable descendant path the document was captured through.
    pub path: PathBuf,
    /// Exact captured bytes.
    pub bytes: Vec<u8>,
    /// Lowercase hexadecimal SHA-256 of `bytes`.
    pub sha256: String,
}

/// Every corpus document captured beneath one workspace root.
#[derive(Debug)]
pub struct CapturedCorpus {
    /// Absolute, normalized workspace root the documents were resolved against.
    pub root: PathBuf,
    /// Captured documents, in corpus declaration order.
    pub documents: Vec<CapturedDocument>,
}

impl CapturedCorpus {
    /// Reopen every captured document through the same confined primitive.
    ///
    /// Revalidates containment, link and alias rules and re-checks each
    /// captured digest, so a caller can re-verify immediately before
    /// publication. Documents are reopened in capture order.
    ///
    /// # Errors
    ///
    /// Returns [`ForgeError::Authoring`] for the first document that no longer
    /// captures safely or no longer matches its captured digest. The failure
    /// names the document key and never an absolute path.
    pub fn verify(&self) -> Result<(), ForgeError> {
        let mut captures = CaptureSet::new(self.root.clone());
        for document in &self.documents {
            captures
                .read(
                    &role(&document.key),
                    &document.path,
                    Some(&document.sha256),
                    MAX_DOCUMENT_BYTES,
                )
                .map_err(|cause| document_error(&document.key, &cause))?;
        }
        Ok(())
    }
}

/// Capture every corpus document beneath `root`, verifying each exact pin.
///
/// One confined capture covers the whole corpus, so documents are resolved
/// against `root` exactly as authoring resolves its pinned inputs: no symlink,
/// no hard-link alias, no alias between two declared paths, no path that
/// escapes `root`, the same per-file and aggregate byte budgets, and an exact
/// SHA-256 match for every document. Documents are returned in corpus order.
///
/// # Errors
///
/// Returns [`ForgeError::Authoring`] when a document is missing, unsafe,
/// aliased, oversized, over budget, or does not match its `expected_sha256`.
/// Every failure names the document key and never an absolute path.
pub fn capture(root: &Path, corpus: &Corpus) -> Result<CapturedCorpus, ForgeError> {
    let mut captures = CaptureSet::new(root.to_path_buf());
    let mut documents = Vec::with_capacity(corpus.documents().len());
    for document in corpus.documents() {
        let bytes = captures
            .read(
                &role(&document.key),
                &document.path,
                Some(&document.expected_sha256),
                MAX_DOCUMENT_BYTES,
            )
            .map_err(|cause| document_error(&document.key, &cause))?;
        documents.push(CapturedDocument {
            key: document.key.clone(),
            path: document.path.clone(),
            sha256: sha256_hex(&bytes),
            bytes,
        });
    }
    Ok(CapturedCorpus { root: root.to_path_buf(), documents })
}

/// Capture role for one document; role text is diagnostics only.
fn role(key: &str) -> String {
    format!("reuse-document-{key}")
}

/// Name the failing document key; a capture never reports an absolute location.
fn document_error(key: &str, cause: &ForgeError) -> ForgeError {
    error(format!("reuse document '{key}': {cause}"))
}

fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::Authoring(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reuse::corpus::{CORPUS_SCHEMA_VERSION, CorpusDocument, DocumentStatus};
    use serde_json::{Value, json};

    fn document(key: &str, path: &str, bytes: &[u8]) -> CorpusDocument {
        CorpusDocument {
            key: key.to_owned(),
            path: PathBuf::from(path),
            title: format!("Title for {key}"),
            status: DocumentStatus::Approved,
            rights_label: "Repository synthetic fixture".to_owned(),
            source_label: "Synthetic interview".to_owned(),
            expected_sha256: sha256_hex(bytes),
            topic_keys: Vec::new(),
            control_ids: Vec::new(),
            supersedes: None,
        }
    }

    fn corpus(documents: Vec<CorpusDocument>) -> Corpus {
        Corpus {
            schema_version: CORPUS_SCHEMA_VERSION.to_owned(),
            corpus_key: "synthetic-corpus".to_owned(),
            title: None,
            documents,
        }
    }

    fn workspace() -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        (temp, root)
    }

    fn shape(captured: &CapturedCorpus) -> Vec<(String, PathBuf, Vec<u8>, String)> {
        captured
            .documents
            .iter()
            .map(|document| {
                (
                    document.key.clone(),
                    document.path.clone(),
                    document.bytes.clone(),
                    document.sha256.clone(),
                )
            })
            .collect()
    }

    fn declared(access: &[u8], current: &[u8]) -> Value {
        json!({
            "schema_version": CORPUS_SCHEMA_VERSION,
            "corpus_key": "synthetic-corpus",
            "documents": [{
                "key": "access-policy",
                "path": "prior/access.md",
                "title": "Access policy",
                "status": "approved",
                "rights_label": "Repository synthetic fixture",
                "source_label": "Synthetic interview",
                "expected_sha256": sha256_hex(access),
                "topic_keys": ["access-topic"],
                "control_ids": ["ac-1"]
            }, {
                "key": "current-policy",
                "path": "current.md",
                "title": "Current policy",
                "status": "approved",
                "rights_label": "Repository synthetic fixture",
                "source_label": "Synthetic interview",
                "expected_sha256": sha256_hex(current)
            }]
        })
    }

    #[test]
    fn captures_a_parsed_corpus_in_order_with_its_declared_pins() {
        let (_temp, root) = workspace();
        let access = b"# Access\n\nApprove access.\n";
        let current = b"# Current\n";
        std::fs::create_dir(root.join("prior")).unwrap();
        std::fs::write(root.join("prior/access.md"), access).unwrap();
        std::fs::write(root.join("current.md"), current).unwrap();
        let corpus =
            Corpus::parse(&serde_json::to_vec(&declared(access, current)).unwrap()).unwrap();

        let captured = capture(&root, &corpus).unwrap();
        assert_eq!(captured.root, root);
        let keys: Vec<_> = captured.documents.iter().map(|item| item.key.as_str()).collect();
        assert_eq!(keys, ["access-policy", "current-policy"]);
        assert_eq!(captured.documents[0].path, Path::new("prior/access.md"));
        assert_eq!(captured.documents[0].bytes, access);
        assert_eq!(captured.documents[1].bytes, current);
        assert_eq!(captured.documents[0].sha256, sha256_hex(access));
        assert_eq!(captured.documents[0].sha256, corpus.documents()[0].expected_sha256);
        assert_eq!(captured.documents[1].sha256, corpus.documents()[1].expected_sha256);
        captured.verify().unwrap();
    }

    #[test]
    fn capture_is_stable_across_two_calls() {
        let (_temp, root) = workspace();
        std::fs::create_dir(root.join("prior")).unwrap();
        std::fs::write(root.join("prior/access.md"), b"# Access\n").unwrap();
        std::fs::write(root.join("current.md"), b"# Current\n").unwrap();
        let corpus = corpus(vec![
            document("access-policy", "prior/access.md", b"# Access\n"),
            document("current-policy", "current.md", b"# Current\n"),
        ]);

        let first = capture(&root, &corpus).unwrap();
        let second = capture(&root, &corpus).unwrap();
        assert_eq!(first.root, second.root);
        assert_eq!(shape(&first), shape(&second));
        first.verify().unwrap();
        second.verify().unwrap();
    }

    #[test]
    fn capture_rejects_a_wrong_digest() {
        let (_temp, root) = workspace();
        std::fs::write(root.join("policy.md"), b"# Policy\n").unwrap();
        let mut wrong = document("policy", "policy.md", b"# Policy\n");
        wrong.expected_sha256 = "b".repeat(64);
        let failure = capture(&root, &corpus(vec![wrong])).unwrap_err().to_string();
        assert!(failure.contains("reuse document 'policy'"));
        assert!(failure.contains("reuse-document-policy"));
        assert!(!failure.contains(root.to_str().unwrap()));
    }

    #[test]
    fn capture_rejects_a_missing_document() {
        let (_temp, root) = workspace();
        let failure = capture(&root, &corpus(vec![document("absent", "absent.md", b"# Absent\n")]))
            .unwrap_err()
            .to_string();
        assert!(failure.contains("reuse document 'absent'"));
        assert!(!failure.contains(root.to_str().unwrap()));
    }

    #[test]
    fn capture_rejects_a_document_holding_hard_link_aliases() {
        let (_temp, root) = workspace();
        std::fs::write(root.join("policy.md"), b"# Policy\n").unwrap();
        std::fs::hard_link(root.join("policy.md"), root.join("policy-copy.md")).unwrap();
        let failure = capture(&root, &corpus(vec![document("policy", "policy.md", b"# Policy\n")]))
            .unwrap_err()
            .to_string();
        assert!(failure.contains("reuse document 'policy'"));
        assert!(failure.contains("hard-link aliases"));
        assert!(!failure.contains(root.to_str().unwrap()));
    }

    #[cfg(unix)]
    #[test]
    fn capture_rejects_a_symlinked_document() {
        let (_temp, root) = workspace();
        std::fs::write(root.join("real.md"), b"# Real\n").unwrap();
        std::os::unix::fs::symlink(root.join("real.md"), root.join("link.md")).unwrap();
        let failure = capture(&root, &corpus(vec![document("linked", "link.md", b"# Real\n")]))
            .unwrap_err()
            .to_string();
        assert!(failure.contains("reuse document 'linked'"));
        assert!(failure.contains("symbolic links"));
        assert!(!failure.contains(root.to_str().unwrap()));
    }

    #[test]
    fn capture_rejects_paths_escaping_the_root() {
        let (temp, root) = workspace();
        std::fs::write(temp.path().join("outside.md"), b"# Outside\n").unwrap();
        for path in ["../outside.md", "prior/../../outside.md"] {
            let failure = capture(&root, &corpus(vec![document("escapee", path, b"# Outside\n")]))
                .unwrap_err()
                .to_string();
            assert!(failure.contains("reuse document 'escapee'"), "{path}: {failure}");
            assert!(!failure.contains(root.to_str().unwrap()), "{path}: {failure}");
        }
        let absolute = root.join("prior/access.md");
        let failure = capture(
            &root,
            &corpus(vec![document("absolute", absolute.to_str().unwrap(), b"# Absolute\n")]),
        )
        .unwrap_err()
        .to_string();
        assert!(failure.contains("reuse document 'absolute'"));
        assert!(!failure.contains(root.to_str().unwrap()));
    }

    #[test]
    fn capture_rejects_two_documents_aliasing_one_file() {
        let (_temp, root) = workspace();
        std::fs::create_dir(root.join("docs")).unwrap();
        std::fs::write(root.join("docs/policy.md"), b"# Policy\n").unwrap();
        let failure = capture(
            &root,
            &corpus(vec![
                document("policy", "docs/policy.md", b"# Policy\n"),
                document("policy-alias", "docs/Policy.md", b"# Policy\n"),
            ]),
        )
        .unwrap_err()
        .to_string();
        assert!(failure.contains("reuse-document-policy-alias"));
        assert!(failure.contains("aliases another input"));
        assert!(!failure.contains(root.to_str().unwrap()));
    }

    #[test]
    fn capture_rejects_two_documents_aliasing_one_unicode_spelling() {
        let (_temp, root) = workspace();
        let decomposed = "A\u{308}.md";
        std::fs::write(root.join("Ä.md"), b"# Policy\n").unwrap();
        let failure = capture(
            &root,
            &corpus(vec![
                document("policy", "Ä.md", b"# Policy\n"),
                document("policy-decomposed", decomposed, b"# Policy\n"),
            ]),
        )
        .unwrap_err()
        .to_string();
        assert!(failure.contains("reuse-document-policy-decomposed"));
        assert!(!failure.contains(root.to_str().unwrap()));
        // A normalizing filesystem resolves both spellings to one file; either
        // way the second document never captures.
        if root.join(decomposed).exists() {
            assert!(failure.contains("aliases another input file"));
        }
    }

    #[test]
    fn capture_rejects_a_document_over_the_per_file_limit() {
        let (_temp, root) = workspace();
        let oversized = vec![b'a'; usize::try_from(MAX_DOCUMENT_BYTES).unwrap() + 1];
        std::fs::write(root.join("huge.md"), &oversized).unwrap();
        let failure = capture(&root, &corpus(vec![document("huge", "huge.md", &oversized)]))
            .unwrap_err()
            .to_string();
        assert!(failure.contains("reuse document 'huge'"));
        assert!(failure.contains(&MAX_DOCUMENT_BYTES.to_string()));
        assert!(!failure.contains(root.to_str().unwrap()));
    }

    #[test]
    fn capture_rejects_a_corpus_over_the_aggregate_budget() {
        let (_temp, root) = workspace();
        let page = vec![b'a'; 1024 * 1024];
        let documents: Vec<_> = (0..=50u32)
            .map(|index| {
                let key = format!("doc-{index:02}");
                let name = format!("{key}.md");
                std::fs::write(root.join(&name), &page).unwrap();
                document(&key, &name, &page)
            })
            .collect();
        let failure = capture(&root, &corpus(documents)).unwrap_err().to_string();
        assert!(failure.contains("reuse document 'doc-50'"));
        assert!(failure.contains("exhausted the 50 MiB total source budget"));
        assert!(!failure.contains(root.to_str().unwrap()));
    }

    #[test]
    fn verify_detects_a_document_changed_after_capture() {
        let (_temp, root) = workspace();
        std::fs::write(root.join("policy.md"), b"# Policy\n").unwrap();
        let captured =
            capture(&root, &corpus(vec![document("policy", "policy.md", b"# Policy\n")])).unwrap();
        captured.verify().unwrap();

        std::fs::write(root.join("policy.md"), b"# Revised\n").unwrap();
        let failure = captured.verify().unwrap_err().to_string();
        assert!(failure.contains("reuse document 'policy'"));
        assert!(failure.contains("byte pin"));
        assert!(!failure.contains(root.to_str().unwrap()));
    }
}
