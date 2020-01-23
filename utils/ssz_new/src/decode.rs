#![allow(clippy::use_self)]

use crate::*;
use core::num::NonZeroUsize;
use ethereum_types::{H256, U128, U256};

macro_rules! decode_for_uintn {
    ( $(($type_ident: ty, $size_in_bits: expr)),* ) => { $(
        impl Decode for $type_ident {
            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
                if bytes.len() == <Self as Decode>::ssz_fixed_len() {
                    let mut arr = [0; $size_in_bits / 8];
                    arr.clone_from_slice(bytes);
                    Ok(<$type_ident>::from_le_bytes(arr))
                } else {
                    Err(DecodeError::InvalidByteLength {
                        len: bytes.len(),
                        expected: <Self as Decode>::ssz_fixed_len(),
                    })
                }
            }

            fn is_ssz_fixed_len() -> bool {
                true
            }

            fn ssz_fixed_len() -> usize {
                $size_in_bits / 8
            }
        }
    )* };
}

decode_for_uintn!(
    (u8, 8),
    (u16, 16),
    (u32, 32),
    (u64, 64),
    (usize, std::mem::size_of::<usize>() * 8)
);

macro_rules! decode_for_u8_array {
    ($size: expr) => {
        impl Decode for [u8; $size] {
            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
                if bytes.len() == <Self as Decode>::ssz_fixed_len() {
                    let mut array: [u8; $size] = [0; $size];
                    array.copy_from_slice(&bytes[..]);

                    Ok(array)
                } else {
                    Err(DecodeError::InvalidByteLength {
                        len: bytes.len(),
                        expected: <Self as Decode>::ssz_fixed_len(),
                    })
                }
            }

            fn is_ssz_fixed_len() -> bool {
                true
            }

            fn ssz_fixed_len() -> usize {
                $size
            }
        }
    };
}

decode_for_u8_array!(4);
decode_for_u8_array!(32);

impl Decode for bool {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() == <Self as Decode>::ssz_fixed_len() {
            match bytes[0] {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(DecodeError::BytesInvalid(format!(
                    "Cannot deserialize bool from {}",
                    bytes[0]
                ))),
            }
        } else {
            Err(DecodeError::InvalidByteLength {
                len: bytes.len(),
                expected: <Self as Decode>::ssz_fixed_len(),
            })
        }
    }

    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        1
    }
}

impl<T: Decode> Decode for Vec<T> {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let bytes_len = bytes.len();
        let fixed_len = <T as Decode>::ssz_fixed_len();

        if bytes.is_empty() {
            Ok(vec![])
        } else if !T::is_ssz_fixed_len() {
            decode_variable_sized_items(bytes)
        } else if bytes_len % fixed_len == 0 {
            let mut result = Vec::with_capacity(bytes.len() / fixed_len);
            for chunk in bytes.chunks(fixed_len) {
                result.push(T::from_ssz_bytes(chunk)?);
            }

            Ok(result)
        } else {
            Err(DecodeError::InvalidByteLength {
                len: bytes_len,
                expected: bytes.len() / <T as Decode>::ssz_fixed_len() + 1,
            })
        }
    }

    fn is_ssz_fixed_len() -> bool {
        false
    }
}

impl Decode for NonZeroUsize {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let val = usize::from_ssz_bytes(bytes)?;

        if val == 0 {
            Err(DecodeError::BytesInvalid(
                "NonZeroUsize cannot be zero.".to_string(),
            ))
        } else {
            Ok(NonZeroUsize::new(val).expect("0 check is done above"))
        }
    }

    fn is_ssz_fixed_len() -> bool {
        <usize as Decode>::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        <usize as Decode>::ssz_fixed_len()
    }
}

impl<T: Decode> Decode for Option<T> {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < BYTES_PER_LENGTH_OFFSET {
            return Err(DecodeError::InvalidByteLength {
                len: bytes.len(),
                expected: BYTES_PER_LENGTH_OFFSET,
            });
        }

        let (index_bytes, value_bytes) = bytes.split_at(BYTES_PER_LENGTH_OFFSET);

        let index = decode_offset(index_bytes)?;
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

    fn is_ssz_fixed_len() -> bool {
        false
    }
}

impl Decode for H256 {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let len = bytes.len();
        let expected = <Self as Decode>::ssz_fixed_len();

        if len == expected {
            Ok(H256::from_slice(bytes))
        } else {
            Err(DecodeError::InvalidByteLength { len, expected })
        }
    }

    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        32
    }
}

impl Decode for U256 {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let len = bytes.len();
        let expected = <Self as Decode>::ssz_fixed_len();

        if len == expected {
            Ok(U256::from_little_endian(bytes))
        } else {
            Err(DecodeError::InvalidByteLength { len, expected })
        }
    }

    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        32
    }
}

impl Decode for U128 {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let len = bytes.len();
        let expected = <Self as Decode>::ssz_fixed_len();

        if len == expected {
            Ok(U128::from_little_endian(bytes))
        } else {
            Err(DecodeError::InvalidByteLength { len, expected })
        }
    }

    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u8() {
        assert_eq!(u8::from_ssz_bytes(&[0b0000_0000]).expect("Test"), 0);
        assert_eq!(
            u8::from_ssz_bytes(&[0b1111_1111]).expect("Test"),
            u8::max_value()
        );
        assert_eq!(u8::from_ssz_bytes(&[0b0000_0001]).expect("Test"), 1);
        assert_eq!(u8::from_ssz_bytes(&[0b1000_0000]).expect("Test"), 128);
    }

    #[test]
    fn u16() {
        assert_eq!(
            u16::from_ssz_bytes(&[0b0000_0000, 0b0000_0000]).expect("Test"),
            0
        );
        assert_eq!(
            u16::from_ssz_bytes(&[0b0000_0001, 0b0000_0000]).expect("Test"),
            1
        );
        assert_eq!(
            u16::from_ssz_bytes(&[0b1000_0000, 0b0000_0000]).expect("Test"),
            128
        );
        assert_eq!(
            u16::from_ssz_bytes(&[0b1111_1111, 0b1111_1111]).expect("Test"),
            u16::max_value()
        );
        assert_eq!(
            u16::from_ssz_bytes(&[0b0000_0000, 0b1000_0000]).expect("Test"),
            0x8000
        );
    }

    #[test]
    fn u32() {
        assert_eq!(u32::from_ssz_bytes(&[0b0000_0000; 4]).expect("Test"), 0);
        assert_eq!(
            u32::from_ssz_bytes(&[0b1111_1111; 4]).expect("Test"),
            u32::max_value()
        );
        assert_eq!(
            u32::from_ssz_bytes(&[0b0000_0001, 0b0000_0000, 0b0000_0000, 0b0000_0000])
                .expect("Test"),
            1
        );
        assert_eq!(
            u32::from_ssz_bytes(&[0b1000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000])
                .expect("Test"),
            128
        );
        assert_eq!(
            u32::from_ssz_bytes(&[0b0000_0000, 0b1000_0000, 0b0000_0000, 0b0000_0000])
                .expect("Test"),
            0x8000
        );
        assert_eq!(
            u32::from_ssz_bytes(&[0b0000_0000, 0b0000_0000, 0b0000_0000, 0b1000_0000])
                .expect("Test"),
            0x8000_0000
        );
    }

    #[test]
    fn u64() {
        assert_eq!(u64::from_ssz_bytes(&[0b0000_0000; 8]).expect("Test"), 0);
        assert_eq!(
            u64::from_ssz_bytes(&[0b1111_1111; 8]).expect("Test"),
            u64::max_value()
        );
        assert_eq!(
            u64::from_ssz_bytes(&[
                0b0000_0001,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000
            ])
            .expect("Test"),
            1
        );
        assert_eq!(
            u64::from_ssz_bytes(&[
                0b1000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000
            ])
            .expect("Test"),
            128
        );
        assert_eq!(
            u64::from_ssz_bytes(&[
                0b0000_0000,
                0b1000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000
            ])
            .expect("Test"),
            0x8000
        );
        assert_eq!(
            u64::from_ssz_bytes(&[
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b1000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000
            ])
            .expect("Test"),
            0x8000_0000
        );
        assert_eq!(
            u64::from_ssz_bytes(&[
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b1000_0000
            ])
            .expect("Test"),
            0x8000_0000_0000_0000
        );
    }

    #[test]
    fn bool() {
        assert_eq!(bool::from_ssz_bytes(&[0_u8]).expect("Test"), false);
        assert_eq!(bool::from_ssz_bytes(&[1_u8]).expect("Test"), true);
        assert!(bool::from_ssz_bytes(&[2_u8]).is_err());
        assert!(bool::from_ssz_bytes(&[0_u8, 0_u8]).is_err());
    }

    #[test]
    fn vector_fixed() {
        assert_eq!(<Vec<u8>>::from_ssz_bytes(&[]).expect("Test"), vec![]);
        assert_eq!(
            <Vec<u8>>::from_ssz_bytes(&[0, 1, 2, 3]).expect("Test"),
            vec![0, 1, 2, 3]
        );
        assert_eq!(
            <Vec<u8>>::from_ssz_bytes(&[u8::max_value(); 100]).expect("Test"),
            vec![u8::max_value(); 100]
        );

        assert_eq!(<Vec<u16>>::from_ssz_bytes(&[]).expect("Test"), vec![]);
        assert_eq!(
            <Vec<u16>>::from_ssz_bytes(&[1, 0, 2, 0, 3, 0, 4, 0]).expect("Test"),
            vec![1, 2, 3, 4]
        );
        assert_eq!(
            <Vec<u16>>::from_ssz_bytes(&[u8::max_value(); 200]).expect("Test"),
            vec![u16::max_value(); 100]
        );

        assert_eq!(<Vec<u32>>::from_ssz_bytes(&[]).expect("Test"), vec![]);
        assert_eq!(
            <Vec<u32>>::from_ssz_bytes(&[1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0])
                .expect("Test"),
            vec![1, 2, 3, 4]
        );
        assert_eq!(
            <Vec<u32>>::from_ssz_bytes(&[u8::max_value(); 400]).expect("Test"),
            vec![u32::max_value(); 100]
        );

        assert_eq!(<Vec<u64>>::from_ssz_bytes(&[]).expect("Test"), vec![]);
        assert_eq!(
            <Vec<u64>>::from_ssz_bytes(&[
                1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0,
                0, 0, 0, 0
            ])
            .expect("Test"),
            vec![1, 2, 3, 4]
        );
        assert_eq!(
            <Vec<u64>>::from_ssz_bytes(&[u8::max_value(); 800]).expect("Test"),
            vec![u64::max_value(); 100]
        );
    }

    #[test]
    fn vector_fixed_error() {
        // wrong values provided
        assert!(<Vec<bool>>::from_ssz_bytes(&[0, 1, 2]).is_err());

        // incorrect length of bytes
        assert!(<Vec<u32>>::from_ssz_bytes(&[0, 1, 2, 4, 5]).is_err());
    }

    #[test]
    fn vector_variable() {
        let vec: Vec<Vec<u8>> = vec![];
        assert_eq!(<Vec<Vec<u8>>>::from_ssz_bytes(&[]).expect("Test"), vec);

        let vec: Vec<Vec<u8>> = vec![vec![], vec![]];
        assert_eq!(
            <Vec<Vec<u8>>>::from_ssz_bytes(&[8, 0, 0, 0, 8, 0, 0, 0]).expect("Test"),
            vec
        );

        let vec: Vec<Vec<u8>> = vec![vec![1, 2, 3], vec![4, 5, 6]];
        assert_eq!(
            <Vec<Vec<u8>>>::from_ssz_bytes(&[8, 0, 0, 0, 11, 0, 0, 0, 1, 2, 3, 4, 5, 6])
                .expect("Test"),
            vec
        );
    }

    #[test]
    fn vector_variable_error() {
        // incorrect bytes length for offset
        assert!(<Vec<Vec<u8>>>::from_ssz_bytes(&[0, 1, 2]).is_err());

        // offset is too large
        assert!(<Vec<Vec<u8>>>::from_ssz_bytes(&[10, 0, 0, 0, 2]).is_err());

        // too short value part
        assert!(<Vec<Vec<u64>>>::from_ssz_bytes(&[8, 0, 0, 0, 8, 0, 0, 0, 1]).is_err());

        // wrong bytes to deserialize value
        assert!(<Vec<Vec<bool>>>::from_ssz_bytes(&[8, 0, 0, 0, 8, 0, 0, 0, 2]).is_err());
    }
}
