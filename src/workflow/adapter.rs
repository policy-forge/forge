//! Connector transport contract shared by every adapter (PRD 065 M-4/M-8/M-9/M-13).
//!
//! Phase 1 ships only the loopback fake transport ([`super::fake`]). A live
//! GitHub adapter must implement this same contract over HTTPS with the closed
//! [`super::config::NetworkPolicy`] envelope: fixed host, TLS verification, no
//! redirects, request/response byte bounds, per-request timeouts, rate limits,
//! and credentials supplied by an approved external provider at request time.

use serde::{Deserialize, Serialize};

use super::config::NetworkPolicy;

/// Desired remote operation for one work item (M-4).
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum OperationKind {
    /// Create a new remote work object.
    Create,
    /// Update an existing remote work object under a version precondition.
    Update,
    /// Request closure of an existing remote work object; FORGE never asserts
    /// the underlying work is complete (M-11).
    CloseRequest,
}

impl OperationKind {
    /// Exact wire label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::CloseRequest => "close-request",
        }
    }
}

/// Minimized remote payload: title plus deterministic body text (M-12).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Payload {
    /// Remote object title.
    pub title: String,
    /// Deterministic minimized body; IDs, reason codes, owners, dates, links.
    pub body: String,
}

/// Remote version precondition recorded by the dry-run plan (M-6/M-9).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Precondition {
    /// Recorded remote object ID.
    pub remote_id: String,
    /// Recorded remote version the plan was built against.
    pub remote_version: u64,
}

/// One transport request. The idempotency key makes retries duplicate-free (M-8).
#[derive(Debug, Clone)]
pub struct OperationRequest {
    /// Allowlisted `owner/repo` target.
    pub target: String,
    /// Stable idempotency key derived by the planner.
    pub idempotency_key: String,
    /// Desired operation.
    pub kind: OperationKind,
    /// Minimized payload.
    pub payload: Payload,
    /// Required for update/close-request; absent for create.
    pub precondition: Option<Precondition>,
}

/// Successful remote outcome with recorded identity (M-4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteObject {
    /// Provider-assigned remote object ID.
    pub remote_id: String,
    /// Monotonic remote version after the operation.
    pub remote_version: u64,
    /// Observed remote state label, e.g. `open` or `closed`.
    pub state: String,
}

/// Closed transport failure classes. Messages are redacted and never carry
/// credentials, request bodies, or provider prose (M-14).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportError {
    /// Target is not allowlisted (M-4).
    TargetRefused,
    /// Remote version drifted from the plan precondition; no write occurred (M-9).
    Conflict {
        /// Version the plan expected.
        expected: u64,
        /// Version actually observed.
        observed: u64,
    },
    /// Provider rate limit; retry after the given seconds (M-13).
    RateLimited {
        /// Provider-supplied retry delay.
        retry_after_secs: u64,
    },
    /// Provider answered with a redirect; following it is refused (M-13).
    RedirectRefused,
    /// Request exceeded the configured byte bound (M-13).
    RequestTooLarge,
    /// Response exceeded the configured byte bound (M-13).
    ResponseTooLarge,
    /// Request did not complete within the configured timeout (M-13).
    Timeout,
    /// Remote object no longer exists.
    NotFound,
    /// Provider-side failure; the operation was not applied.
    ServerFailure {
        /// HTTP-style status class label.
        status: u16,
    },
    /// Transport protocol or credential failure; redacted detail.
    Protocol(String),
}

impl TransportError {
    /// Stable machine-readable class label for audit reports.
    #[must_use]
    pub const fn class(&self) -> &'static str {
        match self {
            Self::TargetRefused => "target-refused",
            Self::Conflict { .. } => "conflict",
            Self::RateLimited { .. } => "rate-limited",
            Self::RedirectRefused => "redirect-refused",
            Self::RequestTooLarge => "request-too-large",
            Self::ResponseTooLarge => "response-too-large",
            Self::Timeout => "timeout",
            Self::NotFound => "not-found",
            Self::ServerFailure { .. } => "server-failure",
            Self::Protocol(_) => "protocol",
        }
    }

    /// Redacted human-readable detail; never includes secrets or payloads.
    #[must_use]
    pub fn detail(&self) -> String {
        match self {
            Self::Conflict { expected, observed } => {
                format!("remote version drift: expected {expected}, observed {observed}")
            }
            Self::RateLimited { retry_after_secs } => {
                format!("rate limited; retry after {retry_after_secs}s")
            }
            Self::ServerFailure { status } => format!("provider failure ({status})"),
            Self::Protocol(message) => message.clone(),
            other => other.class().to_string(),
        }
    }
}

/// Credential boundary (M-5): tokens come from an approved external provider at
/// request time and are never stored in manifests, plans, state, or reports.
pub trait CredentialProvider {
    /// Fetch the current credential. Phase 1 fakes return a synthetic token.
    ///
    /// # Errors
    ///
    /// Returns a redacted message when no credential is available.
    fn token(&self) -> Result<String, TransportError>;
}

/// The connector transport contract. Implementations must enforce the network
/// policy, idempotency, and version preconditions exactly as the fake does.
pub trait Transport {
    /// Execute one planned operation. Repeated calls with the same idempotency
    /// key must return the original outcome without creating duplicates (M-8).
    ///
    /// # Errors
    ///
    /// Returns a closed [`TransportError`] class; no partial write may be
    /// reported as success.
    fn execute(
        &mut self,
        policy: &NetworkPolicy,
        request: &OperationRequest,
    ) -> Result<RemoteObject, TransportError>;

    /// Read-only observation of one remote object for reconciliation (M-11).
    /// `Ok(None)` means the object no longer exists.
    ///
    /// # Errors
    ///
    /// Returns a closed [`TransportError`] class.
    fn observe(
        &mut self,
        policy: &NetworkPolicy,
        target: &str,
        remote_id: &str,
    ) -> Result<Option<RemoteObject>, TransportError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_details_are_closed_and_redacted() {
        let conflict = TransportError::Conflict { expected: 1, observed: 2 };
        assert_eq!(conflict.class(), "conflict");
        assert_eq!(conflict.detail(), "remote version drift: expected 1, observed 2");
        assert_eq!(TransportError::Timeout.detail(), "timeout");
        assert_eq!(
            TransportError::ServerFailure { status: 500 }.detail(),
            "provider failure (500)"
        );
    }
}
