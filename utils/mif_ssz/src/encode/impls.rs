use super::*;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_bool() {
        assert_eq!(true.as_ssz_bytes(), vec![1]);
        assert_eq!(false.as_ssz_bytes(), vec![0]);
    }

    #[test]
    fn test_encode_uint_n() {
        assert_eq!((0 as u8).as_ssz_bytes(), vec![0]);
        assert_eq!((0 as u16).as_ssz_bytes(), vec![0; 2]);
        assert_eq!((0 as u32).as_ssz_bytes(), vec![0; 4]);
        assert_eq!((0 as u64).as_ssz_bytes(), vec![0; 8]);

        assert_eq!((100 as u8).as_ssz_bytes(), vec![100]);
        assert_eq!((100 as u16).as_ssz_bytes(), vec![100, 0]);
        assert_eq!((100 as u32).as_ssz_bytes(), vec![100, 0, 0, 0]);
        assert_eq!((100 as u64).as_ssz_bytes(), vec![100, 0, 0, 0, 0, 0, 0, 0]);
    }
}