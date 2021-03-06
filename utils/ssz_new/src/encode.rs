#![allow(clippy::use_self)] // there is probably a bug with generic vectors

use crate::utils::serialize_offset;
use crate::*;

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
        let byte = if *self { 0b0000_0001 } else { 0b0000_0000 };

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
            fixed_parts.push(if T::is_variable_size() {
                None
            } else {
                Some(element.serialize()?)
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

        let fixed_length: usize = fixed_parts
            .iter()
            .map(|part| match part {
                Some(bytes) => bytes.len(),
                None => BYTES_PER_LENGTH_OFFSET,
            })
            .sum();

        let variable_lengths: Vec<usize> = variable_parts.iter().map(std::vec::Vec::len).collect();

        let mut variable_offsets = Vec::with_capacity(self.len());
        for i in 0..self.len() {
            let variable_length_sum: usize = variable_lengths[..i].iter().sum();
            let offset = fixed_length + variable_length_sum;
            variable_offsets.push(serialize_offset(offset)?);
        }

        let fixed_parts: Vec<&Vec<u8>> = fixed_parts
            .iter()
            .enumerate()
            .map(|(i, part)| match part {
                Some(bytes) => bytes,
                None => &variable_offsets[i],
            })
            .collect();

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
        assert_eq!(0_u8.serialize().expect("Test"), vec![0b0000_0000]);
        assert_eq!(
            u8::max_value().serialize().expect("Test"),
            vec![0b1111_1111]
        );
        assert_eq!(1_u8.serialize().expect("Test"), vec![0b0000_0001]);
        assert_eq!(128_u8.serialize().expect("Test"), vec![0b1000_0000]);
    }

    #[test]
    fn u16() {
        assert_eq!(
            0_u16.serialize().expect("Test"),
            vec![0b0000_0000, 0b0000_0000]
        );
        assert_eq!(
            1_u16.serialize().expect("Test"),
            vec![0b0000_0001, 0b0000_0000]
        );
        assert_eq!(
            128_u16.serialize().expect("Test"),
            vec![0b1000_0000, 0b0000_0000]
        );
        assert_eq!(
            u16::max_value().serialize().expect("Test"),
            vec![0b1111_1111, 0b1111_1111]
        );
        assert_eq!(
            0x8000_u16.serialize().expect("Test"),
            vec![0b0000_0000, 0b1000_0000]
        );
    }

    #[test]
    fn u32() {
        assert_eq!(0_u32.serialize().expect("Test"), vec![0b0000_0000; 4]);
        assert_eq!(
            u32::max_value().serialize().expect("Test"),
            vec![0b1111_1111; 4]
        );
        assert_eq!(
            1_u32.serialize().expect("Test"),
            vec![0b0000_0001, 0b0000_0000, 0b0000_0000, 0b0000_0000]
        );
        assert_eq!(
            128_u32.serialize().expect("Test"),
            vec![0b1000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000]
        );
        assert_eq!(
            0x8000_u32.serialize().expect("Test"),
            vec![0b0000_0000, 0b1000_0000, 0b0000_0000, 0b0000_0000]
        );
        assert_eq!(
            0x8000_0000_u32.serialize().expect("Test"),
            vec![0b0000_0000, 0b0000_0000, 0b0000_0000, 0b1000_0000]
        );
    }

    #[test]
    fn u64() {
        assert_eq!(0_u64.serialize().expect("Test"), vec![0b0000_0000; 8]);
        assert_eq!(
            u64::max_value().serialize().expect("Test"),
            vec![0b1111_1111; 8]
        );
        assert_eq!(
            1_u64.serialize().expect("Test"),
            vec![
                0b0000_0001,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000
            ]
        );
        assert_eq!(
            128_u64.serialize().expect("Test"),
            vec![
                0b1000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000
            ]
        );
        assert_eq!(
            0x8000_u64.serialize().expect("Test"),
            vec![
                0b0000_0000,
                0b1000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000
            ]
        );
        assert_eq!(
            0x8000_0000_u64.serialize().expect("Test"),
            vec![
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b1000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000
            ]
        );
        assert_eq!(
            0x8000_0000_0000_0000_u64.serialize().expect("Test"),
            vec![
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b1000_0000
            ]
        );
    }

    #[test]
    fn bool() {
        assert_eq!(true.serialize().expect("Test"), vec![0b0000_0001]);
        assert_eq!(false.serialize().expect("Test"), vec![0b0000_0000]);
    }

    #[test]
    fn vector_fixed() {
        let vec: Vec<u8> = vec![];
        assert_eq!(vec.serialize().expect("Test"), vec![]);

        let vec: Vec<u8> = vec![0, 1, 2, 3];
        assert_eq!(vec.serialize().expect("Test"), vec![0, 1, 2, 3]);

        let vec: Vec<u8> = vec![u8::max_value(); 100];
        assert_eq!(vec.serialize().expect("Test"), vec![u8::max_value(); 100]);

        let vec: Vec<u16> = vec![];
        assert_eq!(vec.serialize().expect("Test"), vec![]);

        let vec: Vec<u16> = vec![1, 2, 3, 4];
        assert_eq!(vec.serialize().expect("Test"), vec![1, 0, 2, 0, 3, 0, 4, 0]);

        let vec: Vec<u16> = vec![u16::max_value(); 100];
        assert_eq!(vec.serialize().expect("Test"), vec![u8::max_value(); 200]);

        let vec: Vec<u32> = vec![];
        assert_eq!(vec.serialize().expect("Test"), vec![]);

        let vec: Vec<u32> = vec![1, 2, 3, 4];
        assert_eq!(
            vec.serialize().expect("Test"),
            vec![1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0]
        );

        let vec: Vec<u32> = vec![u32::max_value(); 100];
        assert_eq!(vec.serialize().expect("Test"), vec![u8::max_value(); 400]);

        let vec: Vec<u64> = vec![];
        assert_eq!(vec.serialize().expect("Test"), vec![]);

        let vec: Vec<u64> = vec![1, 2, 3, 4];
        assert_eq!(
            vec.serialize().expect("Test"),
            vec![
                1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0,
                0, 0, 0, 0
            ]
        );

        let vec: Vec<u64> = vec![u64::max_value(); 100];
        assert_eq!(vec.serialize().expect("Test"), vec![u8::max_value(); 800]);
    }

    #[test]
    fn vector_variable() {
        let vec: Vec<Vec<u8>> = vec![];
        assert_eq!(vec.serialize().expect("Test"), vec![]);

        let vec: Vec<Vec<u8>> = vec![vec![], vec![]];
        assert_eq!(vec.serialize().expect("Test"), vec![8, 0, 0, 0, 8, 0, 0, 0]);

        let vec: Vec<Vec<u8>> = vec![vec![1, 2, 3], vec![4, 5, 6]];
        assert_eq!(
            vec.serialize().expect("Test"),
            vec![8, 0, 0, 0, 11, 0, 0, 0, 1, 2, 3, 4, 5, 6]
        );
    }
}
