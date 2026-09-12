//! `forge author reuse` end-to-end contracts: closed report, exit codes, spans.

#[cfg(unix)]
use std::process::Command;

use serde_json::{Value, json};
mod common;
use common::{assert_exit, corpus, project, reuse};

#[test]
fn review_corpus_serialization_stays_in_its_own_contract() {
    let (_temp, root) = project();
    corpus(&root, "# Access\n\nc-1 text\n");
    let parsed =
        forge::reuse::corpus::Corpus::parse(&std::fs::read(root.join("corpus.json")).unwrap())
            .unwrap();
    let serialized = serde_json::to_vec(&parsed).unwrap();
    let roundtrip = forge::reuse::corpus::Corpus::parse(&serialized);
    assert!(roundtrip.is_ok(), "a parsed Corpus serializes to rejected JSON: {roundtrip:?}");
}

#[test]
fn review_candidate_pointers_are_bound_to_captured_inputs() {
    let (_temp, root) = project();
    corpus(&root, "# Access\n\nc-1 text\n");
    let output = reuse(&root, &["--format", "json"]);
    assert_exit(&output, 0);
    let good: Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut accepted = Vec::new();
    for (name, path, replacement) in [
        ("outside path", "/sections/0/candidates/0/source_path", json!("../../outside.md")),
        ("wrong source hash", "/sections/0/candidates/0/source_sha256", json!("b".repeat(64))),
        ("span beyond EOF", "/sections/0/candidates/0/span/end", json!(1_048_577)),
        (
            "empty span",
            "/sections/0/candidates/0/span/end",
            good["sections"][0]["candidates"][0]["span"]["start"].clone(),
        ),
        ("missing source", "/sections/0/candidates/0/source_key", json!("undeclared")),
    ] {
        let mut value = good.clone();
        *value.pointer_mut(path).unwrap() = replacement;
        if forge::reuse::report::ReuseReport::parse(&serde_json::to_vec(&value).unwrap()).is_ok() {
            accepted.push(name);
        }
    }
    assert!(accepted.is_empty(), "accepted invalid candidate pointers: {accepted:?}");
}

#[cfg(unix)]
#[test]
fn review_symlinked_corpus_manifest_is_rejected() {
    let (_temp, root) = project();
    corpus(&root, "# Access\n\nc-1 text\n");
    std::fs::rename(root.join("corpus.json"), root.join("real-corpus.json")).unwrap();
    std::os::unix::fs::symlink("real-corpus.json", root.join("corpus.json")).unwrap();
    assert_exit(&reuse(&root, &["--format", "json"]), 2);
}

#[cfg(unix)]
#[test]
fn review_fifo_corpus_is_rejected_without_blocking() {
    use std::time::{Duration, Instant};
    let (_temp, root) = project();
    assert!(Command::new("mkfifo").arg(root.join("corpus.json")).status().unwrap().success());
    let mut child = Command::new(env!("CARGO_BIN_EXE_forge"))
        .current_dir(&root)
        .args(["author", "reuse", "--manifest", "project.json", "--corpus", "corpus.json"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert_eq!(status.code(), Some(2));
            return;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("corpus FIFO blocked the command instead of failing closed");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(unix)]
#[test]
fn review_project_changed_after_seed_is_rejected_before_publication() {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    use std::time::{Duration, Instant};
    let (_temp, root) = project();
    corpus(&root, "# Access\n\nc-1 text\n");
    let bytes = std::fs::read(root.join("corpus.json")).unwrap();
    std::fs::remove_file(root.join("corpus.json")).unwrap();
    assert!(Command::new("mkfifo").arg(root.join("corpus.json")).status().unwrap().success());
    let mut child = Command::new(env!("CARGO_BIN_EXE_forge"))
        .current_dir(&root)
        .args([
            "author",
            "reuse",
            "--manifest",
            "project.json",
            "--corpus",
            "corpus.json",
            "--format",
            "json",
            "--output-dir",
            "out",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut pipe = loop {
        if let Ok(pipe) = std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(root.join("corpus.json"))
        {
            break pipe;
        }
        if let Some(status) = child.try_wait().unwrap() {
            assert_eq!(status.code(), Some(2));
            return;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("child never opened the corpus");
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    // Opening the writer establishes that reuse_seed has completed. The reader
    // cannot finish until the corpus bytes arrive, so mutate a pin in that gap.
    std::fs::write(root.join("pack.json"), b"{}\n").unwrap();
    pipe.write_all(&bytes).unwrap();
    drop(pipe);
    let output = child.wait_with_output().unwrap();
    let published = root.join("out/reuse.json").exists();
    assert_eq!(
        output.status.code(),
        Some(2),
        "changed project accepted; complete stale generation published={published}; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn review_heading_expansion_stays_within_source_budget() {
    let heading = "a".repeat(65536);
    let body = format!("# {heading}\n\n{}", "body\n\n".repeat(1024));
    assert!(body.len() < usize::try_from(forge::reuse::corpus::MAX_DOCUMENT_BYTES).unwrap());
    let blocks = forge::reuse::blocks::blocks(body.as_bytes()).unwrap();
    let allocated: usize =
        blocks.iter().filter_map(|b| b.scope_title.as_ref()).map(String::capacity).sum();
    assert!(
        allocated <= 50 * 1024 * 1024,
        "{} bytes of input retained {} bytes of duplicated headings in {} blocks",
        body.len(),
        allocated,
        blocks.len()
    );
}
