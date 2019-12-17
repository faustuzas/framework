mod decode;
mod encode;
mod utils;

pub use ssz_derive::{SszDeserialize, SszSerialize};
pub use utils::{deserialize_offset, deserialize_variable_sized_items, serialize_offset, Decoder};

pub const BYTES_PER_LENGTH_OFFSET: usize = 4;

pub trait Serialize {
    fn serialize(&self) -> Result<Vec<u8>, Error>;

    fn is_variable_size() -> bool;
}

pub trait Deserialize: Sized {
    fn deserialize(bytes: &[u8]) -> Result<Self, Error>;

    fn is_variable_size() -> bool;

    fn fixed_length() -> usize;
}

#[derive(Debug)]
pub enum Error {
    TooBigOffset(usize),
    InvalidByteLength { required: usize, got: usize },
    BitsOverflow { bits_count: usize, max_bits: usize },
    NoOffsetsLeft,
    InvalidBytes(String),
    TooMuchElements { got: usize, max: usize },
}
