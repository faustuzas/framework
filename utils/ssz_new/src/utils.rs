use crate::*;

const MAX_POSSIBLE_OFFSET_VALUE: usize = usize::max_value() >> BYTES_PER_LENGTH_OFFSET * 8;

pub fn serialize_offset(offset: usize) -> Result<Vec<u8>, Error> {
    if offset < MAX_POSSIBLE_OFFSET_VALUE {
        Ok(offset.to_le_bytes()[..BYTES_PER_LENGTH_OFFSET].to_vec())
    } else {
        Err(Error::TooBigOffset(offset))
    }
}

pub fn deserialize_offset(bytes: &[u8]) -> Result<usize, Error> {
    if bytes.len() == BYTES_PER_LENGTH_OFFSET {
        let mut arr = [0; BYTES_PER_LENGTH_OFFSET];
        arr.clone_from_slice(bytes);
        Ok(u32::from_le_bytes(arr) as usize)
    } else {
        Err(Error::InvalidByteLength {
            got: bytes.len(),
            required: BYTES_PER_LENGTH_OFFSET
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_serialize_offset() {
        assert_eq!(serialize_offset(0).unwrap(), vec![0; BYTES_PER_LENGTH_OFFSET]);
        assert_eq!(serialize_offset(5).unwrap(), vec![5, 0, 0, 0]);
    }

    #[test]
    fn test_serialize_offset_error() {
        assert!(serialize_offset(usize::max_value()).is_err());
        assert!(serialize_offset(MAX_POSSIBLE_OFFSET_VALUE + 1).is_err())
    }

    #[test]
    fn test_deserialize_offset() {
        assert_eq!(deserialize_offset(&[0; BYTES_PER_LENGTH_OFFSET]).unwrap(), 0);
        assert_eq!(deserialize_offset(&[5, 0, 0, 0]).unwrap(), 5);
    }

    #[test]
    fn test_deserialize_offset_error() {
        assert!(deserialize_offset(&[0; BYTES_PER_LENGTH_OFFSET + 1]).is_err());
    }
}