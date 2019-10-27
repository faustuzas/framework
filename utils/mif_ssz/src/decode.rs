/// [00riddle00] Rust makes it a lot more obvious you are moving up in the chain.
/// [00riddle00] As in python, it would be 'import ..color'
use super::*;

/// [00riddle00] public module (with implementations)
pub mod impls;

/// Returned when SSZ decoding fails.
#[derive(Debug, PartialEq)]
pub enum DecodeError {
    /// The bytes supplied were too short to be decoded into the specified type.
    /// [00riddle00] InvalidByteLength (which is enum's variant) includes an anonymous struct inside it.
    /// [00riddle00] It will later be used with 'match' keyword
    InvalidByteLength { len: usize, expected: usize },
    /// The given bytes were too short to be read as a length prefix.
    InvalidLengthPrefix { len: usize, expected: usize },
    /// A length offset pointed to a byte that was out-of-bounds (OOB).
    ///
    /// A bytes may be OOB for the following reasons:
    ///
    /// - It is `>= bytes.len()`.
    /// - When decoding variable length items, the 1st offset points "backwards" into the fixed
    /// length items (i.e., `length[0] < BYTES_PER_LENGTH_OFFSET`).
    /// - When decoding variable-length items, the `n`'th offset was less than the `n-1`'th offset.
    OutOfBoundsByte { i: usize },
    /// The given bytes were invalid for some application-level reason.
    BytesInvalid(String),
}
