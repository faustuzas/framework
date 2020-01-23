mod decode;
mod encode;
mod utils;

pub use utils::{deserialize_offset, deserialize_variable_sized_items, serialize_offset, Decoder};

pub const BYTES_PER_LENGTH_OFFSET: usize = 4;

pub trait Encode {
    fn as_ssz_bytes(&self) -> Vec<u8>;

    fn is_ssz_fixed_len() -> bool;

    fn ssz_bytes_len(&self) -> usize {
        self.as_ssz_bytes().len()
    }
}

pub trait Decode: Sized {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError>;

    fn is_ssz_fixed_len() -> bool;

    fn ssz_fixed_len() -> usize {
        BYTES_PER_LENGTH_OFFSET
    }
}

#[derive(Debug, PartialEq)]
pub enum DecodeError {
    InvalidByteLength { len: usize, expected: usize },
    InvalidLengthPrefix { len: usize, expected: usize },
    OutOfBoundsByte { i: usize },
    BytesInvalid(String),
}
