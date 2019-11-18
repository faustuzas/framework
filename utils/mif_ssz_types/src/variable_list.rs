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

impl<T, C: Unsigned> VariableList<T, C> {
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
        C::to_usize()
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

impl <T, C:Unsigned> From<Vec<T>> for VariableList<T, C> {
    fn from(mut vec: Vec<T>) -> Self {
        // shrink vector to required size
        vec.truncate(C::to_usize());

        Self {
            vec,
            _meta: PhantomData
        }
    }
}

impl<T, C: Unsigned> Into<Vec<T>> for VariableList<T, C> {
    fn into(self) -> Vec<T> {
        self.vec
    }
}

impl<T, C: Unsigned> Default for VariableList<T, C> {
    fn default() -> Self {
        Self {
            vec: Vec::default(),
            _meta: PhantomData,
        }
    }
}

impl<T, C: Unsigned, I: SliceIndex<[T]>> Index<I> for VariableList<T, C> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        Index::index(&self.vec, index)
    }
}

impl<T, C: Unsigned, I: SliceIndex<[T]>> IndexMut<I> for VariableList<T, C> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut self.vec, index)
    }
}

impl<T, C: Unsigned> Deref for VariableList<T, C> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.vec[..]
    }
}

impl<T, C: Unsigned> DerefMut for VariableList<T, C> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.vec[..]
    }
}

impl<'a, T, C: Unsigned> IntoIterator for &'a VariableList<T, C> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: ssz::Encode, C: Unsigned> ssz::Encode for VariableList<T, C> {
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

impl<T: ssz::Decode, C: Unsigned> ssz::Decode for VariableList<T, C> {
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