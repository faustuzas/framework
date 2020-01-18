#![allow(clippy::use_self)]

use crate::*;

macro_rules! deserialize_for_uintn {
    ( $(($type_ident: ty, $size_in_bits: expr)),* ) => { $(
        impl Deserialize for $type_ident {
            fn deserialize(bytes: &[u8]) -> Result<Self, Error> {
                if bytes.len() == Self::fixed_length() {
                    let mut arr = [0; $size_in_bits / 8];
                    arr.clone_from_slice(bytes);
                    Ok(<$type_ident>::from_le_bytes(arr))
                } else {
                    Err(Error::InvalidByteLength {
                        got: bytes.len(),
                        required: Self::fixed_length(),
                    })
                }
            }

            fn is_variable_size() -> bool {
                false
            }

            fn fixed_length() -> usize {
                $size_in_bits / 8
            }
        }
    )* };
}

deserialize_for_uintn!((u8, 8), (u16, 16), (u32, 32), (u64, 64));

impl Deserialize for bool {
    fn deserialize(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() == Self::fixed_length() {
            match bytes[0] {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(Error::InvalidBytes(format!(
                    "Cannot deserialize bool from {}",
                    bytes[0]
                ))),
            }
        } else {
            Err(Error::InvalidByteLength {
                got: bytes.len(),
                required: Self::fixed_length(),
            })
        }
    }

    fn is_variable_size() -> bool {
        false
    }

    fn fixed_length() -> usize {
        1
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize(bytes: &[u8]) -> Result<Self, Error> {
        let bytes_len = bytes.len();

        if bytes.is_empty() {
            Ok(vec![])
        } else if T::is_variable_size() {
            deserialize_variable_sized_items(bytes)
        } else if bytes_len % T::fixed_length() == 0 {
            let mut result = Vec::with_capacity(bytes.len() / T::fixed_length());
            for chunk in bytes.chunks(T::fixed_length()) {
                result.push(T::deserialize(chunk)?);
            }

            Ok(result)
        } else {
            Err(Error::InvalidByteLength {
                got: bytes_len,
                required: bytes.len() / T::fixed_length() + 1,
            })
        }
    }

    fn is_variable_size() -> bool {
        true
    }

    fn fixed_length() -> usize {
        BYTES_PER_LENGTH_OFFSET
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u8() {
        assert_eq!(u8::deserialize(&[0b0000_0000]).expect("Test"), 0);
        assert_eq!(
            u8::deserialize(&[0b1111_1111]).expect("Test"),
            u8::max_value()
        );
        assert_eq!(u8::deserialize(&[0b0000_0001]).expect("Test"), 1);
        assert_eq!(u8::deserialize(&[0b1000_0000]).expect("Test"), 128);
    }

    #[test]
    fn u16() {
        assert_eq!(
            u16::deserialize(&[0b0000_0000, 0b0000_0000]).expect("Test"),
            0
        );
        assert_eq!(
            u16::deserialize(&[0b0000_0001, 0b0000_0000]).expect("Test"),
            1
        );
        assert_eq!(
            u16::deserialize(&[0b1000_0000, 0b0000_0000]).expect("Test"),
            128
        );
        assert_eq!(
            u16::deserialize(&[0b1111_1111, 0b1111_1111]).expect("Test"),
            u16::max_value()
        );
        assert_eq!(
            u16::deserialize(&[0b0000_0000, 0b1000_0000]).expect("Test"),
            0x8000
        );
    }

    #[test]
    fn u32() {
        assert_eq!(u32::deserialize(&[0b0000_0000; 4]).expect("Test"), 0);
        assert_eq!(
            u32::deserialize(&[0b1111_1111; 4]).expect("Test"),
            u32::max_value()
        );
        assert_eq!(
            u32::deserialize(&[0b0000_0001, 0b0000_0000, 0b0000_0000, 0b0000_0000]).expect("Test"),
            1
        );
        assert_eq!(
            u32::deserialize(&[0b1000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000]).expect("Test"),
            128
        );
        assert_eq!(
            u32::deserialize(&[0b0000_0000, 0b1000_0000, 0b0000_0000, 0b0000_0000]).expect("Test"),
            0x8000
        );
        assert_eq!(
            u32::deserialize(&[0b0000_0000, 0b0000_0000, 0b0000_0000, 0b1000_0000]).expect("Test"),
            0x8000_0000
        );
    }

    #[test]
    fn u64() {
        assert_eq!(u64::deserialize(&[0b0000_0000; 8]).expect("Test"), 0);
        assert_eq!(
            u64::deserialize(&[0b1111_1111; 8]).expect("Test"),
            u64::max_value()
        );
        assert_eq!(
            u64::deserialize(&[
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
            u64::deserialize(&[
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
            u64::deserialize(&[
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
            u64::deserialize(&[
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
            u64::deserialize(&[
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
        assert_eq!(bool::deserialize(&[0_u8]).expect("Test"), false);
        assert_eq!(bool::deserialize(&[1_u8]).expect("Test"), true);
        assert!(bool::deserialize(&[2_u8]).is_err());
        assert!(bool::deserialize(&[0_u8, 0_u8]).is_err());
    }

    #[test]
    fn vector_fixed() {
        assert_eq!(<Vec<u8>>::deserialize(&[]).expect("Test"), vec![]);
        assert_eq!(
            <Vec<u8>>::deserialize(&[0, 1, 2, 3]).expect("Test"),
            vec![0, 1, 2, 3]
        );
        assert_eq!(
            <Vec<u8>>::deserialize(&[u8::max_value(); 100]).expect("Test"),
            vec![u8::max_value(); 100]
        );

        assert_eq!(<Vec<u16>>::deserialize(&[]).expect("Test"), vec![]);
        assert_eq!(
            <Vec<u16>>::deserialize(&[1, 0, 2, 0, 3, 0, 4, 0]).expect("Test"),
            vec![1, 2, 3, 4]
        );
        assert_eq!(
            <Vec<u16>>::deserialize(&[u8::max_value(); 200]).expect("Test"),
            vec![u16::max_value(); 100]
        );

        assert_eq!(<Vec<u32>>::deserialize(&[]).expect("Test"), vec![]);
        assert_eq!(
            <Vec<u32>>::deserialize(&[1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0])
                .expect("Test"),
            vec![1, 2, 3, 4]
        );
        assert_eq!(
            <Vec<u32>>::deserialize(&[u8::max_value(); 400]).expect("Test"),
            vec![u32::max_value(); 100]
        );

        assert_eq!(<Vec<u64>>::deserialize(&[]).expect("Test"), vec![]);
        assert_eq!(
            <Vec<u64>>::deserialize(&[
                1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0,
                0, 0, 0, 0
            ])
            .expect("Test"),
            vec![1, 2, 3, 4]
        );
        assert_eq!(
            <Vec<u64>>::deserialize(&[u8::max_value(); 800]).expect("Test"),
            vec![u64::max_value(); 100]
        );
    }

    #[test]
    fn vector_fixed_error() {
        // wrong values provided
        assert!(<Vec<bool>>::deserialize(&[0, 1, 2]).is_err());

        // incorrect length of bytes
        assert!(<Vec<u32>>::deserialize(&[0, 1, 2, 4, 5]).is_err());
    }

    #[test]
    fn vector_variable() {
        let vec: Vec<Vec<u8>> = vec![];
        assert_eq!(<Vec<Vec<u8>>>::deserialize(&[]).expect("Test"), vec);

        let vec: Vec<Vec<u8>> = vec![vec![], vec![]];
        assert_eq!(
            <Vec<Vec<u8>>>::deserialize(&[8, 0, 0, 0, 8, 0, 0, 0]).expect("Test"),
            vec
        );

        let vec: Vec<Vec<u8>> = vec![vec![1, 2, 3], vec![4, 5, 6]];
        assert_eq!(
            <Vec<Vec<u8>>>::deserialize(&[8, 0, 0, 0, 11, 0, 0, 0, 1, 2, 3, 4, 5, 6])
                .expect("Test"),
            vec
        );
    }

    #[test]
    fn vector_variable_error() {
        // incorrect bytes length for offset
        assert!(<Vec<Vec<u8>>>::deserialize(&[0, 1, 2]).is_err());

        // offset is too large
        assert!(<Vec<Vec<u8>>>::deserialize(&[10, 0, 0, 0, 2]).is_err());

        // too short value part
        assert!(<Vec<Vec<u64>>>::deserialize(&[8, 0, 0, 0, 8, 0, 0, 0, 1]).is_err());

        // wrong bytes to deserialize value
        assert!(<Vec<Vec<bool>>>::deserialize(&[8, 0, 0, 0, 8, 0, 0, 0, 2]).is_err());
    }
}
