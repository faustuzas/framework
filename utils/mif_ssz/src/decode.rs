/// [00riddle00] Rust makes it a lot more obvious you are moving up in the chain.
/// [00riddle00] As in python, it would be 'import ..color'
use super::*;

/// [00riddle00] public module (with implementations)
pub mod impls;

/// Returned when SSZ decoding fails.
#[derive(Debug, PartialEq)]
pub enum DecodeError {
    /// The bytes supplied were too short to be decoded into the specified type.
    /// [00riddle00] InvalidByteLength (which is enum's variant) includes an
    /// [00riddle00] anonymous struct inside it.
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

/// Provides SSZ decoding (de-serialization) via the `from_ssz_bytes(&bytes)` method.
///
/// See `examples/` for manual implementations or the crate root for implementations using
/// `#[derive(Decode)]`.
/// [00riddle00] implementing
/// [00riddle00] trait inheritance: trait 'Decode' extends trait 'Sized'.
/// [00riddle00] trait inheritance is just a way to specify requirements, that is,
/// [00riddle00] it means that we can know that if some type T implements 'Decode', it also necessarily
/// [00riddle00] implements 'Sized'. However, it does not mean that if a type extends 'Decode', it will
/// [00riddle00] automatically extend 'Sized'.
/// [00riddle00] A trait can extend a combination of other traits (every type which implements it must also
/// [00riddle00] implement all other traits used in a combination.
///
/// [00riddle00] Implementing multiple traits at the same time is not possible. This might be because in the vast
/// [00riddle00] majority of situations traits are sufficiently different so such implementation won’t work anyway.
///
/// [00riddle00] here only function signatures are declared
pub trait Decode: Sized {
    /// Returns `true` if this object has a fixed-length.
    /// I.e., there are no variable length items in this object or any of it's contained objects.
    /// [00riddle00] see [specs:line=160,line=170]
    fn is_ssz_fixed_len() -> bool;

    /// The number of bytes this object occupies in the fixed-length portion of the SSZ bytes.
    /// By default, this is set to `BYTES_PER_LENGTH_OFFSET`
    /// [00riddle00] (see [specs:line=43])
    /// which is suitable for variable length objects, but not fixed-length objects.
    /// Fixed-length objects _must_ return a value which represents their length.
    /// [00riddle00] see [specs:Types (until line 70)]
    fn ssz_fixed_len() -> usize {
        BYTES_PER_LENGTH_OFFSET
    }

    /// Attempts to decode `Self` from `bytes`, returning a `DecodeError` on failure.
    ///
    /// The supplied bytes must be the exact length required to decode `Self`, excess bytes will
    /// result in an error.
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError>;
}
