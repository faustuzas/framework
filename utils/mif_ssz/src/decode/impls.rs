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
        true
    }

    fn ssz_fixed_len() -> usize {
        1
    }

    fn from_ssz_bytes(byte_stream: &[u8]) -> Result<Self, DecodeError> {
        let expect = <Self as Decode>::ssz_fixed_len();
        let lg = byte_stream.len();

        if expect != lg {
            return Err(DecodeError::InvalidByteLength { len: lg, expected: expect });
        }

        match byte_stream[0] {
            0b0000_0001 => Ok(true),
            0b0000_0000 => Ok(false),
            _ => Err(DecodeError::BytesInvalid(
                format!("Out-of-range for boolean: {}", byte_stream[0]).to_string(),
            )),
        }
    }
}

impl Decode for NonZeroUsize {
    fn is_ssz_fixed_len() -> bool {
        <usize as Decode>::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        <usize as Decode>::ssz_fixed_len()
    }

    fn from_ssz_bytes(byte_stream: &[u8]) -> Result<Self, DecodeError> {
        let x = usize::from_ssz_bytes(byte_stream)?;

        if x == 0 {
            Err(DecodeError::BytesInvalid(
                "NonZeroUsize cannot be zero.".to_string(),
            ))
        } else {
            // `unwrap` is safe here as `NonZeroUsize::new()` succeeds if `x > 0` and this path
            // never executes when `x == 0`.
            Ok(NonZeroUsize::new(x).unwrap())
        }
    }
}

/// The SSZ union type.
impl<T: Decode> Decode for Option<T> {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(byte_stream: &[u8]) -> Result<Self, DecodeError> {
        let lg = byte_stream.len();
        if BYTES_PER_LENGTH_OFFSET > lg {
            return Err(DecodeError::InvalidByteLength {
                expected: BYTES_PER_LENGTH_OFFSET,
                len: lg,
            });
        }

        let (index_bytes, value_bytes) = byte_stream.split_at(BYTES_PER_LENGTH_OFFSET);

        let index = read_union_index(index_bytes)?;
        match index {
            0 => Ok(None),
            1 => Ok(Some(T::from_ssz_bytes(value_bytes)?)),
            _ => Err(DecodeError::BytesInvalid(format!(
                "{} is not a valid union index for Option<T>",
                index
            ))),
        }
    }
}


impl Decode for H256 {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        32
    }

    fn from_ssz_bytes(byte_stream: &[u8]) -> Result<Self, DecodeError> {
        let expect = <Self as Decode>::ssz_fixed_len();
        let lg = byte_stream.len();

        if expect != lg {
            Err(DecodeError::InvalidByteLength { expected: expect, len: lg })
        } else {
            Ok(H256::from_slice(byte_stream))
        }
    }
}

impl Decode for U256 {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        32
    }

    fn from_ssz_bytes(byte_stream: &[u8]) -> Result<Self, DecodeError> {
        let expect = <Self as Decode>::ssz_fixed_len();
        let lg = byte_stream.len();

        if expect != lg {
            Err(DecodeError::InvalidByteLength { expected: expect, len: lg })
        } else {
            Ok(U256::from_little_endian(byte_stream))
        }
    }
}

impl Decode for U128 {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        16
    }

    fn from_ssz_bytes(byte_stream: &[u8]) -> Result<Self, DecodeError> {
        let expect = <Self as Decode>::ssz_fixed_len();
        let lg = byte_stream.len();

        if expect != lg {
            Err(DecodeError::InvalidByteLength { expected: expect, len: lg })
        } else {
            Ok(U128::from_little_endian(byte_stream))
        }
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
