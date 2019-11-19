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
                len: 0, expected: 1
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
