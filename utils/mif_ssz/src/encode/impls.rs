use super::*;
use std::mem;
use core::num::NonZeroUsize;
use ethereum_types::{H256, U128, U256};

macro_rules! uint_n_encoding_impl {
    ($type: ident, $size: expr) => {
        impl Encode for $type {
            fn is_ssz_fixed_len() -> bool {
                true
            }

            fn ssz_fixed_len() -> usize {
                $size / 8
            }

            fn ssz_bytes_len(&self) -> usize {
                $size / 8
            }

            fn ssz_append(&self, buf: &mut Vec<u8>) {
                buf.extend_from_slice(&self.to_le_bytes());
            }
        }
    };
}

uint_n_encoding_impl!(u8, 8);
uint_n_encoding_impl!(u16, 16);
uint_n_encoding_impl!(u32, 32);
uint_n_encoding_impl!(u64, 64);

#[cfg(target_pointer_width = "32")]
uint_n_encoding_impl!(usize, 32);

#[cfg(target_pointer_width = "64")]
uint_n_encoding_impl!(usize, 64);

impl Encode for bool {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        let value: u8 = if *self { 1 } else { 0 };
        buf.extend_from_slice(&value.to_le_bytes())
    }

    fn ssz_fixed_len() -> usize {
        1
    }

    fn ssz_bytes_len(&self) -> usize {
        1
    }
}

impl <T: Encode> Encode for Vec<T> {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        // vector is made of fixed-length elements
        if T::is_ssz_fixed_len() {
            buf.reserve(T::ssz_fixed_len() * self.len());

            for el in self {
                el.ssz_append(buf);
            }

            return;
        }

        let mut encoder = SszEncoder::list(buf, self.len() * BYTES_PER_LENGTH_OFFSET);
        for el in self {
            encoder.append(el);
        }

        encoder.finalize();
    }

    fn ssz_bytes_len(&self) -> usize {
        if T::is_ssz_fixed_len() {
            <T as Encode>::ssz_fixed_len() * self.len()
        } else {
            let offsets_length = BYTES_PER_LENGTH_OFFSET * self.len();
            let data_length: usize = self.iter().map(|item| item.ssz_bytes_len()).sum();

            offsets_length + data_length
        }
    }
}

/// The SSZ Union type.
impl<T: Encode> Encode for Option<T> {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        match self {
            None => buf.append(&mut encode_union_index(0)),
            Some(encodable) => {
                buf.append(&mut encode_union_index(1));
                encodable.ssz_append(buf);
            }
        }
    }

    fn ssz_bytes_len(&self) -> usize {
        match self {
            None => BYTES_PER_LENGTH_OFFSET,
            Some(encodable) => BYTES_PER_LENGTH_OFFSET +
                if <T as Encode>::is_ssz_fixed_len() {
                    <T as Encode>::ssz_fixed_len()
                } else { encodable.ssz_bytes_len() }
        }
    }
}

impl Encode for NonZeroUsize {
    fn is_ssz_fixed_len() -> bool {
        <usize as Encode>::is_ssz_fixed_len()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        self.get().ssz_append(buf)
    }

    fn ssz_fixed_len() -> usize {
        mem::size_of::<usize>()
    }

    fn ssz_bytes_len(&self) -> usize {
        <usize as Encode>::ssz_fixed_len()
    }
}

/// Raw binary data of fixed length (32 bytes)
impl Encode for H256 {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_bytes())
    }

    fn ssz_fixed_len() -> usize {
        32
    }

    fn ssz_bytes_len(&self) -> usize {
        32
    }
}

/// Little-endian big ints
macro_rules! le_integer_encoding_impl {
    ($type: ident, $size: expr) => {
        impl Encode for $type {
            fn is_ssz_fixed_len() -> bool {
                true
            }

            fn ssz_fixed_len() -> usize {
                $size / 8
            }

            fn ssz_bytes_len(&self) -> usize {
                $size / 8
            }

            fn ssz_append(&self, buf: &mut Vec<u8>) {
                let current_size = buf.len();
                let additional_size = <Self as Encode>::ssz_fixed_len();

                buf.resize(current_size + additional_size, 0);
                self.to_little_endian(&mut buf[current_size..]);
            }
        }
    };
}

le_integer_encoding_impl!(U128, 128);
le_integer_encoding_impl!(U256, 256);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_bool() {
        assert_eq!(true.as_ssz_bytes(), vec![1]);
        assert_eq!(false.as_ssz_bytes(), vec![0]);
    }

    #[test]
    fn test_encode_u8() {
        assert_eq!(0_u8.as_ssz_bytes(), vec![0]);
        assert_eq!(1_u8.as_ssz_bytes(), vec![1]);
        assert_eq!(100_u8.as_ssz_bytes(), vec![100]);
        assert_eq!(255_u8.as_ssz_bytes(), vec![255]);
    }

    #[test]
    fn test_encode_u16() {
        assert_eq!(1_u16.as_ssz_bytes(), vec![1, 0]);
        assert_eq!(100_u16.as_ssz_bytes(), vec![100, 0]);
        assert_eq!((1_u16 << 8).as_ssz_bytes(), vec![0, 1]);
        assert_eq!(65535_u16.as_ssz_bytes(), vec![255, 255]);
    }

    #[test]
    fn test_encode_u32() {
        assert_eq!(1_u32.as_ssz_bytes(), vec![1, 0, 0, 0]);
        assert_eq!(100_u32.as_ssz_bytes(), vec![100, 0, 0, 0]);
        assert_eq!((1_u32 << 16).as_ssz_bytes(), vec![0, 0, 1, 0]);
        assert_eq!((1_u32 << 24).as_ssz_bytes(), vec![0, 0, 0, 1]);
        assert_eq!((!0_u32).as_ssz_bytes(), vec![255, 255, 255, 255]);
    }

    #[test]
    fn test_encode_u64() {
        assert_eq!(1_u64.as_ssz_bytes(), vec![1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            (!0_u64).as_ssz_bytes(),
            vec![255, 255, 255, 255, 255, 255, 255, 255]
        );
    }

    #[test]
    fn test_encode_vec_of_u8() {
        let vec: Vec<u8> = vec![];
        assert_eq!(vec.as_ssz_bytes(), vec![]);

        let vec: Vec<u8> = vec![1];
        assert_eq!(vec.as_ssz_bytes(), vec![1]);

        let vec: Vec<u8> = vec![0, 1, 2, 3];
        assert_eq!(vec.as_ssz_bytes(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_encode_vec_of_vec_of_u8() {
        let vec: Vec<Vec<u8>> = vec![];
        assert_eq!(vec.as_ssz_bytes(), vec![]);

        let vec: Vec<Vec<u8>> = vec![vec![]];
        assert_eq!(vec.as_ssz_bytes(), vec![4, 0, 0, 0]);

        let vec: Vec<Vec<u8>> = vec![vec![], vec![]];
        assert_eq!(vec.as_ssz_bytes(), vec![8, 0, 0, 0, 8, 0, 0, 0]);

        let vec: Vec<Vec<u8>> = vec![vec![0, 1, 2], vec![11, 22, 33]];
        assert_eq!(
            vec.as_ssz_bytes(),
            vec![8, 0, 0, 0, 11, 0, 0, 0, 0, 1, 2, 11, 22, 33]
        );
    }

    #[test]
    fn test_encode_union() {
        assert_eq!(Some(123 as u8).as_ssz_bytes(), vec![1, 0, 0, 0, 123]);
        assert_eq!((None as Option<u8>).as_ssz_bytes(), vec![0; 4]);
    }

    #[test]
    fn test_encode_h256() {
        assert_eq!(H256::zero().as_ssz_bytes(), vec![0; 32]);
        assert_eq!(H256::from_slice(&[1; 32]).as_ssz_bytes(), vec![1; 32]);

        let bytes = vec![
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];

        assert_eq!(H256::from_slice(&bytes).as_ssz_bytes(), bytes);
    }

    #[test]
    fn test_encode_u128() {
        assert_eq!(U128::from_dec_str("0").unwrap().as_ssz_bytes(), vec![0; 16]);

        let bytes = vec![64, 226, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(U128::from_dec_str("123456").unwrap().as_ssz_bytes(), bytes)
    }
}