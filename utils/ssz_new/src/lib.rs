type Byte = u8;

mod types;
mod utils;
mod encode;

pub use types::{Bitvector};

pub const BYTES_PER_LENGTH_OFFSET: usize = 4;

trait SszEncode {
    fn serialize(&self) -> Result<Vec<Byte>, Error>;

    fn is_variable_size() -> bool;
}

#[derive(Debug)]
pub enum Error {
    TooBigOffset { offset: usize },
    BitsOverflow { bits_count: usize, max_bits: usize },
    IndexOutOfBound { index: usize, max: usize }
}