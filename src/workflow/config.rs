//! Connector configuration and network-safety contract (PRD 065 M-4/M-5/M-13).
//!
//! Phase 1 expresses the network-safety envelope as closed, validated types even
//! though the live transport is deferred. The fake in-process adapter enforces
//! the same rules; Phase 2 must bind a real HTTPS client to these exact limits
//! (fixed host, TLS required, no redirects, byte bounds, timeouts, rate limits)
//! and source credentials from an approved external provider — never from this
//! file, flags, logs, or state.

use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{error, read_bounded, reject_secret_text, render_json, valid_repo_target};
use crate::ForgeError;
use crate::json_strict::{self, Limits};

/// Versioned connector configuration format.
pub const CONFIG_SCHEMA_VERSION: &str = "forge.integration-connector/1";
/// Raw configuration byte ceiling.
pub const MAX_CONFIG_BYTES: u64 = 1024 * 1024;
/// Maximum allowlisted targets.
pub const MAX_TARGETS: usize = 64;

/// The only reference connector shipped in Phase 1.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ConnectorKind {
    /// GitHub Issues reference adapter (fake transport in Phase 1).
    GithubIssues,
}

/// Fixed provider host for the reference connector (M-13). No arbitrary URLs.
pub const GITHUB_API_HOST: &str = "api.github.com";

/// Network-safety envelope enforced by every adapter transport (M-13).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NetworkPolicy {
    /// Fixed provider host; the transport must refuse any other host.
    pub host: String,
    /// TLS with certificate verification is mandatory; plaintext is refused.
    pub tls_required: bool,
    /// Redirects are never followed; a redirect response is a hard failure.
    pub follow_redirects: bool,
    /// Maximum serialized request body bytes.
    pub max_request_bytes: usize,
    /// Maximum response body bytes.
    pub max_response_bytes: usize,
    /// Per-request timeout in milliseconds.
    pub timeout_ms: u64,
    /// Maximum requests per minute the transport may issue.
    pub max_requests_per_minute: u32,
}

impl Default for NetworkPolicy {
    fn default() -> Self {
        Self {
            host: GITHUB_API_HOST.to_string(),
            tls_required: true,
            follow_redirects: false,
            max_request_bytes: 256 * 1024,
            max_response_bytes: 1024 * 1024,
            timeout_ms: 30_000,
            max_requests_per_minute: 60,
        }
    }
}

impl NetworkPolicy {
    /// Enforce the closed Phase 1 safety envelope.
    ///
    /// # Errors
    ///
    /// Returns [`ForgeError::Integration`] when any bound is loosened beyond the
    /// contract or the host is not the fixed provider host.
    pub fn validate(&self) -> Result<(), ForgeError> {
        if self.host != GITHUB_API_HOST {
            return Err(error(format!(
                "connector host must be the fixed provider host '{GITHUB_API_HOST}'"
            )));
        }
        if !self.tls_required {
            return Err(error("TLS verification is required; plaintext transport is refused"));
        }
        if self.follow_redirects {
            return Err(error("following redirects is refused by the network-safety contract"));
        }
        if self.max_request_bytes == 0 || self.max_request_bytes > 1024 * 1024 {
            return Err(error("max_request_bytes must be within 1..=1048576"));
        }
        if self.max_response_bytes == 0 || self.max_response_bytes > 4 * 1024 * 1024 {
            return Err(error("max_response_bytes must be within 1..=4194304"));
        }
        if self.timeout_ms == 0 || self.timeout_ms > 120_000 {
            return Err(error("timeout_ms must be within 1..=120000"));
        }
        if self.max_requests_per_minute == 0 || self.max_requests_per_minute > 600 {
            return Err(error("max_requests_per_minute must be within 1..=600"));
        }
        Ok(())
    }
}

/// Explicit connector configuration with a closed target allowlist (M-4).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConnectorConfig {
    /// Exact [`CONFIG_SCHEMA_VERSION`].
    pub schema_version: String,
    /// Reference connector selection.
    pub connector: ConnectorKind,
    /// Explicit `owner/repo` allowlist; every operation target must appear here.
    pub targets: Vec<String>,
    /// Network-safety envelope.
    pub network: NetworkPolicy,
}

impl ConnectorConfig {
    /// Parse and validate a bounded connector configuration.
    ///
    /// # Errors
    ///
    /// Returns [`ForgeError::Integration`] for malformed documents, empty or
    /// invalid allowlists, loosened network bounds, or credential-shaped content.
    pub fn parse(bytes: &[u8]) -> Result<Self, ForgeError> {
        if bytes.len() as u64 > MAX_CONFIG_BYTES {
            return Err(error(format!(
                "connector config exceeds the {MAX_CONFIG_BYTES} byte limit"
            )));
        }
        let value = json_strict::parse_value(
            bytes,
            "integration connector config",
            Limits { max_depth: 16, max_string_bytes: 4096 },
        )
        .map_err(|_| error("invalid bounded duplicate-free connector config JSON"))?;
        let config: Self = serde_json::from_value(value)
            .map_err(|_| error("connector config fields do not match the closed contract"))?;
        config.validate()?;
        Ok(config)
    }

    /// Validate all closed-contract invariants.
    ///
    /// # Errors
    ///
    /// Returns [`ForgeError::Integration`] when the configuration is unsafe.
    pub fn validate(&self) -> Result<(), ForgeError> {
        if self.schema_version != CONFIG_SCHEMA_VERSION {
            return Err(error(format!(
                "unsupported schema_version '{}'; expected {CONFIG_SCHEMA_VERSION}",
                self.schema_version
            )));
        }
        if self.targets.is_empty() || self.targets.len() > MAX_TARGETS {
            return Err(error(format!("targets must list 1..={MAX_TARGETS} owner/repo entries")));
        }
        let mut seen = BTreeSet::new();
        for target in &self.targets {
            reject_secret_text(target, "target")?;
            if !valid_repo_target(target) {
                return Err(error(format!(
                    "invalid allowlist target '{target}'; expected owner/repo"
                )));
            }
            if !seen.insert(target.as_str()) {
                return Err(error(format!("duplicate allowlist target '{target}'")));
            }
        }
        self.network.validate()
    }

    /// Whether `target` is explicitly allowlisted (M-4).
    #[must_use]
    pub fn allows(&self, target: &str) -> bool {
        self.targets.iter().any(|allowed| allowed == target)
    }

    /// Render deterministic JSON bytes with one trailing newline.
    ///
    /// # Errors
    ///
    /// Returns [`ForgeError::Integration`] when serialization fails.
    pub fn render(&self) -> Result<String, ForgeError> {
        render_json(self, "connector config")
    }
}

/// Load a connector configuration from disk.
///
/// # Errors
///
/// Returns [`ForgeError::Integration`] when the file cannot be read or is invalid.
pub fn load(path: &Path) -> Result<ConnectorConfig, ForgeError> {
    ConnectorConfig::parse(&read_bounded(path, MAX_CONFIG_BYTES)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> ConnectorConfig {
        ConnectorConfig {
            schema_version: CONFIG_SCHEMA_VERSION.to_string(),
            connector: ConnectorKind::GithubIssues,
            targets: vec!["policy-forge/forge".to_string()],
            network: NetworkPolicy::default(),
        }
    }

    #[test]
    fn default_policy_is_the_closed_envelope() {
        config().validate().unwrap();
        assert!(config().allows("policy-forge/forge"));
        assert!(!config().allows("other/repo"));
    }

    #[test]
    fn loosened_network_bounds_are_refused() {
        for mutate in [
            |n: &mut NetworkPolicy| n.host = "evil.example".to_string(),
            |n: &mut NetworkPolicy| n.tls_required = false,
            |n: &mut NetworkPolicy| n.follow_redirects = true,
            |n: &mut NetworkPolicy| n.max_request_bytes = 0,
            |n: &mut NetworkPolicy| n.max_response_bytes = 8 * 1024 * 1024,
            |n: &mut NetworkPolicy| n.timeout_ms = 0,
            |n: &mut NetworkPolicy| n.max_requests_per_minute = 0,
        ] {
            let mut c = config();
            mutate(&mut c.network);
            assert!(c.validate().is_err());
        }
    }

    #[test]
    fn empty_or_invalid_allowlists_are_refused() {
        let mut c = config();
        c.targets.clear();
        assert!(c.validate().is_err());
        c.targets = vec!["https://github.com/a/b".to_string()];
        assert!(c.validate().is_err());
        c.targets = vec!["a/b".to_string(), "a/b".to_string()];
        assert!(c.validate().is_err());
    }

    #[test]
    fn secrets_in_config_are_refused() {
        let mut c = config();
        c.targets = vec!["ghp_0123456789abcdef/repo".to_string()];
        assert!(c.validate().is_err());
    }

    #[test]
    fn round_trip_is_deterministic() {
        let c = config();
        let bytes = c.render().unwrap();
        assert_eq!(bytes, c.render().unwrap());
        assert_eq!(ConnectorConfig::parse(bytes.as_bytes()).unwrap(), c);
    }
}
