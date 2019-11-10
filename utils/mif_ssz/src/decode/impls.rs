use super::*;
use core::num::NonZeroUsize;
use ethereum_types::{H256, U128, U256};

macro_rules! impl_decodable_for_uint {
    ($type: ident, $bit_size: expr) => {
        impl Decode for $type {
            fn is_ssz_fixed_len() -> bool {
                panic!("not yet implemented!");
            }

            fn ssz_fixed_len() -> usize {
                panic!("not yet implemented!");
            }

            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
                panic!("not yet implemented!");
            }
        }
    };
}

impl_decodable_for_uint!(u8, 8);
impl_decodable_for_uint!(u16, 16);
impl_decodable_for_uint!(u32, 32);
impl_decodable_for_uint!(u64, 64);

#[cfg(target_pointer_width = "32")]
impl_decodable_for_uint!(usize, 32);

#[cfg(target_pointer_width = "64")]
impl_decodable_for_uint!(usize, 64);

macro_rules! impl_decode_for_tuples {
    ($(
        $Tuple:ident {
            $(($idx:tt) -> $T:ident)+
        }
    )+) => {
        $(
            impl<$($T: Decode),+> Decode for ($($T,)+) {
                fn is_ssz_fixed_len() -> bool {
                    panic!("not yet implemented!");
                }

                fn ssz_fixed_len() -> usize {
                    panic!("not yet implemented!");
                }

                fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
                    panic!("not yet implemented!");
                }
            }
        )+
    }
}

//impl_decode_for_tuples! { }

impl Decode for bool {
    fn is_ssz_fixed_len() -> bool {
        panic!("not yet implemented!");
    }

    fn ssz_fixed_len() -> usize {
        panic!("not yet implemented!");
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        panic!("not yet implemented!");
    }
}

impl Decode for NonZeroUsize {
    fn is_ssz_fixed_len() -> bool {
        panic!("not yet implemented!");
    }

    fn ssz_fixed_len() -> usize {
        panic!("not yet implemented!");
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        panic!("not yet implemented!");
    }
}

/// The SSZ union type.
impl<T: Decode> Decode for Option<T> {
    fn is_ssz_fixed_len() -> bool {
        panic!("not yet implemented!");
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        panic!("not yet implemented!");
    }
}

impl Decode for H256 {
    fn is_ssz_fixed_len() -> bool {
        panic!("not yet implemented!");
    }

    fn ssz_fixed_len() -> usize {
        panic!("not yet implemented!");
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        panic!("not yet implemented!");
    }
}

impl Decode for U256 {
    fn is_ssz_fixed_len() -> bool {
        panic!("not yet implemented!");
    }

    fn ssz_fixed_len() -> usize {
        panic!("not yet implemented!");
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        panic!("not yet implemented!");
    }
}

impl Decode for U128 {
    fn is_ssz_fixed_len() -> bool {
        panic!("not yet implemented!");
    }

    fn ssz_fixed_len() -> usize {
        panic!("not yet implemented!");
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        panic!("not yet implemented!");
    }
}

macro_rules! impl_decodable_for_u8_array {
    ($len: expr) => {
        impl Decode for [u8; $len] {
            fn is_ssz_fixed_len() -> bool {
                panic!("not yet implemented!");
            }

            fn ssz_fixed_len() -> usize {
                panic!("not yet implemented!");
            }

            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
                panic!("not yet implemented!");
            }
        }
    };
}

impl_decodable_for_u8_array!(4);
impl_decodable_for_u8_array!(32);

impl<T: Decode> Decode for Vec<T> {
    fn is_ssz_fixed_len() -> bool {
        panic!("not yet implemented!");
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        panic!("not yet implemented!");
    }
}

/// Decodes `bytes` as if it were a list of variable-length items.
///
/// The `mif_ssz::SszDecoder` can also perform this functionality, however it it significantly faster
/// as it is optimized to read same-typed items whilst `mif_ssz::SszDecoder` supports reading items of
/// differing types.
pub fn decode_list_of_variable_length_items<T: Decode>(
    bytes: &[u8],
) -> Result<Vec<T>, DecodeError> {
    panic!("not yet implemented!");
}
