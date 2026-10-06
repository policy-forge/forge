//! Private streaming controls retain production visibility and full fixed profile limits.
use super::*;
use crate::workspace::preparation::{Interruption, ProgressUpdate, Stage, WorkError};

/// Actual component controller, never a native owner or timer override.
#[derive(Default)]
struct Caller {
    calls: usize,
    stop: Option<usize>,
    stopped: Option<Interruption>,
}
impl WorkControl for Caller {
    /// Observe every real stream callback and inject a typed caller stop when armed.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> Result<(), WorkError> {
        self.calls += 1;
        if self.stop == Some(self.calls) {
            self.stopped = Some(Interruption::CancelRequested);
            return Err(WorkError::Interrupted(Interruption::CancelRequested));
        }
        Ok(())
    }
    /// Retain the exact original interruption.
    fn interruption(&self) -> Option<Interruption> {
        self.stopped
    }
}
/// Exact U64/UTF8/options/hash/list bytes match a frozen independent literal vector.
#[test]
fn primitive_stream_matches_full_literal_framing() {
    let expected = [
        0x02, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0xc3, 0xa9, 0x00, 0x78, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
        0x1e, 0x1f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let mut stream = Stream::new(b"", 1_048_576, &mut ledger, &mut caller).unwrap();
    stream.u64(258).unwrap();
    stream.string("é\0x").unwrap();
    stream.optional_string(None).unwrap();
    stream.optional_string(Some("")).unwrap();
    stream.hash("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f").unwrap();
    stream.strings(&[]).unwrap();
    assert_eq!(stream.encoded, expected.len());
    assert_eq!(
        crate::hashing::lower_hex(&stream.finish().unwrap()),
        crate::hashing::sha256_hex(&expected)
    );
}
/// The actual one-MiB profile includes its NUL prefix and stops before an extra update.
#[test]
fn full_fixed_profile_cap_includes_prefix_and_capacity_remains_first() {
    let prefix = b"forge.lifecycle-review-sources/1\0";
    let body = vec![b'x'; 1_048_576 - prefix.len()];
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    {
        let mut stream = Stream::new(prefix, 1_048_576, &mut ledger, &mut caller).unwrap();
        stream.put(&body).unwrap();
        assert_eq!(stream.encoded, 1_048_576);
        assert_eq!(stream.put(b"x").err(), Some(ContractError::Capacity));
    }
    let calls = caller.calls;
    caller.stop = Some(calls + 1);
    assert_eq!(subject_id("a", "b", &mut ledger, &mut caller).err(), Some(ContractError::Capacity));
    assert_eq!(caller.calls, calls);
}
/// Actual lowerhex syntax is checked before any digest update, without normalization.
#[test]
fn hex_operands_refuse_uppercase_or_wrong_extent() {
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let mut stream =
        Stream::new(b"forge.review-item/2\0", 1_048_576, &mut ledger, &mut caller).unwrap();
    let prefix = stream.encoded;
    for wrong in ["A".repeat(64), "0".repeat(63), "0".repeat(65)] {
        assert_eq!(stream.hash(&wrong).err(), Some(ContractError::Invalid));
        assert_eq!(stream.encoded, prefix);
    }
}
