//! Shared cryptographic digest rendering.
//!
//! Persisted FORGE fingerprints use exactly 64 lowercase hexadecimal
//! characters. Keeping that representation here prevents individual domain
//! modules from drifting from the shared wire contract.

use sha2::{Digest, Sha256};

const LOWER_HEX: &[u8; 16] = b"0123456789abcdef";

/// Encode arbitrary bytes as lowercase hexadecimal.
pub(crate) fn lower_hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        encoded.push(char::from(LOWER_HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(LOWER_HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

/// Return the SHA-256 digest of `bytes` as lowercase hexadecimal.
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    lower_hex(&Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::{lower_hex, sha256_hex};

    #[test]
    fn lower_hex_preserves_leading_zeroes() {
        assert_eq!(lower_hex(&[0x00, 0x01, 0x0f, 0x10, 0xff]), "00010f10ff");
    }

    #[test]
    fn renders_lowercase_zero_padded_sha256() {
        assert_eq!(
            sha256_hex(b"forge"),
            "71b41d6dd48dc58eba8f5cf9edf30fef6597fdf285a521bb8fcbad4b3d50887d"
        );
    }

    #[test]
    fn renders_empty_input_sha256() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
