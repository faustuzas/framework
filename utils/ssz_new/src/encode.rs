#![allow(clippy::use_self)]

use crate::utils::*;
use crate::*;
use core::num::NonZeroUsize;
use ethereum_types::{H256, U128, U256};

macro_rules! encode_for_uintn {
    ( $(($type_ident: ty, $size_in_bits: expr)),* ) => { $(
        impl Encode for $type_ident {
            fn ssz_append(&self, buf: &mut Vec<u8>) {
                buf.extend_from_slice(&self.to_le_bytes());
            }

            fn is_ssz_fixed_len() -> bool {
                true
            }

            fn ssz_bytes_len(&self) -> usize {
                <Self as Encode>::ssz_fixed_len()
            }

            fn ssz_fixed_len() -> usize {
                 $size_in_bits / 8
            }
        }
    )* };
}

encode_for_uintn!(
    (u8, 8),
    (u16, 16),
    (u32, 32),
    (u64, 64),
    (usize, std::mem::size_of::<usize>() * 8)
);

macro_rules! encode_for_u8_array {
    ($size: expr) => {
        impl Encode for [u8; $size] {
            fn ssz_append(&self, buf: &mut Vec<u8>) {
                buf.extend_from_slice(&self[..]);
            }

            fn is_ssz_fixed_len() -> bool {
                true
            }

            fn ssz_bytes_len(&self) -> usize {
                <Self as Encode>::ssz_fixed_len()
            }

            fn ssz_fixed_len() -> usize {
                $size
            }
        }
    };
}

encode_for_u8_array!(4);
encode_for_u8_array!(32);

impl Encode for bool {
    fn ssz_append(&self, buf: &mut Vec<u8>) {
        let byte = if *self { 0b0000_0001 } else { 0b0000_0000 };
        buf.push(byte);
    }

    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_bytes_len(&self) -> usize {
        <Self as Encode>::ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        1
    }
}

impl<T: Encode> Encode for Vec<T> {
    fn ssz_append(&self, buf: &mut Vec<u8>) {
        let mut fixed_parts = Vec::with_capacity(self.len());
        for element in self {
            fixed_parts.push(if T::is_ssz_fixed_len() {
                Some(element.as_ssz_bytes())
            } else {
                None
            });
        }

        let mut variable_parts = Vec::with_capacity(self.len());
        for element in self {
            variable_parts.push(if T::is_ssz_fixed_len() {
                vec![]
            } else {
                element.as_ssz_bytes()
            });
        }

        encode_items_from_parts(buf, &fixed_parts, &variable_parts);
    }

    fn is_ssz_fixed_len() -> bool {
        false
    }
}

impl<T: Encode> Encode for Option<T> {
    fn ssz_append(&self, buf: &mut Vec<u8>) {
        match self {
            None => buf.append(&mut encode_offset(0)),
            Some(t) => {
                buf.append(&mut encode_offset(1));
                buf.append(&mut t.as_ssz_bytes());
            }
        }
    }

    fn is_ssz_fixed_len() -> bool {
        false
    }
}

impl Encode for NonZeroUsize {
    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.append(&mut self.get().as_ssz_bytes());
    }

    fn is_ssz_fixed_len() -> bool {
        <usize as Encode>::is_ssz_fixed_len()
    }

    fn ssz_bytes_len(&self) -> usize {
        <Self as Encode>::ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        <usize as Encode>::ssz_fixed_len()
    }
}

impl Encode for H256 {
    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_bytes())
    }

    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_bytes_len(&self) -> usize {
        <Self as Encode>::ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        32
    }
}

impl Encode for U256 {
    fn ssz_append(&self, buf: &mut Vec<u8>) {
        let current_len = buf.len();

        buf.resize(current_len + 32, 0);
        self.to_little_endian(&mut buf[current_len..]);
    }

    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_bytes_len(&self) -> usize {
        <Self as Encode>::ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        32
    }
}

impl Encode for U128 {
    fn ssz_append(&self, buf: &mut Vec<u8>) {
        let current_len = buf.len();

        buf.resize(current_len + 16, 0);
        self.to_little_endian(&mut buf[current_len..]);
    }

    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_bytes_len(&self) -> usize {
        <Self as Encode>::ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        16
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn u8() {
        assert_eq!(0_u8.as_ssz_bytes(), vec![0b0000_0000]);
        assert_eq!(u8::max_value().as_ssz_bytes(), vec![0b1111_1111]);
        assert_eq!(1_u8.as_ssz_bytes(), vec![0b0000_0001]);
        assert_eq!(128_u8.as_ssz_bytes(), vec![0b1000_0000]);

        assert_eq!(1_u8.ssz_bytes_len(), 1);
        assert_eq!(<u8 as Encode>::ssz_fixed_len(), 1);
        assert!(<u8 as Encode>::is_ssz_fixed_len());
    }

    #[test]
    fn u16() {
        assert_eq!(0_u16.as_ssz_bytes(), vec![0b0000_0000, 0b0000_0000]);
        assert_eq!(1_u16.as_ssz_bytes(), vec![0b0000_0001, 0b0000_0000]);
        assert_eq!(128_u16.as_ssz_bytes(), vec![0b1000_0000, 0b0000_0000]);
        assert_eq!(
            u16::max_value().as_ssz_bytes(),
            vec![0b1111_1111, 0b1111_1111]
        );
        assert_eq!(0x8000_u16.as_ssz_bytes(), vec![0b0000_0000, 0b1000_0000]);

        assert_eq!(1_u16.ssz_bytes_len(), 2);
        assert_eq!(<u16 as Encode>::ssz_fixed_len(), 2);
        assert!(<u16 as Encode>::is_ssz_fixed_len());
    }

    #[test]
    fn u32() {
        assert_eq!(0_u32.as_ssz_bytes(), vec![0b0000_0000; 4]);
        assert_eq!(u32::max_value().as_ssz_bytes(), vec![0b1111_1111; 4]);
        assert_eq!(
            1_u32.as_ssz_bytes(),
            vec![0b0000_0001, 0b0000_0000, 0b0000_0000, 0b0000_0000]
        );
        assert_eq!(
            128_u32.as_ssz_bytes(),
            vec![0b1000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000]
        );
        assert_eq!(
            0x8000_u32.as_ssz_bytes(),
            vec![0b0000_0000, 0b1000_0000, 0b0000_0000, 0b0000_0000]
        );
        assert_eq!(
            0x8000_0000_u32.as_ssz_bytes(),
            vec![0b0000_0000, 0b0000_0000, 0b0000_0000, 0b1000_0000]
        );

        assert_eq!(1_u32.ssz_bytes_len(), 4);
        assert_eq!(<u32 as Encode>::ssz_fixed_len(), 4);
        assert!(<u32 as Encode>::is_ssz_fixed_len());
    }

    #[test]
    fn u64() {
        assert_eq!(0_u64.as_ssz_bytes(), vec![0b0000_0000; 8]);
        assert_eq!(u64::max_value().as_ssz_bytes(), vec![0b1111_1111; 8]);
        assert_eq!(
            1_u64.as_ssz_bytes(),
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
            128_u64.as_ssz_bytes(),
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
            0x8000_u64.as_ssz_bytes(),
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
            0x8000_0000_u64.as_ssz_bytes(),
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
            0x8000_0000_0000_0000_u64.as_ssz_bytes(),
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

        assert_eq!(1_u64.ssz_bytes_len(), 8);
        assert_eq!(<u64 as Encode>::ssz_fixed_len(), 8);
        assert!(<u64 as Encode>::is_ssz_fixed_len());
    }

    #[test]
    fn usize() {
        let usize_size = std::mem::size_of::<usize>();

        let encoded = 1_usize.as_ssz_bytes();
        assert_eq!(encoded.len(), usize_size);
        for (i, byte) in encoded.iter().enumerate() {
            if i == 0 {
                assert_eq!(*byte, 1)
            } else {
                assert_eq!(*byte, 0)
            }
        }

        assert_eq!(usize::max_value().as_ssz_bytes(), vec![255; usize_size]);

        assert_eq!(1_usize.ssz_bytes_len(), usize_size);
        assert_eq!(<usize as Encode>::ssz_fixed_len(), usize_size);
        assert!(<usize as Encode>::is_ssz_fixed_len());
    }

    #[test]
    fn non_zero_usize() {
        let usize_size = std::mem::size_of::<usize>();

        let nzusize = NonZeroUsize::new(usize::max_value()).expect("Test");
        assert_eq!(nzusize.as_ssz_bytes(), vec![255; usize_size]);

        assert_eq!(nzusize.ssz_bytes_len(), usize_size);
        assert_eq!(<NonZeroUsize as Encode>::ssz_fixed_len(), usize_size);
        assert!(<NonZeroUsize as Encode>::is_ssz_fixed_len());
    }

    #[test]
    fn bool() {
        assert_eq!(true.as_ssz_bytes(), vec![0b0000_0001]);
        assert_eq!(false.as_ssz_bytes(), vec![0b0000_0000]);

        assert_eq!(true.ssz_bytes_len(), 1);
        assert_eq!(<bool as Encode>::ssz_fixed_len(), 1);
        assert!(<bool as Encode>::is_ssz_fixed_len());
    }

    #[test]
    fn vector_fixed() {
        let vec: Vec<u8> = vec![];
        assert_eq!(vec.as_ssz_bytes(), vec![]);

        let vec: Vec<u8> = vec![0, 1, 2, 3];
        assert_eq!(vec.as_ssz_bytes(), vec![0, 1, 2, 3]);

        let vec: Vec<u8> = vec![u8::max_value(); 100];
        assert_eq!(vec.as_ssz_bytes(), vec![u8::max_value(); 100]);

        let vec: Vec<u16> = vec![];
        assert_eq!(vec.as_ssz_bytes(), vec![]);

        let vec: Vec<u16> = vec![1, 2, 3, 4];
        assert_eq!(vec.as_ssz_bytes(), vec![1, 0, 2, 0, 3, 0, 4, 0]);

        let vec: Vec<u16> = vec![u16::max_value(); 100];
        assert_eq!(vec.as_ssz_bytes(), vec![u8::max_value(); 200]);

        let vec: Vec<u32> = vec![];
        assert_eq!(vec.as_ssz_bytes(), vec![]);

        let vec: Vec<u32> = vec![1, 2, 3, 4];
        assert_eq!(
            vec.as_ssz_bytes(),
            vec![1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0]
        );

        let vec: Vec<u32> = vec![u32::max_value(); 100];
        assert_eq!(vec.as_ssz_bytes(), vec![u8::max_value(); 400]);

        let vec: Vec<u64> = vec![];
        assert_eq!(vec.as_ssz_bytes(), vec![]);

        let vec: Vec<u64> = vec![1, 2, 3, 4];
        assert_eq!(
            vec.as_ssz_bytes(),
            vec![
                1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0,
                0, 0, 0, 0
            ]
        );

        let vec: Vec<u64> = vec![u64::max_value(); 100];
        assert_eq!(vec.as_ssz_bytes(), vec![u8::max_value(); 800]);
        assert!(!<Vec<u64> as Encode>::is_ssz_fixed_len());
        assert_eq!(<U256 as Encode>::ssz_fixed_len(), BYTES_PER_LENGTH_OFFSET);
    }

    #[test]
    fn vector_variable() {
        let vec: Vec<Vec<u8>> = vec![];
        assert_eq!(vec.as_ssz_bytes(), vec![]);

        let vec: Vec<Vec<u8>> = vec![vec![], vec![]];
        assert_eq!(vec.as_ssz_bytes(), vec![8, 0, 0, 0, 8, 0, 0, 0]);

        let vec: Vec<Vec<u8>> = vec![vec![1, 2, 3], vec![4, 5, 6]];
        assert_eq!(
            vec.as_ssz_bytes(),
            vec![8, 0, 0, 0, 11, 0, 0, 0, 1, 2, 3, 4, 5, 6]
        );
    }

    #[test]
    fn option() {
        let some = Some(u16::max_value());
        assert_eq!(some.as_ssz_bytes(), vec![1, 0, 0, 0, 255, 255]);

        let none: Option<u16> = None;
        assert_eq!(none.as_ssz_bytes(), vec![0, 0, 0, 0]);

        assert_eq!(none.ssz_bytes_len(), 4);
        assert_eq!(some.ssz_bytes_len(), 6);

        assert_eq!(
            <Option<u16> as Encode>::ssz_fixed_len(),
            BYTES_PER_LENGTH_OFFSET
        );
        assert!(!<Option<u16> as Encode>::is_ssz_fixed_len());
    }

    #[test]
    fn u8_array() {
        assert_eq!([1; 4].as_ssz_bytes(), vec![1; 4]);
        assert_eq!([1; 32].as_ssz_bytes(), vec![1; 32]);

        assert_eq!([1; 4].ssz_bytes_len(), 4);
        assert_eq!([1; 32].ssz_bytes_len(), 32);

        assert_eq!(<[u8; 4] as Encode>::ssz_fixed_len(), 4);
        assert_eq!(<[u8; 32] as Encode>::ssz_fixed_len(), 32);

        assert!(<[u8; 4] as Encode>::is_ssz_fixed_len());
        assert!(<[u8; 32] as Encode>::is_ssz_fixed_len());
    }

    #[test]
    fn h256() {
        assert_eq!(H256::zero().as_ssz_bytes(), vec![0; 32]);

        assert_eq!(H256::zero().ssz_bytes_len(), 32);
        assert_eq!(<H256 as Encode>::ssz_fixed_len(), 32);
        assert!(<H256 as Encode>::is_ssz_fixed_len());
    }

    #[test]
    fn u256() {
        let u = U256::from_dec_str("0").expect("Test");
        assert_eq!(u.as_ssz_bytes(), vec![0; 32]);

        assert_eq!(u.ssz_bytes_len(), 32);
        assert_eq!(<U256 as Encode>::ssz_fixed_len(), 32);
        assert!(<U256 as Encode>::is_ssz_fixed_len());
    }

    #[test]
    fn u128() {
        let u = U128::from_dec_str("0").expect("Test");
        assert_eq!(u.as_ssz_bytes(), vec![0; 16]);

        assert_eq!(u.ssz_bytes_len(), 16);
        assert_eq!(<U128 as Encode>::ssz_fixed_len(), 16);
        assert!(<U128 as Encode>::is_ssz_fixed_len());
    }
}
