use crate::*;

const MAX_POSSIBLE_OFFSET_VALUE: usize = usize::max_value() >> (BYTES_PER_LENGTH_OFFSET * 8);

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
            required: BYTES_PER_LENGTH_OFFSET,
        })
    }
}

pub fn deserialize_variable_sized_items<T: Deserialize>(bytes: &[u8]) -> Result<Vec<T>, Error> {
    let first_offset_bytes = bytes.get(0..BYTES_PER_LENGTH_OFFSET);
    let first_offset = match first_offset_bytes {
        Some(bytes) => deserialize_offset(bytes),
        _ => Err(Error::InvalidByteLength {
            required: BYTES_PER_LENGTH_OFFSET,
            got: bytes.len(),
        }),
    }?;

    let number_of_elements = first_offset / BYTES_PER_LENGTH_OFFSET;
    let mut result = Vec::with_capacity(number_of_elements);

    let mut previous_offset = first_offset;
    for i in 1..=number_of_elements {
        let next_offset = if i == number_of_elements {
            bytes.len()
        } else {
            match bytes.get(i * BYTES_PER_LENGTH_OFFSET..(i + 1) * BYTES_PER_LENGTH_OFFSET) {
                Some(bytes) => deserialize_offset(bytes),
                _ => Err(Error::InvalidByteLength {
                    required: (i + 1) * BYTES_PER_LENGTH_OFFSET,
                    got: bytes.len(),
                }),
            }?
        };

        let element = match bytes.get(previous_offset..next_offset) {
            Some(bytes) => T::deserialize(bytes),
            _ => Err(Error::InvalidByteLength {
                required: next_offset,
                got: bytes.len(),
            }),
        }?;

        result.push(element);
        previous_offset = next_offset;
    }
    Ok(result)
}

pub struct Decoder<'a> {
    bytes: &'a [u8],
    registration_offset: usize,
    fixed_part_offset: usize,
    offsets: Vec<usize>,
    current_offset_index: usize,
}

impl<'a> Decoder<'a> {
    pub fn for_bytes(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            registration_offset: 0,
            fixed_part_offset: 0,
            offsets: vec![],
            current_offset_index: 0,
        }
    }

    pub fn next_type<T: Deserialize>(&mut self) -> Result<(), Error> {
        if T::is_variable_size() {
            let offset = match self
                .bytes
                .get(self.registration_offset..self.registration_offset + BYTES_PER_LENGTH_OFFSET)
            {
                Some(bytes) => deserialize_offset(bytes),
                _ => Err(Error::InvalidByteLength {
                    got: self.bytes.len(),
                    required: self.registration_offset + BYTES_PER_LENGTH_OFFSET,
                }),
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
                _ => Err(Error::NoOffsetsLeft),
            }?;

            let next_offset = match self.offsets.get(self.current_offset_index + 1) {
                Some(offset) => *offset,
                _ => self.bytes.len(),
            };

            match self.bytes.get(current_offset..next_offset) {
                Some(bytes) => T::deserialize(bytes),
                _ => Err(Error::InvalidByteLength {
                    got: self.bytes.len(),
                    required: next_offset,
                }),
            }
        } else {
            match self
                .bytes
                .get(self.fixed_part_offset..self.fixed_part_offset + T::fixed_length())
            {
                Some(bytes) => T::deserialize(bytes),
                _ => Err(Error::InvalidByteLength {
                    got: self.bytes.len(),
                    required: self.fixed_part_offset + T::fixed_length(),
                }),
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
        assert_eq!(
            serialize_offset(0).expect("Test"),
            vec![0; BYTES_PER_LENGTH_OFFSET]
        );
        assert_eq!(serialize_offset(5).expect("Test"), vec![5, 0, 0, 0]);
    }

    #[test]
    fn test_serialize_offset_error() {
        assert!(serialize_offset(usize::max_value()).is_err());
        assert!(serialize_offset(MAX_POSSIBLE_OFFSET_VALUE + 1).is_err())
    }

    #[test]
    fn test_deserialize_offset() {
        assert_eq!(
            deserialize_offset(&[0; BYTES_PER_LENGTH_OFFSET]).expect("Test"),
            0
        );
        assert_eq!(deserialize_offset(&[5, 0, 0, 0]).expect("Test"), 5);
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
            decoder.next_type::<u8>().expect("Test");
            decoder.next_type::<u8>().expect("Test");
            decoder.next_type::<u8>().expect("Test");
            decoder.next_type::<u8>().expect("Test");
            assert_eq!(decoder.deserialize_next::<u8>().expect("Test"), 1);
            assert_eq!(decoder.deserialize_next::<u8>().expect("Test"), 2);
            assert_eq!(decoder.deserialize_next::<u8>().expect("Test"), 3);
            assert_eq!(decoder.deserialize_next::<u8>().expect("Test"), 4);
        }

        #[test]
        fn single_vec() {
            let mut decoder = Decoder::for_bytes(&[4, 0, 0, 0, 1, 2, 3, 4]);
            decoder.next_type::<Vec<u8>>().expect("Test");
            assert_eq!(
                decoder.deserialize_next::<Vec<u8>>().expect("Test"),
                vec![1, 2, 3, 4]
            );
        }

        #[test]
        fn mixed() {
            let mut decoder = Decoder::for_bytes(&[
                1, 13, 0, 0, 0, 255, 255, 255, 255, 16, 0, 0, 0, 3, 2, 3, 1, 0, 2, 0, 3, 0,
            ]);
            decoder.next_type::<bool>().expect("Test");
            decoder.next_type::<Vec<u8>>().expect("Test");
            decoder.next_type::<u32>().expect("Test");
            decoder.next_type::<Vec<u16>>().expect("Test");
            assert_eq!(decoder.deserialize_next::<bool>().expect("Test"), true);
            assert_eq!(
                decoder.deserialize_next::<u32>().expect("Test"),
                u32::max_value()
            );
            assert_eq!(
                decoder.deserialize_next::<Vec<u8>>().expect("Test"),
                vec![3, 2, 3]
            );
            assert_eq!(
                decoder.deserialize_next::<Vec<u16>>().expect("Test"),
                vec![1, 2, 3]
            );
        }
    }

    mod deserialize_variable_sized_items {
        use super::*;

        #[test]
        fn happy_path() {
            let items: Vec<Vec<u8>> = deserialize_variable_sized_items(&[
                12, 0, 0, 0, 16, 0, 0, 0, 22, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
            ])
            .expect("Test");

            assert_eq!(
                items,
                vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8, 9, 10], vec![]]
            )
        }

        #[test]
        fn empty_bytes() {
            let result: Result<Vec<Vec<u8>>, _> = deserialize_variable_sized_items(&[]);
            assert!(result.is_err())
        }

        #[test]
        fn bad_first_offset() {
            let result: Result<Vec<Vec<u16>>, _> =
                deserialize_variable_sized_items(&[88, 0, 0, 0, 1, 2, 3]);
            assert!(result.is_err())
        }

        #[test]
        fn bad_next_offsets() {
            let result: Result<Vec<Vec<u16>>, _> =
                deserialize_variable_sized_items(&[8, 0, 0, 0, 100, 0, 0, 0, 1, 2, 3]);
            assert!(result.is_err())
        }

        #[test]
        fn bad_element_data() {
            let result: Result<Vec<Vec<u16>>, _> =
                deserialize_variable_sized_items(&[8, 0, 0, 0, 9, 0, 0, 0, 1]);
            assert!(result.is_err())
        }
    }
}
