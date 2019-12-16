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

pub struct Decoder<'a> {
    bytes: &'a[u8],
    registration_offset: usize,
    fixed_part_offset: usize,
    offsets: Vec<usize>,
    current_offset_index: usize
}

impl<'a> Decoder<'a> {
    pub fn for_bytes(bytes: &'a[u8]) -> Self {
        Self {
            bytes,
            registration_offset: 0,
            fixed_part_offset: 0,
            offsets: vec![],
            current_offset_index: 0
        }
    }

    pub fn next_type<T: Deserialize>(&mut self) -> Result<(), Error>{
        if T::is_variable_size() {
            let offset = match self.bytes.get(self.registration_offset .. self.registration_offset + BYTES_PER_LENGTH_OFFSET) {
                Some(bytes) => deserialize_offset(bytes),
                _ => Err(Error::InvalidByteLength {
                        got: self.bytes.len(),
                        required: self.registration_offset + BYTES_PER_LENGTH_OFFSET
                    })
            }?;
            self.offsets.push(offset);
        }
        self.registration_offset += T::fixed_length();
        Ok(())
    }

    pub fn deserialize_next<T: Deserialize>(&mut self) -> Result<T, Error> {
        let result = if T::is_variable_size() {
            let current_offset = match self.offsets.get(self.current_offset_index) {
                Some(offset) => Ok(*offset),
                _ => Err(Error::NoOffsetsLeft)
            }?;

            let next_offset = match self.offsets.get(self.current_offset_index + 1) {
                Some(offset) => *offset,
                _ => self.bytes.len()
            };

            match self.bytes.get(current_offset..next_offset) {
                Some(bytes) => T::deserialize(bytes),
                _ => Err(Error::InvalidByteLength {
                    got: self.bytes.len(),
                    required: next_offset
                })
            }
        } else {
            match self.bytes.get(self.fixed_part_offset .. self.fixed_part_offset + T::fixed_length()) {
                Some(bytes) => T::deserialize(bytes),
                _ => Err(Error::InvalidByteLength {
                    got: self.bytes.len(),
                    required: self.fixed_part_offset + T::fixed_length()
                })
            }
        };

        if result.is_ok() {
            if T::is_variable_size() {
                self.current_offset_index += 1;
            }
            self.fixed_part_offset += T::fixed_length();
        }

        result
    }
}

#[cfg(test)]
mod tests {
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

    mod decoder {
        use super::*;
        
        #[test]
        fn only_fixed() {
            let mut decoder = Decoder::for_bytes(&[1, 2, 3, 4]);
            decoder.next_type::<u8>().unwrap();
            decoder.next_type::<u8>().unwrap();
            decoder.next_type::<u8>().unwrap();
            decoder.next_type::<u8>().unwrap();
            assert_eq!(decoder.deserialize_next::<u8>().unwrap(), 1);
            assert_eq!(decoder.deserialize_next::<u8>().unwrap(), 2);
            assert_eq!(decoder.deserialize_next::<u8>().unwrap(), 3);
            assert_eq!(decoder.deserialize_next::<u8>().unwrap(), 4);
        }

        #[test]
        fn single_vec() {
            let mut decoder = Decoder::for_bytes(&[4, 0, 0, 0, 1, 2, 3, 4]);
            decoder.next_type::<Vec<u8>>().unwrap();
            assert_eq!(decoder.deserialize_next::<Vec<u8>>().unwrap(), vec![1, 2, 3, 4]);
        }

        #[test]
        fn mixed() {
            let mut decoder = Decoder::for_bytes(&[
                1,
                13, 0, 0, 0,
                255, 255, 255, 255,
                16, 0, 0, 0,
                3, 2, 3,
                1, 0, 2, 0, 3, 0
            ]);
            decoder.next_type::<bool>().unwrap();
            decoder.next_type::<Vec<u8>>().unwrap();
            decoder.next_type::<u32>().unwrap();
            decoder.next_type::<Vec<u16>>().unwrap();
            assert_eq!(decoder.deserialize_next::<bool>().unwrap(), true);
            assert_eq!(decoder.deserialize_next::<Vec<u8>>().unwrap(), vec![3, 2, 3]);
            assert_eq!(decoder.deserialize_next::<u32>().unwrap(), u32::max_value());
            assert_eq!(decoder.deserialize_next::<Vec<u16>>().unwrap(), vec![1, 2, 3]);
        }
    }
}