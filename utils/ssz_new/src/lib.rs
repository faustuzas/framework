mod decode;
mod encode;
mod utils;
mod types;

pub use utils::{
    decode_offset, decode_variable_sized_items, encode_items_from_parts, encode_offset, ssz_encode,
    Decoder,
};

pub const BYTES_PER_LENGTH_OFFSET: usize = 4;

pub trait SszEncode {
    fn ssz_append(&self, buf: &mut Vec<u8>);

    fn is_ssz_fixed_len() -> bool;

    fn ssz_fixed_len() -> usize {
        BYTES_PER_LENGTH_OFFSET
    }

    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];

        self.ssz_append(&mut buf);

        buf
    }
}

pub trait SszDecode: Sized {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, SszDecodeError>;

    fn is_ssz_fixed_len() -> bool;

    fn ssz_fixed_len() -> usize {
        BYTES_PER_LENGTH_OFFSET
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum SszDecodeError {
    InvalidByteLength { len: usize, expected: usize },
    InvalidLengthPrefix { len: usize, expected: usize },
    OutOfBoundsByte { i: usize },
    BytesInvalid(String),
}
