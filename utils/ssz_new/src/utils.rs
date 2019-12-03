use crate::*;

const MAX_POSSIBLE_OFFSET_VALUE: usize = usize::max_value() >> BYTES_PER_LENGTH_OFFSET * 8;

pub fn serialize_offset(offset: usize) -> Result<Vec<u8>, Error> {
    if offset < MAX_POSSIBLE_OFFSET_VALUE {
        Ok(offset.to_le_bytes()[..BYTES_PER_LENGTH_OFFSET].to_vec())
    } else {
        Err(Error::TooBigOffset { offset })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_serialize_offset() {
        assert_eq!(serialize_offset(0).unwrap(), vec![0; BYTES_PER_LENGTH_OFFSET]);
        assert_eq!(serialize_offset(5).unwrap(), vec![5, 0, 0, 0]);
        assert!(serialize_offset(usize::max_value()).is_err());
    }

    #[test]
    fn test_serialize_offset_error() {
        assert!(serialize_offset(MAX_POSSIBLE_OFFSET_VALUE + 1).is_err())
    }
}