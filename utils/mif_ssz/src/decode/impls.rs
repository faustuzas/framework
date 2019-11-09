use super::*;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_bool() {
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
}
