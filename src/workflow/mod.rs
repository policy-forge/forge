//! Connector-neutral change sets and the fake-server adapter contract (PRD 065 Phase 1).
//!
//! FORGE exports deterministic `forge.integration-change-set/1` work items from
//! existing local reports, plans them against an explicit target allowlist, and
//! applies the exact unexpired plan through a connector transport. Phase 1 ships
//! only the in-process fake transport; the live GitHub HTTPS transport is deferred
//! pending owner approval of an HTTP-client dependency and the auth/TLS design
//! (see `docs/integration-change-sets.md`). No code in this module performs
//! outbound network access.

pub mod adapter;
pub mod change_set;
pub mod config;
pub mod fake;
pub mod plan;

use std::path::Path;

use crate::ForgeError;
use crate::io;

/// Error wrapper shared by every integration stage; messages never carry secrets.
pub(crate) fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::Integration(message.into())
}

/// Read one bounded regular file through the shared I/O guardrails.
pub(crate) fn read_bounded(path: &Path, max_bytes: u64) -> Result<Vec<u8>, ForgeError> {
    io::read_bounded(path, max_bytes).map_err(|cause| {
        error(format!("cannot read integration input '{}': {cause}", path.display()))
    })
}

/// Serialize one deterministic document with exactly one trailing newline.
pub(crate) fn render_json<T: serde::Serialize>(
    value: &T,
    label: &str,
) -> Result<String, ForgeError> {
    let mut output = serde_json::to_string_pretty(value)
        .map_err(|cause| error(format!("{label} serialization failed: {cause}")))?;
    output.push('\n');
    Ok(output)
}

/// Express `target` relative to `base` for repository-relative links (M-12).
/// Absolute paths outside the base, or non-normal components, are refused.
pub(crate) fn repo_relative(base: &Path, target: &Path) -> Result<String, ForgeError> {
    let base = base.canonicalize().map_err(|cause| {
        error(format!("cannot resolve project root '{}': {cause}", base.display()))
    })?;
    let target = if target.is_absolute() { target.to_path_buf() } else { base.join(target) };
    let target = target.canonicalize().map_err(|cause| {
        error(format!("cannot resolve source path '{}': {cause}", target.display()))
    })?;
    let relative = target.strip_prefix(&base).map_err(|_| {
        error(format!(
            "source '{}' lies outside the project root; only repository-relative links are exported",
            target.display()
        ))
    })?;
    let text = relative.to_string_lossy().replace('\\', "/");
    if text.is_empty() {
        return Err(error("source path resolves to the project root itself"));
    }
    Ok(text)
}

/// Reject credential-shaped strings before they can enter any manifest, plan,
/// state file, report, or payload (M-5/M-14). This is a boundary guard, not a
/// general-purpose secret scanner.
pub(crate) fn reject_secret_text(value: &str, label: &str) -> Result<(), ForgeError> {
    const MARKERS: [&str; 10] = [
        "ghp_",
        "gho_",
        "ghu_",
        "ghs_",
        "ghr_",
        "github_pat_",
        "glpat-",
        "xoxb-",
        "xoxp-",
        "-----begin",
    ];
    let lower = value.to_ascii_lowercase();
    if MARKERS.iter().any(|marker| lower.contains(marker))
        || lower.contains("bearer ")
        || lower.contains("authorization:")
    {
        return Err(error(format!(
            "{label} contains credential-shaped content; secrets must stay in the external credential provider"
        )));
    }
    Ok(())
}

/// Validate an explicit `owner/repo` target label (M-4).
pub(crate) fn valid_repo_target(target: &str) -> bool {
    let mut parts = target.split('/');
    let (Some(owner), Some(repo), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    let segment = |value: &str| {
        !value.is_empty()
            && value.len() <= 100
            && value.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
            && !value.starts_with('.')
            && !value.ends_with('.')
    };
    segment(owner) && segment(repo) && repo != "." && repo != ".."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_targets_are_closed_owner_repo_labels() {
        assert!(valid_repo_target("policy-forge/forge"));
        assert!(valid_repo_target("a_b/c.d"));
        assert!(!valid_repo_target("https://github.com/a/b"));
        assert!(!valid_repo_target("a/b/c"));
        assert!(!valid_repo_target("a/"));
        assert!(!valid_repo_target("/b"));
        assert!(!valid_repo_target("a/../b"));
        assert!(!valid_repo_target(".hidden/repo"));
    }

    #[test]
    fn credential_shapes_are_refused() {
        for secret in [
            "ghp_0123456789abcdef",
            "github_pat_abc",
            "glpat-abc",
            "Authorization: Bearer abc",
            "-----BEGIN PRIVATE KEY-----",
        ] {
            assert!(reject_secret_text(secret, "field").is_err(), "{secret}");
        }
        assert!(reject_secret_text("ordinary policy title", "field").is_ok());
    }
}
