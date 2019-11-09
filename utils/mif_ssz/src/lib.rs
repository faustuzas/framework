mod encode;
mod decode;

/// The number of bytes used to represent an offset.
pub const OFFSET_LENGTH: usize = 4;

/// The maximum value that can be represented using `OFFSET_LENGTH`.
#[cfg(target_pointer_width = "32")]
pub const MAX_VALUE_LENGTH: usize = (std::u32::MAX >> (8 * (4 - OFFSET_LENGTH))) as usize;
#[cfg(target_pointer_width = "64")]
pub const MAX_VALUE_LENGTH: usize = (std::u64::MAX >> (8 * (8 - OFFSET_LENGTH))) as usize;

pub use decode::{
    impls::decode_list_of_variable_length_items, Decode, DecodeError, SszDecoder, SszDecoderBuilder,
};
pub use encode::{Encode, SszEncoder};

pub fn ssz_encode<T: Encode>(val: &T) -> Vec<u8> {
    val.as_ssz_bytes()
}