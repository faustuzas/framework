use super::*;

impl<T: ssz::Serialize, N: Unsigned> ssz::Serialize for FixedVector<T, N> {
    fn serialize(&self) -> Result<Vec<u8>, ssz::Error> {
        if T::is_variable_size() {
            let mut variable_parts = Vec::with_capacity(self.len());
            for element in self.iter() {
                variable_parts.push(element.serialize()?)
            }

            let fixed_length = self.len() * ssz::BYTES_PER_LENGTH_OFFSET;
            let variable_lengths: Vec<usize> =
                variable_parts.iter().map(|part| part.len()).collect();

            let mut variable_offsets = Vec::with_capacity(self.len());
            for i in 0..self.len() {
                let variable_length_sum: usize = variable_lengths[..i].iter().sum();
                let offset = fixed_length + variable_length_sum;
                variable_offsets.push(ssz::serialize_offset(offset)?);
            }

            let variable_len: usize = variable_lengths.iter().sum();
            let total_len = fixed_length + variable_len;
            let mut result = Vec::with_capacity(total_len);
            for offset in variable_offsets {
                result.extend(offset);
            }

            for part in variable_parts {
                result.extend(part);
            }

            Ok(result)
        } else {
            let mut fixed_parts = Vec::with_capacity(self.len());
            for element in self.iter() {
                fixed_parts.push(element.serialize()?);
            }

            let len = fixed_parts.iter().map(|part| part.len()).sum();

            let mut result = Vec::with_capacity(len);
            for part in fixed_parts {
                result.extend(part);
            }

            Ok(result)
        }
    }

    fn is_variable_size() -> bool {
        <T as ssz::Serialize>::is_variable_size()
    }
}

impl<T: ssz::Deserialize + Default, N: Unsigned> ssz::Deserialize for FixedVector<T, N> {
    fn deserialize(bytes: &[u8]) -> Result<Self, ssz::Error> {
        if bytes.is_empty() {
            return Err(ssz::Error::InvalidByteLength {
                got: 0,
                required: T::fixed_length(),
            });
        }

        let items_count = N::to_usize();
        if <T as ssz::Deserialize>::is_variable_size() {
            let items = ssz::deserialize_variable_sized_items(bytes)?;

            if items_count == items.len() {
                Ok(items.into())
            } else {
                Err(ssz::Error::InvalidBytes(format!(
                    "Cannot parse FixedVector[{}] from bytes",
                    items_count
                )))
            }
        } else {
            if bytes.len() % items_count == 0 {
                let mut result = Vec::with_capacity(items_count);
                for chunk in bytes.chunks(T::fixed_length()) {
                    result.push(T::deserialize(chunk)?);
                }

                Ok(result.into())
            } else {
                Err(ssz::Error::InvalidByteLength {
                    got: bytes.len(),
                    required: bytes.len() / T::fixed_length() + 1,
                })
            }
        }
    }

    fn is_variable_size() -> bool {
        <T as ssz::Deserialize>::is_variable_size()
    }

    fn fixed_length() -> usize {
        if <T as ssz::Deserialize>::is_variable_size() {
            ssz::BYTES_PER_LENGTH_OFFSET
        } else {
            N::to_usize() * <T as ssz::Deserialize>::fixed_length()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ssz::Serialize;

    mod serialize {
        use super::*;

        #[test]
        fn fixed() {
            let vec: FixedVector<u16, typenum::U3> = FixedVector::from(vec![1, 2, 3]);
            assert_eq!(vec.serialize().unwrap(), vec![1, 0, 2, 0, 3, 0]);
            let vec: FixedVector<u16, typenum::U5> = FixedVector::from(vec![1, 2, 3]);
            assert_eq!(vec.serialize().unwrap(), vec![1, 0, 2, 0, 3, 0, 0, 0, 0, 0]);
        }

        #[test]
        fn variable() {
            let vec: FixedVector<Vec<u8>, typenum::U3> =
                FixedVector::from(vec![vec![1, 2], vec![], vec![3]]);
            assert_eq!(
                vec.serialize().unwrap(),
                vec![12, 0, 0, 0, 14, 0, 0, 0, 14, 0, 0, 0, 1, 2, 3]
            );

            let vec: FixedVector<Vec<u8>, typenum::U5> =
                FixedVector::from(vec![vec![1, 2], vec![], vec![3, 4, 5]]);
            assert_eq!(
                vec.serialize().unwrap(),
                vec![
                    20, 0, 0, 0, 22, 0, 0, 0, 22, 0, 0, 0, 25, 0, 0, 0, 25, 0, 0, 0, 1, 2, 3, 4, 5
                ]
            );
        }
    }

    mod deserialize {
        use super::*;
        use ssz::Deserialize;
        use typenum::{U3, U5, U6};

        #[test]
        fn fixed() {
            let vec =
                <FixedVector<u16, U3> as Deserialize>::deserialize(&[5, 0, 2, 0, 3, 0]).unwrap();
            assert_eq!(vec.to_vec(), vec![5, 2, 3]);
            let vec =
                <FixedVector<u8, U6> as Deserialize>::deserialize(&[5, 0, 2, 0, 3, 0]).unwrap();
            assert_eq!(vec.to_vec(), vec![5, 0, 2, 0, 3, 0]);
        }

        #[test]
        fn variable() {
            let vec = <FixedVector<Vec<u8>, U3> as Deserialize>::deserialize(&[
                12, 0, 0, 0, 14, 0, 0, 0, 14, 0, 0, 0, 1, 2, 3,
            ])
            .unwrap();

            assert_eq!(vec.to_vec(), vec![vec![1, 2], vec![], vec![3]]);

            let vec = <FixedVector<Vec<u8>, U5> as Deserialize>::deserialize(&[
                20, 0, 0, 0, 22, 0, 0, 0, 22, 0, 0, 0, 25, 0, 0, 0, 25, 0, 0, 0, 1, 2, 3, 4, 5,
            ])
            .unwrap();
            assert_eq!(
                vec.to_vec(),
                vec![vec![1, 2], vec![], vec![3, 4, 5], vec![], vec![]]
            );
        }

        mod errors {
            use super::*;

            #[test]
            fn wrong_size() {
                let result = <FixedVector<u8, U6> as Deserialize>::deserialize(&[1, 2, 3, 4]);
                assert!(result.is_err());

                let result = <FixedVector<Vec<u8>, U6> as Deserialize>::deserialize(&[
                    12, 0, 0, 0, 14, 0, 0, 0, 14, 0, 0, 0, 1, 2, 3,
                ]);
                assert!(result.is_err());
            }
        }
    }
}
