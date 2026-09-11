//! Ephemeral local unlock and scoped capabilities. These are not reviewer identities.

use std::time::{Duration, Instant};

use argon2::{Algorithm, Argon2, Params, Version};
use serde::Serialize;
use sha2::{Digest as _, Sha256};
use subtle::ConstantTimeEq as _;
use zeroize::Zeroizing;

use super::contract::{Error, Result};

// SEC-ARG D-1: parameters are fixed, never supplied by a request or environment.
const MEMORY_KIB: u32 = 65_536;
const ITERATIONS: u32 = 3;
const LANES: u32 = 1;
const TAG_BYTES: usize = 32;
const SALT_BYTES: usize = 16;
const MAX_CAPABILITIES: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Mode {
    Browser,
    Machine,
}

struct Verifier {
    salt: Zeroizing<[u8; SALT_BYTES]>,
    tag: Zeroizing<[u8; TAG_BYTES]>,
}

struct Capability {
    hash: Zeroizing<[u8; 32]>,
    window: Instant,
    requests: u32,
}

pub(crate) struct Session {
    pub id: String,
    pub mode: Mode,
    pub read_only: bool,
    pub launched_at: String,
    verifier: Option<Verifier>,
    capabilities: Vec<Capability>,
    unlock_failures: u32,
    unlock_after: Instant,
    stopped: bool,
}

pub(crate) fn random_token() -> Result<Zeroizing<String>> {
    use std::fmt::Write as _;
    let mut bytes = Zeroizing::new([0_u8; 32]);
    getrandom::fill(bytes.as_mut()).map_err(|_| internal())?;
    let mut token = Zeroizing::new(String::with_capacity(64));
    for byte in bytes.iter() {
        write!(&mut *token, "{byte:02x}").map_err(|_| internal())?;
    }
    Ok(token)
}

fn internal() -> Error {
    Error::new("internal-error", "The session operation could not be completed.", false)
}
fn unauthorized() -> Error {
    Error::new("unauthorized", "A valid session capability is required.", false)
}

fn digest(secret: &[u8]) -> Zeroizing<[u8; 32]> {
    Zeroizing::new(Sha256::digest(secret).into())
}

fn hash_passphrase(passphrase: &str, salt: &[u8]) -> Result<Zeroizing<[u8; TAG_BYTES]>> {
    let params =
        Params::new(MEMORY_KIB, ITERATIONS, LANES, Some(TAG_BYTES)).map_err(|_| internal())?;
    let mut tag = Zeroizing::new([0_u8; TAG_BYTES]);
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(passphrase.as_bytes(), salt, tag.as_mut())
        .map_err(|_| internal())?;
    Ok(tag)
}

fn valid_passphrase(passphrase: &str) -> bool {
    (15..=128).contains(&passphrase.chars().count())
}

impl Session {
    pub(crate) fn new(
        mode: Mode,
        read_only: bool,
        passphrase: Option<Zeroizing<String>>,
    ) -> Result<Self> {
        let verifier = match (mode, passphrase) {
            (Mode::Machine, None) => None,
            (Mode::Browser, Some(passphrase)) if valid_passphrase(&passphrase) => {
                let mut salt = Zeroizing::new([0_u8; SALT_BYTES]);
                getrandom::fill(salt.as_mut()).map_err(|_| internal())?;
                let tag = hash_passphrase(&passphrase, salt.as_ref())?;
                Some(Verifier { salt, tag })
            }
            _ => return Err(Error::invalid()),
        };
        Ok(Self {
            id: format!("sess_{}", uuid::Uuid::new_v4().simple()),
            mode,
            read_only,
            launched_at: chrono::Utc::now().to_rfc3339(),
            verifier,
            capabilities: Vec::new(),
            unlock_failures: 0,
            unlock_after: Instant::now(),
            stopped: false,
        })
    }

    fn issue(&mut self) -> Result<Zeroizing<String>> {
        if self.stopped || self.capabilities.len() >= MAX_CAPABILITIES {
            return Err(unauthorized());
        }
        let token = random_token()?;
        self.capabilities.push(Capability {
            hash: digest(token.as_bytes()),
            window: Instant::now(),
            requests: 0,
        });
        Ok(token)
    }

    pub(crate) fn machine_capability(&mut self) -> Result<Zeroizing<String>> {
        if self.mode != Mode::Machine || !self.capabilities.is_empty() {
            return Err(unauthorized());
        }
        self.issue()
    }

    pub(crate) fn unlock(&mut self, passphrase: &str) -> Result<Zeroizing<String>> {
        if self.stopped || self.mode != Mode::Browser {
            return Err(unauthorized());
        }
        let now = Instant::now();
        if now < self.unlock_after || self.capabilities.len() >= MAX_CAPABILITIES {
            return Err(Error::new(
                "unlock-throttled",
                "Unlock is temporarily unavailable. Wait before retrying.",
                true,
            ));
        }
        // Invalid-length and malformed submissions take the same fixed-cost hash
        // path as incorrect in-range values. No attacker-selected cost or allocation.
        let valid = valid_passphrase(passphrase);
        let candidate = if valid { passphrase } else { "invalid unlock candidate" };
        let verified = if let Some(verifier) = &self.verifier {
            let tag = hash_passphrase(candidate, verifier.salt.as_ref())?;
            bool::from(tag.as_slice().ct_eq(verifier.tag.as_slice())) && valid
        } else {
            false
        };
        if !verified {
            self.unlock_failures = self.unlock_failures.saturating_add(1).min(6);
            // Start the retry delay after verification. On slower machines the
            // fixed-cost hash itself can exceed the initial two-second delay.
            self.unlock_after = Instant::now() + Duration::from_secs(1 << self.unlock_failures);
            return Err(Error::new("unlock-failed", "The workspace could not be unlocked.", false));
        }
        self.unlock_failures = 0;
        self.unlock_after = Instant::now() + Duration::from_secs(1);
        self.issue()
    }

    /// Browser clients must carry browser metadata; machine capabilities cannot
    /// cross into a browser context even if the token is copied there.
    pub(crate) fn authorize(&mut self, token: &str, browser: bool, mutation: bool) -> Result<()> {
        if self.stopped || browser != (self.mode == Mode::Browser) || token.len() != 64 {
            return Err(unauthorized());
        }
        let candidate = digest(token.as_bytes());
        // Evaluate every slot rather than returning at the first matching secret.
        let matched = self.capabilities.iter().enumerate().fold(None, |found, (index, cap)| {
            if bool::from(candidate.as_slice().ct_eq(cap.hash.as_slice())) {
                Some(index)
            } else {
                found
            }
        });
        let capability = self
            .capabilities
            .get_mut(matched.ok_or_else(unauthorized)?)
            .ok_or_else(unauthorized)?;
        if capability.window.elapsed() >= Duration::from_secs(1) {
            capability.window = Instant::now();
            capability.requests = 0;
        }
        capability.requests += 1;
        if capability.requests > 30 {
            return Err(Error::new(
                "invalid-request",
                "Request rate exceeded. Retry shortly.",
                true,
            ));
        }
        if mutation && self.read_only {
            return Err(Error::new(
                "read-only-session",
                "This session does not permit project changes.",
                false,
            ));
        }
        Ok(())
    }

    pub(crate) fn stop(&mut self) {
        self.stopped = true;
        self.verifier = None;
        self.capabilities.clear();
    }
}

/// The dependency reads the controlling terminal, not redirected stdin. No
/// command-line, environment, or workspace-file password path is supported.
pub(crate) fn prompt() -> Result<Zeroizing<String>> {
    let first = Zeroizing::new(
        rpassword::prompt_password("Set workspace passphrase (15–128 characters): ").map_err(
            |_| {
                Error::new(
                    "invalid-request",
                    "A controlling terminal is required; automation must use --machine-session.",
                    false,
                )
            },
        )?,
    );
    if !valid_passphrase(&first) {
        return Err(Error::new("invalid-request", "Use a passphrase of 15–128 characters.", false));
    }
    let second =
        Zeroizing::new(rpassword::prompt_password("Confirm passphrase: ").map_err(|_| internal())?);
    if !bool::from(digest(first.as_bytes()).as_slice().ct_eq(digest(second.as_bytes()).as_slice()))
    {
        return Err(Error::new(
            "invalid-request",
            "The passphrase confirmation did not match.",
            false,
        ));
    }
    Ok(first)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_scope_read_only_and_shutdown() {
        let mut session = Session::new(Mode::Machine, true, None).unwrap();
        let token = session.machine_capability().unwrap();
        assert_eq!(token.len(), 64);
        assert!(session.machine_capability().is_err());
        session.authorize(&token, false, false).unwrap();
        assert!(session.authorize(&token, true, false).is_err());
        assert_eq!(session.authorize(&token, false, true).unwrap_err().code, "read-only-session");
        session.stop();
        assert!(session.authorize(&token, false, false).is_err());
    }

    #[test]
    fn browser_unlock_throttles_and_mints_distinct_capabilities() {
        let mut session =
            Session::new(Mode::Browser, true, Some(Zeroizing::new("valid long passphrase".into())))
                .unwrap();
        assert_eq!(
            session
                .unlock("wrong long passphrase")
                .map(|_| ())
                .expect_err("wrong passphrase must fail")
                .code,
            "unlock-failed"
        );
        assert_eq!(
            session
                .unlock("valid long passphrase")
                .map(|_| ())
                .expect_err("retry must be throttled")
                .code,
            "unlock-throttled"
        );
        session.unlock_after = Instant::now();
        let first = session.unlock("valid long passphrase").unwrap();
        session.unlock_after = Instant::now();
        let second = session.unlock("valid long passphrase").unwrap();
        assert_ne!(*first, *second);
        session.authorize(&first, true, false).unwrap();
        session.authorize(&second, true, false).unwrap();
        assert!(session.authorize(&first, false, false).is_err());
    }

    #[test]
    fn passphrase_length_counts_unicode_characters() {
        assert!(valid_passphrase(&"é".repeat(15)));
        assert!(!valid_passphrase(&"é".repeat(14)));
        assert!(valid_passphrase(&"é".repeat(128)));
        assert!(!valid_passphrase(&"é".repeat(129)));
    }
}
