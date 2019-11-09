use super::*;

macro_rules! uint_n_decoding_impl {
    ($type: ident, $bit_size: expr) => {
        impl Decode for $type {
            fn is_ssz_fixed_len() -> bool {
                true
            }

            fn ssz_fixed_len() -> usize {
                $bit_size / 8
            }

            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
                let len = bytes.len();
                let expected = <Self as Decode>::ssz_fixed_len();

                if len != expected {
                    Err(DecodeError::InvalidByteLength { len, expected })
                } else {
                    let mut array: [u8; $bit_size / 8] = std::default::Default::default();
                    array.clone_from_slice(bytes);

                    Ok(Self::from_le_bytes(array))
                }
            }
        }
    };
}

uint_n_decoding_impl!(u8, 8);
uint_n_decoding_impl!(u16, 16);
uint_n_decoding_impl!(u32, 32);
uint_n_decoding_impl!(u64, 64);


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
    fn vec_of_u16() {
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
    fn vec_of_vec_of_u16() {
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
