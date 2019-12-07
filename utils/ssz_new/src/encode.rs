use crate::*;
use crate::utils::serialize_offset;

macro_rules! serialize_for_uintn {
    ( $($type_ident: ty),* ) => { $(
        impl Serialize for $type_ident {
            fn serialize(&self) -> Result<Vec<u8>, Error> {
                Ok(self.to_le_bytes().to_vec())
            }

            fn is_variable_size() -> bool {
                false
            }
        }
    )* };
}

serialize_for_uintn!(u8, u16, u32, u64);

impl Serialize for bool {
    fn serialize(&self) -> Result<Vec<u8>, Error> {
        let byte = if *self {
            0b0000_0001
        } else {
            0b0000_0000
        };

        Ok(vec![byte])
    }

    fn is_variable_size() -> bool {
        false
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize(&self) -> Result<Vec<u8>, Error> {
        let mut fixed_parts = Vec::with_capacity(self.len());
        for element in self {
            fixed_parts.push(if !T::is_variable_size() {
                Some(element.serialize()?)
            } else {
                None
            });
        }

        let mut variable_parts = Vec::with_capacity(self.len());
        for element in self {
            variable_parts.push(if T::is_variable_size() {
                element.serialize()?
            } else {
                vec![]
            });
        }

        let fixed_length: usize = fixed_parts.iter()
            .map(|part| match part {
                Some(bytes) => bytes.len(),
                None => BYTES_PER_LENGTH_OFFSET
            }).sum();

        let variable_lengths: Vec<usize> = variable_parts.iter()
            .map(|part| part.len())
            .collect();

        let mut variable_offsets = Vec::with_capacity(self.len());
        for i in 0..self.len() {
            let variable_length_sum: usize = variable_lengths[..i].iter().sum();
            let offset = fixed_length + variable_length_sum;
            variable_offsets.push(serialize_offset(offset)?);
        }

        let fixed_parts: Vec<&Vec<u8>> = fixed_parts.iter()
            .enumerate()
            .map(|(i, part)| match part {
                Some(bytes) => bytes,
                None => &variable_offsets[i]
            }).collect();

        let variable_lengths_sum: usize = variable_lengths.iter().sum();
        let total_bytes = fixed_length + variable_lengths_sum;
        let mut result = Vec::with_capacity(total_bytes);

        for part in fixed_parts {
            result.extend(part);
        }

        for part in variable_parts {
            result.extend(part);
        }

        Ok(result)
    }

    fn is_variable_size() -> bool {
        true
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn u8() {
        assert_eq!(0u8.serialize().unwrap(), vec![0b0000_0000]);
        assert_eq!(u8::max_value().serialize().unwrap(), vec![0b1111_1111]);
        assert_eq!(1u8.serialize().unwrap(), vec![0b0000_0001]);
        assert_eq!(128u8.serialize().unwrap(), vec![0b1000_0000]);
    }

    #[test]
    fn u16() {
        assert_eq!(0u16.serialize().unwrap(), vec![0b0000_0000, 0b0000_0000]);
        assert_eq!(u16::max_value().serialize().unwrap(), vec![0b1111_1111, 0b1111_1111]);
        assert_eq!(1u16.serialize().unwrap(), vec![0b0000_0001, 0b0000_0000]);
        assert_eq!(128u16.serialize().unwrap(), vec![0b1000_0000, 0b0000_0000]);
        assert_eq!(32768u16.serialize().unwrap(), vec![0b0000_0000, 0b1000_0000]);
    }

    #[test]
    fn u32() {
        assert_eq!(0u32.serialize().unwrap(), vec![0b0000_0000; 4]);
        assert_eq!(u32::max_value().serialize().unwrap(), vec![0b1111_1111; 4]);
        assert_eq!(1u32.serialize().unwrap(), vec![0b0000_0001, 0b0000_0000, 0b0000_0000, 0b0000_0000]);
        assert_eq!(128u32.serialize().unwrap(), vec![0b1000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000]);
        assert_eq!(32768u32.serialize().unwrap(), vec![0b0000_0000, 0b1000_0000, 0b0000_0000, 0b0000_0000]);
        assert_eq!(2147483648u32.serialize().unwrap(),
                   vec![0b0000_0000, 0b0000_0000, 0b0000_0000, 0b1000_0000]
        );
    }

    #[test]
    fn u64() {
        assert_eq!(0u64.serialize().unwrap(), vec![0b0000_0000; 8]);
        assert_eq!(u64::max_value().serialize().unwrap(), vec![0b1111_1111; 8]);
        assert_eq!(1u64.serialize().unwrap(),
                   vec![0b0000_0001, 0b0000_0000, 0b0000_0000, 0b0000_0000,
                        0b0000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000]
        );
        assert_eq!(128u64.serialize().unwrap(),
                   vec![0b1000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000,
                   0b0000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000]
        );
        assert_eq!(32768u64.serialize().unwrap(),
                   vec![0b0000_0000, 0b1000_0000, 0b0000_0000, 0b0000_0000,
                        0b0000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000]
        );
        assert_eq!(2147483648u64.serialize().unwrap(),
                   vec![0b0000_0000, 0b0000_0000, 0b0000_0000, 0b1000_0000,
                        0b0000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000]
        );
        assert_eq!(9223372036854775808u64.serialize().unwrap(),
                   vec![0b0000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000,
                        0b0000_0000, 0b0000_0000, 0b0000_0000, 0b1000_0000]
        );
    }

    #[test]
    fn bool() {
        assert_eq!(true.serialize().unwrap(), vec![0b0000_0001]);
        assert_eq!(false.serialize().unwrap(), vec![0b0000_0000]);
    }

    #[test]
    fn vector_fixed() {
        let vec: Vec<u8> = vec![];
        assert_eq!(vec.serialize().unwrap(), vec![]);

        let vec: Vec<u8> = vec![0, 1, 2, 3];
        assert_eq!(vec.serialize().unwrap(), vec![0, 1, 2, 3]);

        let vec: Vec<u8> = vec![u8::max_value(); 100];
        assert_eq!(vec.serialize().unwrap(), vec![u8::max_value(); 100]);

        let vec: Vec<u16> = vec![];
        assert_eq!(vec.serialize().unwrap(), vec![]);

        let vec: Vec<u16> = vec![1, 2, 3, 4];
        assert_eq!(vec.serialize().unwrap(), vec![1, 0, 2, 0, 3, 0, 4, 0]);

        let vec: Vec<u16> = vec![u16::max_value(); 100];
        assert_eq!(vec.serialize().unwrap(), vec![u8::max_value(); 200]);

        let vec: Vec<u32> = vec![];
        assert_eq!(vec.serialize().unwrap(), vec![]);

        let vec: Vec<u32> = vec![1, 2, 3, 4];
        assert_eq!(vec.serialize().unwrap(), vec![1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0]);

        let vec: Vec<u32> = vec![u32::max_value(); 100];
        assert_eq!(vec.serialize().unwrap(), vec![u8::max_value(); 400]);

        let vec: Vec<u64> = vec![];
        assert_eq!(vec.serialize().unwrap(), vec![]);

        let vec: Vec<u64> = vec![1, 2, 3, 4];
        assert_eq!(vec.serialize().unwrap(),
                   vec![1, 0, 0, 0, 0, 0, 0, 0,
                        2, 0, 0, 0, 0, 0, 0, 0,
                        3, 0, 0, 0, 0, 0, 0, 0,
                        4, 0, 0, 0, 0, 0, 0, 0]);

        let vec: Vec<u64> = vec![u64::max_value(); 100];
        assert_eq!(vec.serialize().unwrap(), vec![u8::max_value(); 800]);
    }

    #[test]
    fn vector_variable() {
        let vec: Vec<Vec<u8>> = vec![];
        assert_eq!(vec.serialize().unwrap(), vec![]);

        let vec: Vec<Vec<u8>> = vec![vec![], vec![]];
        assert_eq!(vec.serialize().unwrap(), vec![8, 0, 0, 0, 8, 0, 0, 0]);

        let vec: Vec<Vec<u8>> = vec![vec![1, 2, 3], vec![4, 5, 6]];
        assert_eq!(vec.serialize().unwrap(), vec![8, 0, 0, 0, 11, 0, 0, 0, 1, 2, 3, 4, 5, 6]);
    }
}
