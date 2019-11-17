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
                0b0000_0000 => Ok(false),
                0b0000_0001 => Ok(true),
                _ => Err(DecodeError::BytesInvalid(
                    format!("Out-of-range for boolean: {}", bytes[0]).to_string(),
                )),
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
        if bytes.len() < BYTES_PER_LENGTH_OFFSET {
            return Err(DecodeError::InvalidByteLength {
                len: bytes.len(),
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
    let mut first_value_byte = next_offset(bytes)?;

    let items_count = first_value_byte / BYTES_PER_LENGTH_OFFSET;
    let mut items = Vec::with_capacity(items_count);

    for i in 1..=items_count {
        let items_bytes = (if i == items_count {
            // read to the end
            bytes.get(first_value_byte..)
        } else {
            let next_first_value_byte = next_offset(&bytes[(i * BYTES_PER_LENGTH_OFFSET)..])?;

            let bytes = bytes.get(first_value_byte..next_first_value_byte);
            first_value_byte = next_first_value_byte;

            bytes
        }).ok_or_else(|| DecodeError::OutOfBoundsByte { i: first_value_byte })?;

        items.push(T::from_ssz_bytes(items_bytes)?);
    }

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_bool() {
        assert_eq!(
            bool::from_ssz_bytes(&[0; 2]),
            Err(DecodeError::InvalidByteLength {
                len: 2,
                expected: 1
            })
        );

        assert_eq!(
            bool::from_ssz_bytes(&[]),
            Err(DecodeError::InvalidByteLength {
                len: 0,
                expected: 1
            })
        );

        if let Err(DecodeError::BytesInvalid(_)) = bool::from_ssz_bytes(&[2]) {
            // Success.
        } else {
            panic!("Did not return error on invalid bool val")
        }
    }

    #[test]
    fn test_decode_u16() {
        assert_eq!(<u16>::from_ssz_bytes(&[0, 0]), Ok(0));
        assert_eq!(<u16>::from_ssz_bytes(&[16, 0]), Ok(16));
        assert_eq!(<u16>::from_ssz_bytes(&[0, 1]), Ok(256));
        assert_eq!(<u16>::from_ssz_bytes(&[255, 255]), Ok(65535));

        assert_eq!(
            <u16>::from_ssz_bytes(&[255]),
            Err(DecodeError::InvalidByteLength {
                len: 1,
                expected: 2
            })
        );

        assert_eq!(
            <u16>::from_ssz_bytes(&[]),
            Err(DecodeError::InvalidByteLength {
                len: 0,
                expected: 2
            })
        );

        assert_eq!(
            <u16>::from_ssz_bytes(&[0, 1, 2]),
            Err(DecodeError::InvalidByteLength {
                len: 3,
                expected: 2
            })
        );
    }

    #[test]
    fn test_decode_vec_of_u16() {
        assert_eq!(<Vec<u16>>::from_ssz_bytes(&[0, 0, 0, 0]), Ok(vec![0, 0]));
        assert_eq!(
            <Vec<u16>>::from_ssz_bytes(&[0, 0, 1, 0, 2, 0, 3, 0]),
            Ok(vec![0, 1, 2, 3])
        );
        assert_eq!(<u16>::from_ssz_bytes(&[16, 0]), Ok(16));
        assert_eq!(<u16>::from_ssz_bytes(&[0, 1]), Ok(256));
        assert_eq!(<u16>::from_ssz_bytes(&[255, 255]), Ok(65535));

        assert_eq!(
            <u16>::from_ssz_bytes(&[255]),
            Err(DecodeError::InvalidByteLength {
                len: 1,
                expected: 2
            })
        );

        assert_eq!(
            <u16>::from_ssz_bytes(&[]),
            Err(DecodeError::InvalidByteLength {
                len: 0,
                expected: 2
            })
        );

        assert_eq!(
            <u16>::from_ssz_bytes(&[0, 1, 2]),
            Err(DecodeError::InvalidByteLength {
                len: 3,
                expected: 2
            })
        );
    }

    #[test]
    fn test_decode_vec_of_vec_of_u16() {
        assert_eq!(
            <Vec<Vec<u16>>>::from_ssz_bytes(&[4, 0, 0, 0]),
            Ok(vec![vec![]])
        );

        assert_eq!(
            <Vec<u16>>::from_ssz_bytes(&[0, 0, 1, 0, 2, 0, 3, 0]),
            Ok(vec![0, 1, 2, 3])
        );
        assert_eq!(<u16>::from_ssz_bytes(&[16, 0]), Ok(16));
        assert_eq!(<u16>::from_ssz_bytes(&[0, 1]), Ok(256));
        assert_eq!(<u16>::from_ssz_bytes(&[255, 255]), Ok(65535));

        assert_eq!(
            <u16>::from_ssz_bytes(&[255]),
            Err(DecodeError::InvalidByteLength {
                len: 1,
                expected: 2
            })
        );

        assert_eq!(
            <u16>::from_ssz_bytes(&[]),
            Err(DecodeError::InvalidByteLength {
                len: 0,
                expected: 2
            })
        );

        assert_eq!(
            <u16>::from_ssz_bytes(&[0, 1, 2]),
            Err(DecodeError::InvalidByteLength {
                len: 3,
                expected: 2
            })
        );
    }
}
