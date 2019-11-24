use super::*;
use std::mem;
use core::num::NonZeroUsize;
use ethereum_types::{H256, U128, U256};

macro_rules! uint_n_decoding_impl {
    ($type: ident, $size: expr) => {
        impl Decode for $type {
            fn is_ssz_fixed_len() -> bool {
                true
            }

            fn ssz_fixed_len() -> usize {
                $size / 8
            }

            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
                let len = bytes.len();
                let expected = <Self as Decode>::ssz_fixed_len();

                if len != expected {
                    return Err(DecodeError::InvalidByteLength { len, expected })
                }

                let mut array: [u8; $size / 8] = std::default::Default::default();
                array.clone_from_slice(bytes);

                Ok(Self::from_le_bytes(array))
            }
        }
    };
}

uint_n_decoding_impl!(u8, 8);
uint_n_decoding_impl!(u16, 16);
uint_n_decoding_impl!(u32, 32);
uint_n_decoding_impl!(u64, 64);

#[cfg(target_pointer_width = "32")]
uint_n_decoding_impl!(usize, 32);

#[cfg(target_pointer_width = "64")]
uint_n_decoding_impl!(usize, 64);

impl Decode for bool {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        1
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let len = bytes.len();
        let expected = <Self as Decode>::ssz_fixed_len();

        if len != expected {
            Err(DecodeError::InvalidByteLength { len, expected })
        } else {
            match bytes[0] {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(DecodeError::BytesInvalid(format!("Invalid value for boolean: {}", bytes[0])))
            }

        }
    }
}

impl <T: Decode> Decode for Vec<T> {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            Ok(vec![])
        } else if T::is_ssz_fixed_len() {
            bytes
                .chunks(T::ssz_fixed_len())
                .map(|chunk| T::from_ssz_bytes(chunk))
                .collect()
        } else {
            decode_list_of_variable_length_items(bytes)
        }
    }
}

/// The SSZ Union type.
impl<T: Decode> Decode for Option<T> {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let len = bytes.len();

        if len < BYTES_PER_LENGTH_OFFSET {
            return Err(DecodeError::InvalidByteLength {
                len,
                expected: BYTES_PER_LENGTH_OFFSET,
            });
        }

        let (index_bytes, value_bytes) = bytes.split_at(BYTES_PER_LENGTH_OFFSET);

        let index = read_union_index(index_bytes)?;
        if index == 0 {
            Ok(None)
        } else if index == 1 {
            Ok(Some(T::from_ssz_bytes(value_bytes)?))
        } else {
            Err(DecodeError::BytesInvalid(format!(
                "{} is not a valid union index for Option<T>",
                index
            )))
        }
    }
}

impl Decode for NonZeroUsize {
    fn is_ssz_fixed_len() -> bool {
        <usize as Encode>::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        mem::size_of::<usize>()
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let len = bytes.len();
        let expected = <Self as Decode>::ssz_fixed_len();

        if len != expected {
            return Err(DecodeError::InvalidByteLength { len, expected });
        }

        match NonZeroUsize::new(<usize as Decode>::from_ssz_bytes(bytes)?) {
            Some(val) => Ok(val),
            None => Err(DecodeError::BytesInvalid("NonZeroUsize cannot be zero".to_string()))
        }
    }
}

/// Raw binary data of fixed length (32 bytes)
impl Decode for H256 {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        32
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let len = bytes.len();
        let expected = <Self as Decode>::ssz_fixed_len();

        if len != expected {
            return Err(DecodeError::InvalidByteLength { len, expected });
        }

        Ok(H256::from_slice(bytes))
    }
}

macro_rules! le_integer_decoding_impl {
    ($type: ident, $size: expr) => {
        impl Decode for $type {
            fn is_ssz_fixed_len() -> bool {
                true
            }

            fn ssz_fixed_len() -> usize {
                $size / 8
            }

            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
                let len = bytes.len();
                let expected = <Self as Decode>::ssz_fixed_len();

                if len != expected {
                    return Err(DecodeError::InvalidByteLength { len, expected });
                }

                Ok($type::from_little_endian(bytes))
            }
        }
    };
}

le_integer_decoding_impl!(U128, 128);
le_integer_decoding_impl!(U256, 256);

pub fn decode_list_of_variable_length_items<T: Decode>(
    bytes: &[u8],
) -> Result<Vec<T>, DecodeError> {
    let mut value_offset = next_offset(bytes)?;

    // offset cannot point to the beginning of the bytes
    if value_offset < BYTES_PER_LENGTH_OFFSET {
        return Err(DecodeError::OutOfBoundsByte {
            i: value_offset,
        });
    }

    let items_count = value_offset / BYTES_PER_LENGTH_OFFSET;
    let mut items = Vec::with_capacity(items_count);

    for i in 1..=items_count {
        let items_bytes = (
            // last item
            if i == items_count {
                // read to the end
                bytes.get(value_offset..)
            } else {
                let next_value_offset = next_offset(&bytes[(i * BYTES_PER_LENGTH_OFFSET)..])?;

                // take all bytes between two offsets
                let bytes = bytes.get(value_offset..next_value_offset);
                value_offset = next_value_offset;

                bytes
        }).ok_or_else(|| DecodeError::OutOfBoundsByte { i: value_offset })?;

        items.push(T::from_ssz_bytes(items_bytes)?);
    }

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_bool() {
        assert_eq!(bool::from_ssz_bytes(&[1]).unwrap(), true);
        assert_eq!(bool::from_ssz_bytes(&[0]).unwrap(), false);
    }

    #[test]
    fn test_decode_bool_error() {
        assert_eq!(bool::from_ssz_bytes(&[1, 1]), Err(DecodeError::InvalidByteLength {
            len: 2,
            expected: 1
        }));

        assert_eq!(bool::from_ssz_bytes(&[]), Err(DecodeError::InvalidByteLength {
            len: 0,
            expected: 1
        }));

        assert_eq!(bool::from_ssz_bytes(&[2]), Err(DecodeError::BytesInvalid(
            "Invalid value for boolean: 2".to_string()))
        )
    }

    #[test]
    fn test_decode_u8() {
        assert_eq!(u8::from_ssz_bytes(&[0]).unwrap(), 0_u8);
        assert_eq!(u8::from_ssz_bytes(&[1]).unwrap(), 1_u8);
        assert_eq!(u8::from_ssz_bytes(&[100]).unwrap(), 100_u8);
        assert_eq!(u8::from_ssz_bytes(&[255]).unwrap(), 255_u8);
    }

    #[test]
    fn test_decode_u8_error() {
        assert_eq!(u8::from_ssz_bytes(&[1, 1]), Err(DecodeError::InvalidByteLength {
            len: 2,
            expected: 1
        }));

        assert_eq!(u8::from_ssz_bytes(&[]), Err(DecodeError::InvalidByteLength {
            len: 0,
            expected: 1
        }));
    }

    #[test]
    fn test_decode_u16() {
        assert_eq!(u16::from_ssz_bytes(&[1, 0]).unwrap(), 1_u16);
        assert_eq!(u16::from_ssz_bytes(&[100, 0]).unwrap(), 100_u16);
        assert_eq!(u16::from_ssz_bytes(&[0, 1]).unwrap(), 1_u16 << 8);
        assert_eq!(u16::from_ssz_bytes(&[255, 255]).unwrap(), 65535_u16);
    }

    #[test]
    fn test_decode_u16_error() {
        assert_eq!(u16::from_ssz_bytes(&[1, 1, 1]), Err(DecodeError::InvalidByteLength {
            len: 3,
            expected: 2
        }));

        assert_eq!(u16::from_ssz_bytes(&[]), Err(DecodeError::InvalidByteLength {
            len: 0,
            expected: 2
        }));
    }

    #[test]
    fn test_decode_u32() {
        assert_eq!(u32::from_ssz_bytes(&[1, 0, 0, 0]).unwrap(), 1_u32);
        assert_eq!(u32::from_ssz_bytes(&[100, 0, 0, 0]).unwrap(), 100_u32);
        assert_eq!(u32::from_ssz_bytes(&[0, 0, 1, 0]).unwrap(), 1_u32 << 16);
        assert_eq!(u32::from_ssz_bytes(&[0, 0, 0, 1]).unwrap(), 1_u32 << 24);
        assert_eq!(u32::from_ssz_bytes(&[255, 255, 255, 255]).unwrap(), !0_u32);
    }

    #[test]
    fn test_decode_u32_error() {
        assert_eq!(u32::from_ssz_bytes(&[1, 1, 1, 1, 5]), Err(DecodeError::InvalidByteLength {
            len: 5,
            expected: 4
        }));

        assert_eq!(u32::from_ssz_bytes(&[]), Err(DecodeError::InvalidByteLength {
            len: 0,
            expected: 4
        }));
    }

    #[test]
    fn test_decode_u64() {
        assert_eq!(u64::from_ssz_bytes(&[1, 0, 0, 0, 0, 0, 0, 0]).unwrap(), 1_u64);
        assert_eq!(u64::from_ssz_bytes(&[255, 255, 255, 255, 255, 255, 255, 255]).unwrap(), !0_u64);
    }

    #[test]
    fn test_decode_u64_error() {
        assert_eq!(u64::from_ssz_bytes(&[1, 1, 1, 1, 5, 5, 7, 1, 1]), Err(DecodeError::InvalidByteLength {
            len: 9,
            expected: 8
        }));

        assert_eq!(u64::from_ssz_bytes(&[]), Err(DecodeError::InvalidByteLength {
            len: 0,
            expected: 8
        }));
    }

    #[test]
    fn test_decode_vec_of_u8() {
        assert_eq!(<Vec<u8>>::from_ssz_bytes(&[]).unwrap(), vec![]);
        assert_eq!(<Vec<u8>>::from_ssz_bytes(&[1]).unwrap(), vec![1]);
        assert_eq!(<Vec<u8>>::from_ssz_bytes(&[0, 1, 2, 3]).unwrap(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_decode_vec_of_u64_error() {
        assert_eq!(<Vec<u64>>::from_ssz_bytes(&[0, 1, 2, 3, 4, 5]), Err(DecodeError::InvalidByteLength {
            len: 6,
            expected: 8
        }));

        assert_eq!(<Vec<u64>>::from_ssz_bytes(&[0, 1, 2, 3, 4, 5, 6, 7, 8]), Err(DecodeError::InvalidByteLength {
            len: 1,
            expected: 8
        }));
    }

    #[test]
    fn test_decode_vec_of_vec() {
        assert_eq!(<Vec<Vec<u8>>>::from_ssz_bytes(&[]).unwrap(), vec![] as Vec<Vec<u8>>);
        assert_eq!(<Vec<Vec<u8>>>::from_ssz_bytes(&[4, 0, 0, 0]).unwrap(), vec![vec![]] as Vec<Vec<u8>>);
        assert_eq!(<Vec<Vec<u8>>>::from_ssz_bytes(&[8, 0, 0, 0, 8, 0, 0, 0]).unwrap(), vec![vec![], vec![]] as Vec<Vec<u8>>);
        assert_eq!(<Vec<Vec<u8>>>::from_ssz_bytes(&[8, 0, 0, 0, 11, 0, 0, 0, 0, 1, 2, 11, 22, 33]).unwrap(), vec![vec![0_u8, 1_u8, 2_u8], vec![11_u8, 22_u8, 33_u8]]);
    }

    #[test]
    fn test_decode_vec_of_vec_error() {
        // offset is too short
        assert_eq!(<Vec<Vec<u8>>>::from_ssz_bytes(&[0, 1, 1]), Err(DecodeError::InvalidLengthPrefix {
            len: 3,
            expected: BYTES_PER_LENGTH_OFFSET
        }));

        // offset points to the beginning of the bytes
        assert_eq!(<Vec<Vec<u8>>>::from_ssz_bytes(&[0, 0, 0, 0]), Err(DecodeError::OutOfBoundsByte {
            i: 0
        }));

        // offset is too large
        assert_eq!(<Vec<Vec<u8>>>::from_ssz_bytes(&[8, 0, 0, 0, 32, 0, 0, 0]), Err(DecodeError::OutOfBoundsByte {
            i: 32
        }));

        // cannot decode item from data which offset points to
        assert_eq!(<Vec<Vec<u64>>>::from_ssz_bytes(&[4, 0, 0, 0, 1]), Err(DecodeError::InvalidByteLength {
            len: 1,
            expected: 8
        }));
    }

    #[test]
    fn test_decode_union() {
        assert_eq!(<Option<u8>>::from_ssz_bytes(&[1, 0, 0, 0, 123]).unwrap(), Some(123_u8));
        assert_eq!(<Option<u8>>::from_ssz_bytes(&[0; 4]).unwrap(), None);
    }

    #[test]
    fn test_decode_union_error() {
        assert_eq!(<Option<u8>>::from_ssz_bytes(&[1, 0, 0]), Err(DecodeError::InvalidByteLength {
            len: 3,
            expected: BYTES_PER_LENGTH_OFFSET,
        }));

        assert_eq!(<Option<u8>>::from_ssz_bytes(&[3, 0, 0, 0]), Err(DecodeError::BytesInvalid(format!(
            "{} is not a valid union index for Option<T>",
            3
        ))));
    }

    #[test]
    fn test_decode_h256() {
        assert_eq!(H256::from_ssz_bytes(&[0; 32]).unwrap(), H256::zero());
        assert_eq!(H256::from_ssz_bytes(&[1; 32]).unwrap(), H256::from_slice(&[1; 32]));

        let bytes = vec![
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];
        assert_eq!(H256::from_ssz_bytes(&bytes).unwrap(), H256::from_slice(&bytes));
    }

    #[test]
    fn test_decode_h256_error() {
        assert_eq!(H256::from_ssz_bytes(&[0; 31]), Err(DecodeError::InvalidByteLength {
            len: 31,
            expected: 32
        }));

        assert_eq!(H256::from_ssz_bytes(&[0; 33]), Err(DecodeError::InvalidByteLength {
            len: 33,
            expected: 32
        }));
    }

    #[test]
    fn test_decode_u128() {
        assert_eq!(U128::from_ssz_bytes(&[0; 16]).unwrap(), U128::from_dec_str("0").unwrap());
        assert_eq!(U128::from_ssz_bytes(&[64, 226, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap(), U128::from_dec_str("123456").unwrap());
    }

    #[test]
    fn test_decode_u128_error() {
        assert_eq!(U128::from_ssz_bytes(&[0; 15]), Err(DecodeError::InvalidByteLength {
            len: 15,
            expected: 16
        }));

        assert_eq!(U128::from_ssz_bytes(&[0; 17]), Err(DecodeError::InvalidByteLength {
            len: 17,
            expected: 16
        }));
    }
}
