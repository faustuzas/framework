use super::tree_hash::vec_tree_hash_root;
use super::Error;
use serde_derive::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::ops::{Deref, Index, IndexMut};
use std::slice::SliceIndex;
use typenum::Unsigned;

pub use typenum;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FixedVector<T, N> {
    vec: Vec<T>,
    _meta: PhantomData<N>,
}

impl<T, N: Unsigned> FixedVector<T, N> {
    pub fn new(vec: Vec<T>) -> Result<Self, Error> {
        if vec.len() == Self::capacity() {
            Ok(Self {
                vec,
                _meta: PhantomData
            })
        } else {
            Err(Error::OutOfBounds { i: vec.len(), len: Self::capacity() })
        }
    }

    pub fn from_elem(elem: T) -> Self where T: Clone {
        Self {
            vec: vec![elem; Self::capacity()],
            _meta: PhantomData
        }
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool { self.len() == 0 }

    pub fn capacity() -> usize { N::to_usize() }
}

impl<T: Default, N: Unsigned> From<Vec<T>> for FixedVector<T, N> {
    fn from(mut vec: Vec<T>) -> Self {
        vec.resize_with(Self::capacity(), Default::default);

        Self {
            vec,
            _meta: PhantomData
        }
    }
}

impl<T, N: Unsigned> Into<Vec<T>> for FixedVector<T, N> {
    fn into(self) -> Vec<T> {
        self.vec
    }
}

impl<T, N: Unsigned> Default for FixedVector<T, N> {
    fn default() -> Self {
        Self {
            vec: Vec::default(),
            _meta: PhantomData,
        }
    }
}

impl<T, N: Unsigned, I: SliceIndex<[T]>> Index<I> for FixedVector<T, N> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        Index::index(&self.vec, index)
    }
}

impl<T, N: Unsigned, I: SliceIndex<[T]>> IndexMut<I> for FixedVector<T, N> {

    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut self.vec, index)
    }
}

impl<T, N: Unsigned> Deref for FixedVector<T, N> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.vec[..]
    }
}

impl<T: ssz::Encode, N: Unsigned> ssz::Encode for FixedVector<T, N> {
    fn is_ssz_fixed_len() -> bool {
        T::is_ssz_fixed_len()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        if T::is_ssz_fixed_len() {
            buf.reserve(Self::ssz_fixed_len());

            for el in &self.vec {
                el.ssz_append(buf)
            }
        } else {
            let mut encoder = ssz::SszEncoder::list(buf, self.len() * ssz::BYTES_PER_LENGTH_OFFSET);

            for el in &self.vec {
                encoder.append(el);
            }

            encoder.finalize();
        }
    }

    fn ssz_fixed_len() -> usize {
        if Self::is_ssz_fixed_len() {
            N::to_usize() * T::ssz_fixed_len()
        } else {
            ssz::BYTES_PER_LENGTH_OFFSET
        }
    }

    fn ssz_bytes_len(&self) -> usize {
        self.vec.ssz_bytes_len()
    }
}

impl<T: ssz::Decode + Default, N: Unsigned> ssz::Decode for FixedVector<T, N> {
    fn is_ssz_fixed_len() -> bool {
        T::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
       if Self::is_ssz_fixed_len() {
           N::to_usize() * T::ssz_fixed_len()
       } else {
            ssz::BYTES_PER_LENGTH_OFFSET
       }
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        if bytes.is_empty() {
            Err(ssz::DecodeError::InvalidByteLength {
                len: 0, expected: N::to_usize() * T::ssz_fixed_len()
            })
        } else if T::is_ssz_fixed_len() {
            let items_result = bytes
                .chunks(T::ssz_fixed_len())
                .map(|chunk| T::from_ssz_bytes(chunk))
                .collect::<Result<Vec<T>, _>>();

            match items_result {
                Ok(items) => {
                    if items.len() == N::to_usize() {
                        Ok(items.into())
                    } else {
                        Err(ssz::DecodeError::BytesInvalid(format!(
                            "Wrong number of items parsed. Got: {}, expected: {}",
                            items.len(), N::to_usize()
                        )))
                    }
                },
                Err(err) => Err(err)
            }
        } else {
            ssz::decode_list_of_variable_length_items(bytes)
                .and_then(|items| Ok(items.into()))
        }
    }
}

impl<T: tree_hash::TreeHash, N: Unsigned> tree_hash::TreeHash for FixedVector<T, N> {
    fn tree_hash_type() -> tree_hash::TreeHashType {
        tree_hash::TreeHashType::Vector
    }

    fn tree_hash_packed_encoding(&self) -> Vec<u8> {
        unreachable!("Vector should not be packed.")
    }

    fn tree_hash_packing_factor() -> usize {
        unreachable!("Vector should not be packed.")
    }

    fn tree_hash_root(&self) -> Vec<u8> {
        vec_tree_hash_root::<T, N>(&self.vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typenum::*;
    use ssz::*;

    #[test]
    fn test_new() {
        let items = vec![1, 2, 3, 4, 5];
        let vector_result: Result<FixedVector<i32, U5>, _> = FixedVector::new(items.clone());
        assert!(vector_result.is_ok());
        assert_eq!(vector_result.unwrap().vec, items);
    }

    #[test]
    fn test_new_error() {
        let vector_result: Result<FixedVector<i32, U3>, _> = FixedVector::new( vec![1, 2, 3, 4, 5]);
        assert_eq!(vector_result, Err(Error::OutOfBounds {
            i: 5,
            len: 3
        }));
    }

    #[test]
    fn test_from_elem() {
        let vector: FixedVector<i32, U10> = FixedVector::from_elem(5);
        assert_eq!(vector.vec, vec![5; 10]);
    }

    #[test]
    fn test_from_into() {
        let vector: FixedVector<i32, U4> = FixedVector::from(vec![0, 1, 2, 3]);
        assert_eq!(vector.len(), 4);
        assert_eq!(vector.vec, vec![0, 1, 2, 3]);

        let vec: Vec<i32> = vector.into();
        assert_eq!(vec, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_default() {
        let vector: FixedVector<i32, U0> = FixedVector::default();
        assert_eq!(vector.len(), 0);
        assert_eq!(vector.vec, vec![]);
    }

    #[test]
    fn test_index() {
        let vector: FixedVector<usize, U4> = FixedVector::from(vec![0, 1, 2, 3]);
        for i in 0..4 {
            assert_eq!(vector[i], i);
        }
    }

    #[test]
    fn test_index_mut() {
        let mut vector: FixedVector<usize, U4> = FixedVector::from(vec![0, 1, 2, 3]);
        for i in 0..4 {
            vector[i] += 2;
            assert_eq!(vector[i], i + 2);
        }
    }

    #[test]
    fn test_deref() {
        let vector: FixedVector<i32, U4> = FixedVector::from(vec![0, 1, 2, 3]);
        let slice = [0, 1, 2, 3];
        assert_eq!(*vector, slice);
    }

    #[test]
    fn test_into_iter() {
        let vec_from_vector: Vec<i32> = <FixedVector<i32, U4>>::from(vec![0, 1, 2, 3])
            .into_iter()
            .map(|el| el * el)
            .collect();

        assert_eq!(vec_from_vector, vec![0, 1, 4, 9]);
    }

    #[test]
    fn test_ssz_round_trip() {
        let vector: FixedVector<u16, U4> = FixedVector::from(vec![1, 2, 3, 4]);
        let decoded_res = <FixedVector<u16, U4>>::from_ssz_bytes(vector.as_ssz_bytes().as_slice());
        assert!(decoded_res.is_ok());
        assert_eq!(decoded_res.unwrap(), vector)
    }

    #[test]
    fn test_ssz_decode_error() {
        // 0 bytes passed
        assert_eq!(<FixedVector<u8, U4>>::from_ssz_bytes(&[]), Err(DecodeError::InvalidByteLength {
            len: 0, expected: 4
        }));

        // incorrect amount of bytes passed
        assert_eq!(<FixedVector<u16, U4>>::from_ssz_bytes(&[0, 1, 0, 2, 0, 3]), Err(ssz::DecodeError::BytesInvalid(
            "Wrong number of items parsed. Got: 3, expected: 4".to_string()
        )));

        // invalid bytes passed
        assert_eq!(<FixedVector<bool, U4>>::from_ssz_bytes(&[0, 2]), Err(DecodeError::BytesInvalid(
            "Invalid value for boolean: 2".to_string())
        ));
    }
}