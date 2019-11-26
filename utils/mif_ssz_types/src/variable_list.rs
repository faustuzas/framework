use crate::tree_hash::vec_tree_hash_root;
use super::Error;
use std::marker::PhantomData;
use typenum::Unsigned;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::slice::SliceIndex;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VariableList<T, C> {
    vec: Vec<T>,
    _meta: PhantomData<C>,
}

impl<T, N: Unsigned> VariableList<T, N> {
    pub fn new(vec: Vec<T>) -> Result<Self, Error> {
        if vec.len() <= Self::max_len() {
            Ok(Self {
                vec,
                _meta: PhantomData
            })
        } else {
            Err(Error::OutOfBounds {
                i: vec.len(),
                len: Self::max_len()
            })
        }
    }

    pub fn empty() -> Self {
        Self {
            vec: vec![],
            _meta: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn max_len() -> usize {
        N::to_usize()
    }

    pub fn push(&mut self, value: T) -> Result<(), Error> {
        if self.vec.len() < Self::max_len() {
            self.vec.push(value);
            Ok(())
        } else {
            Err(Error::OutOfBounds {
                i: self.vec.len() + 1,
                len: Self::max_len()
            })
        }
    }
}

impl <T, N:Unsigned> From<Vec<T>> for VariableList<T, N> {
    fn from(mut vec: Vec<T>) -> Self {
        // shrink vector to required size
        vec.truncate(N::to_usize());

        Self {
            vec,
            _meta: PhantomData
        }
    }
}

impl<T, N: Unsigned> Into<Vec<T>> for VariableList<T, N> {
    fn into(self) -> Vec<T> {
        self.vec
    }
}

impl<T, N: Unsigned> Default for VariableList<T, N> {
    fn default() -> Self {
        Self {
            vec: Vec::default(),
            _meta: PhantomData,
        }
    }
}

impl<T, N: Unsigned, I: SliceIndex<[T]>> Index<I> for VariableList<T, N> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        Index::index(&self.vec, index)
    }
}

impl<T, N: Unsigned, I: SliceIndex<[T]>> IndexMut<I> for VariableList<T, N> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut self.vec, index)
    }
}

impl<T, N: Unsigned> Deref for VariableList<T, N> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.vec.as_slice()
    }
}

impl<T, N: Unsigned> DerefMut for VariableList<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.vec[..]
    }
}

impl<'a, T, N: Unsigned> IntoIterator for &'a VariableList<T, N> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: ssz::Encode, N: Unsigned> ssz::Encode for VariableList<T, N> {
    fn is_ssz_fixed_len() -> bool {
        <Vec<T>>::is_ssz_fixed_len()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        self.vec.ssz_append(buf)
    }

    fn ssz_fixed_len() -> usize {
        <Vec<T>>::ssz_fixed_len()
    }

    fn ssz_bytes_len(&self) -> usize {
        self.vec.ssz_bytes_len()
    }
}

impl<T: ssz::Decode, N: Unsigned> ssz::Decode for VariableList<T, N> {
    fn is_ssz_fixed_len() -> bool {
        <Vec<T>>::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        <Vec<T>>::ssz_fixed_len()
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        let vec = <Vec<T> as ssz::Decode>::from_ssz_bytes(bytes)?;

        Self::new(vec).map_err(|e|
            ssz::DecodeError::BytesInvalid(format!("VariableList {:?}", e)))
    }
}

impl<T: tree_hash::TreeHash, N: Unsigned> tree_hash::TreeHash for VariableList<T, N> {
    fn tree_hash_type() -> tree_hash::TreeHashType {
        tree_hash::TreeHashType::List
    }

    fn tree_hash_packed_encoding(&self) -> Vec<u8> {
        unreachable!("List should not be packed.")
    }

    fn tree_hash_packing_factor() -> usize {
        unreachable!("List should not be packed.")
    }

    fn tree_hash_root(&self) -> Vec<u8> {
        let root = vec_tree_hash_root::<T, N>(&self.vec);

        tree_hash::mix_in_length(&root, self.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typenum::*;
    use ssz::*;

    #[test]
    fn test_new() {
        let items = vec![1, 2, 3];
        let list_result: Result<VariableList<i32, U3>, _> = VariableList::new(items.clone());
        assert!(list_result.is_ok());
        assert_eq!(list_result.unwrap().vec, items);
    }

    #[test]
    fn test_new_error() {
        let items = vec![1, 2, 3, 4];
        let list_result: Result<VariableList<i32, U3>, _> = VariableList::new(items.clone());
        assert_eq!(list_result, Err(Error::OutOfBounds {
            i: 4,
            len: 3
        }));
    }

    #[test]
    fn test_empty_len() {
        let list: VariableList<i32, U0> = VariableList::empty();
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_from_into() {
        let list: VariableList<i32, U3> = VariableList::from(vec![0, 1, 2, 3]);
        assert_eq!(list.len(), 3);
        assert_eq!(list.vec, vec![0, 1, 2]);

        let list_vec: Vec<i32> = list.into();
        assert_eq!(list_vec, vec![0, 1, 2]);
    }

    #[test]
    fn test_default() {
        let list: VariableList<i32, U0> = VariableList::default();
        assert_eq!(list.len(), 0);
        assert_eq!(list.vec, vec![]);
    }

    #[test]
    fn test_index() {
        let list: VariableList<usize, U4> = VariableList::from(vec![0, 1, 2, 3]);
        for i in 0..4 {
            assert_eq!(list[i], i);
        }
    }

    #[test]
    fn test_index_mut() {
        let mut list: VariableList<usize, U4> = VariableList::from(vec![0, 1, 2, 3]);
        for i in 0..4 {
            list[i] += 2;
            assert_eq!(list[i], i + 2);
        }
    }

    #[test]
    fn test_deref() {
        let list: VariableList<i32, U4> = VariableList::from(vec![0, 1, 2, 3]);
        let slice = [0, 1, 2, 3];
        assert_eq!(*list, slice);
    }

    #[test]
    fn test_into_iter() {
        let vec_from_list: Vec<i32> = <VariableList<i32, U4>>::from(vec![0, 1, 2, 3])
            .into_iter()
            .map(|el| el * el)
            .collect();

        assert_eq!(vec_from_list, vec![0, 1, 4, 9]);
    }

    #[test]
    fn test_ssz_round_trip() {
        let list: VariableList<u16, U4> = VariableList::from(vec![1, 2, 3, 4]);
        let decoded_res = <VariableList<u16, U4>>::from_ssz_bytes(list.as_ssz_bytes().as_slice());
        assert!(decoded_res.is_ok());
        assert_eq!(decoded_res.unwrap(), list)
    }
}