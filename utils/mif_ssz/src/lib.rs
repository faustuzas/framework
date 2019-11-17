mod encode;
mod decode;

/// Number of bytes per serialized length offset.
pub const BYTES_PER_LENGTH_OFFSET: usize = 4;
/// Number of bytes per chunk.
pub const BYTES_PER_CHUNK: usize = 32;
/// Number of bits per byte.
pub const BITS_PER_BYTE: usize = 8;

///// The maximum value that can be represented using `BYTES_PER_LENGTH_OFFSET`.
#[cfg(target_pointer_width = "64")]
pub const MAX_VALUE_LENGTH: usize = (std::u64::MAX >> (8 * (8 - BYTES_PER_LENGTH_OFFSET))) as usize;

#[cfg(target_pointer_width = "32")]
pub const MAX_VALUE_LENGTH: usize = (std::u32::MAX >> (8 * (4 - BYTES_PER_LENGTH_OFFSET))) as usize;

pub use decode::{Decode, DecodeError, SszDecoder, SszDecoderBuilder};
pub use encode::{Encode, SszEncoder};

pub fn ssz_encode<T: Encode>(val: &T) -> Vec<u8> {
    val.as_ssz_bytes()
}