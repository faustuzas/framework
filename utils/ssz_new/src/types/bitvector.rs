use crate::*;
use std::marker::PhantomData;
use typenum::Unsigned;
use std::ops::Index;

pub struct Bitvector<N: Unsigned> {
    bytes: Vec<Byte>,
    _size_meta: PhantomData<N>
}

impl<N: Unsigned> Bitvector<N> {
    pub fn new() -> Self {
        Self {
            bytes: vec![0; (N::to_usize() + 7) / 8],
            _size_meta: PhantomData
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        // check that all bits above N are 0
//        let bytes =

        if bytes.len() <= N::to_usize() {
            // check

            Ok(Self {
                bytes: bytes.to_vec(),
                _size_meta: PhantomData
            })
        } else {
            Err(Error::BitsOverflow {
                bits_count: bytes.len() * 8,
                max_bits: N::to_usize()
            })
        }
    }
}

impl<N: Unsigned> Index<usize> for Bitvector<N> {
    type Output = Byte;

    fn index(&self, index: usize) -> &Self::Output {
        &self.bytes[index]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use typenum::{U5, U16};

    #[test]
    fn new() {
        let bitfield: Bitvector<U16> = Bitvector::new();
        assert_eq!(bitfield.bytes, vec![0; 2]);

        let bitfield: Bitvector<U5> = Bitvector::new();
        assert_eq!(bitfield.bytes, vec![0; 1]);
    }
    
    #[test]
    fn from_bytes() {
//        let bitfield: Bitvector<U5> = Bitvector::from_bytes();
    }
}
