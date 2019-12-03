type Byte = u8;

mod utils;
mod encode;

pub use utils::serialize_offset;
pub use ssz_derive::SszSerialize;

pub const BYTES_PER_LENGTH_OFFSET: usize = 4;

pub trait Serialize {
    fn serialize(&self) -> Result<Vec<Byte>, Error>;

    fn is_variable_size() -> bool;
}

#[derive(Debug)]
pub enum Error {
    TooBigOffset { offset: usize },
    BitsOverflow { bits_count: usize, max_bits: usize },
    IndexOutOfBound { index: usize, max: usize }
}