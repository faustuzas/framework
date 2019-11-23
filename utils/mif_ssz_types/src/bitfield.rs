use crate::Error;
use core::marker::PhantomData;
use typenum::Unsigned;
use ssz::{Encode, Decode, DecodeError};
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde_hex::{encode as serde_hex_encode, PrefixedHexVisitor};

/// A marker struct used to declare SSZ `Variable` behaviour on a `Bitfield`.
#[derive(Clone, PartialEq, Debug)]
pub struct Variable<N> {
    _meta: PhantomData<N>,
}

/// A marker struct used to declare SSZ `Fixed` behaviour on a `Bitfield`.
#[derive(Clone, PartialEq, Debug)]
pub struct Fixed<N> {
    _meta: PhantomData<N>,
}

/// A marker trait that defines the behaviour of a `Bitfield`.
pub trait BitfieldBehaviour: Clone {}

impl<N: Unsigned + Clone> BitfieldBehaviour for Variable<N> {}
impl<N: Unsigned + Clone> BitfieldBehaviour for Fixed<N> {}

#[derive(Clone, PartialEq, Debug)]
pub struct Bitfield<C> {
    bytes: Vec<u8>,
    len: usize,
    _meta: PhantomData<C>
}

impl<N: Unsigned + Clone> Bitfield<Variable<N>> {
    pub fn with_capacity(bits_len: usize) -> Result<Self, Error> {
        if bits_len <= N::to_usize() {
            Ok(Self {
                bytes: vec![0; bytes_required(bits_len)],
                len: bits_len,
                _meta: PhantomData
            })
        } else {
            Err(Error::OutOfBounds { i: bits_len, len: Self::max_len() })
        }
    }

    pub fn max_len() -> usize {
        N::to_usize()
    }

    /// Encodes itself to SSZ encoding with leading zero set to true
    /// to indicate the length of the bitfield
    pub fn into_bytes(self) -> Vec<u8> {
        let bits_len = self.len();
        let mut bytes = self.bytes;

        bytes.resize(bytes_required(bits_len + 1), 0);

        let mut bitfield: Bitfield<Variable<N>> =
            Bitfield::from_raw_bytes(bytes, bits_len + 1)
                .unwrap_or_else(|_| unreachable!(
                    "Bitfield with {} bytes must have enough capacity for {} bits",
                    bytes_required(bits_len + 1), bits_len + 1)
                );

        // set the marker bit for the end of the list
        bitfield.set(bits_len, true)
            .expect("bits_len must fall in bounds of the bitfield");

        bitfield.bytes
    }

    /// Decodes SSZ encoded bytes with leading zero set to true
    /// to indicate the length of the bitfield
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        let bytes_len = bytes.len();
        let mut bitfield: Bitfield<Variable<N>> =
            Bitfield::from_raw_bytes(bytes, bytes_len * 8)?;

        // the length of the bitfield is determined by last 1 bit
        let bits_len = bitfield.highest_set_bit()
            .ok_or(Error::MissingLengthInformation)?;

        if bits_len / 8 + 1 != bytes_len {
            return Err(Error::InvalidByteCount {
                given: bytes_len,
                expected: bits_len / 8 + 1
            });
        }

        if bits_len <= Self::max_len() {
            bitfield.set(bits_len, false)
                .expect("Length bit has been found");

            let mut bytes = bitfield.into_raw_bytes();
            bytes.truncate(bytes_required(bits_len));

            Self::from_raw_bytes(bytes, bits_len)
        } else {
            Err(Error::OutOfBounds {
                i: bits_len,
                len: Self::max_len()
            })
        }
    }

    pub fn intersection(&self, other: &Self) -> Self {
        let min_bits_len = std::cmp::min(self.len(), other.len());
        let mut result = Self::with_capacity(min_bits_len)
            .expect("Min length always l");

        for i in 0..min_bits_len {
            result.bytes[i] = self.bytes[i] & other.bytes[i];
        }

        result
    }

    pub fn union(&self, other: &Self) -> Self {
        let max_bits_len = std::cmp::max(self.bytes.len(), other.bytes.len());
        let mut result = Self::with_capacity(max_bits_len)
            .expect("Max length will always be less than N");

        // because on of them can be longer
        // we need to make sure we have a fallback if an index is too high
        for i in 0..max_bits_len {
            result.bytes[i] = self.bytes.get(i).copied().unwrap_or(0)
                | other.bytes.get(i).copied().unwrap_or(0);
        }

        result
    }
}

impl<N: Unsigned + Clone> Bitfield<Fixed<N>> {
    pub fn new() -> Self {
        Self {
            bytes: vec![0; bytes_required(Self::capacity())],
            len: Self::capacity(),
            _meta: PhantomData
        }
    }

    pub fn capacity() -> usize {
        N::to_usize()
    }

    /// Bitlist with fixed length do not need to set length bit so
    /// we can just return raw bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.into_raw_bytes()
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        Self::from_raw_bytes(bytes, Self::capacity())
    }
}

impl<N: Unsigned + Clone> Default for Bitfield<Fixed<N>> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: BitfieldBehaviour> Bitfield<T> {

    pub fn set(&mut self, i: usize, value: bool) -> Result<(), Error> {
        let bits_len = self.len();

        if i < bits_len {
            let byte = self.bytes
                .get_mut(i / 8)
                .ok_or(Error::OutOfBounds { i, len: bits_len })?;

            if value {
                *byte |= get_true_bit_at(i)
            } else {
                *byte &= get_false_bit_at(i)
            }

            Ok(())
        } else {
            Err(Error::OutOfBounds { i, len: bits_len })
        }
    }

    pub fn get(&self, i: usize) -> Result<bool, Error> {
        let bits_len = self.len();

        if i < bits_len {
            let byte = self.bytes.get(i / 8)
                .ok_or(Error::OutOfBounds { i, len: bits_len})?;

            Ok(*byte & get_true_bit_at(i) > 0)
        } else {
            Err(Error::OutOfBounds { i, len: bits_len })
        }
    }

    /// Returns the number of bits stored in `self`.
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the underlying bytes representation
    pub fn into_raw_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Returns a view into the underlying bytes representation
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    pub fn from_raw_bytes(bytes: Vec<u8>, bits_len: usize) -> Result<Self, Error> {
        if bits_len == 0 {
            if bytes.len() == 1 && bytes == [0] {
                Ok(Self {
                    bytes,
                    len: 0,
                    _meta: PhantomData
                })
            } else {
                Err(Error::ExcessBits)
            }
        } else if bytes.len() != bytes_required(bits_len) {
            Err(Error::InvalidByteCount {
                given: bytes.len(),
                expected: bytes_required(bits_len),
            })
        } else {
            // Ensure there are no bits higher than `bits_len` that are set to true.
            let mask = !(u8::max_value() << 8 - (bits_len % 8) as u8);

            if (bytes.last().expect("Guarded against empty bytes") & mask) == 0 {
                Ok(Self {
                    bytes,
                    len: bits_len,
                    _meta: PhantomData
                })
            } else {
                Err(Error::ExcessBits)
            }
        }
    }

    pub fn highest_set_bit(&self) -> Option<usize> {
        self.bytes.iter()
            .enumerate()
            .rev()
            .find(|(i, byte)| **byte > 0)
            .map(|(i, byte)| i * 8 + 7 - byte.leading_zeros() as usize)
    }

    pub fn iter(&self) -> BitIter<'_, T> {
        BitIter {
            bitfield: self,
            i: 0
        }
    }

    pub fn is_zero(&self) -> bool {
        self.bytes.iter().all(|b| *b == 0)
    }

    pub fn num_set_bits(&self) -> usize {
        self.bytes
            .iter()
            .map(|byte| byte.count_ones() as usize)
            .sum()
    }

    pub fn difference(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.difference_inplace(other);
        result
    }

    pub fn difference_inplace(&mut self, other: &Self) {
        let min_bytes_len = std::cmp::min(self.bytes.len(), other.bytes.len());

        for i in 0..min_bytes_len {
            self.bytes[i] &= !other.bytes[i];
        }
    }

    pub fn shift_up(&mut self, n: usize) -> Result<(), Error> {
        let bits_len = self.len();

        if n <= bits_len {
            for i in (n..bits_len).rev() {
                self.set(i, self.get(i - n)?)?;
            }

            for i in 0..n {
                self.set(i, false).unwrap();
            }

            Ok(())
        } else {
            Err(Error::OutOfBounds {
                i: n,
                len: bits_len
            })
        }
    }
}

/// An iterator over the bits in a `Bitfield`.
pub struct BitIter<'a, T> {
    bitfield: &'a Bitfield<T>,
    i: usize,
}

impl<'a, T: BitfieldBehaviour> Iterator for BitIter<'a, T> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        let bit_value = self.bitfield.get(self.i).ok()?;
        self.i += 1;

        Some(bit_value)
    }
}

impl<N: Unsigned + Clone> Encode for Bitfield<Variable<N>> {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.append(&mut self.clone().into_bytes())
    }

    fn ssz_bytes_len(&self) -> usize {
        bytes_required(self.len() + 1)
    }
}

impl<N: Unsigned + Clone> Decode for Bitfield<Variable<N>> {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(bytes.to_vec()).map_err(|e|
            DecodeError::BytesInvalid(format!("Error occurred while decoding BitList: {:?}", e)))
    }
}

impl<N: Unsigned + Clone> Encode for Bitfield<Fixed<N>> {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.append(&mut self.clone().into_bytes())
    }

    fn ssz_fixed_len() -> usize {
        bytes_required(N::to_usize())
    }

    fn ssz_bytes_len(&self) -> usize {
        <Self as Encode>::ssz_fixed_len()
    }
}

impl<N: Unsigned + Clone> Decode for Bitfield<Fixed<N>> {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        bytes_required(N::to_usize())
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(bytes.to_vec()).map_err(|e|
            DecodeError::BytesInvalid(format!("Error occurred while decoding BitVector: {:?}", e)))
    }
}

macro_rules! serde_bitfield_impls {
    ($type: ident) => {
        impl <N: Unsigned + Clone> Serialize for Bitfield<$type<N>> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&serde_hex_encode(self.as_ssz_bytes()))
            }
        }

        impl <'a, N: Unsigned + Clone> Deserialize<'a> for Bitfield<$type<N>> {
            fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error>{
                let deserialized_bytes = deserializer.deserialize_str(PrefixedHexVisitor)?;
                Self::from_ssz_bytes(&deserialized_bytes)
                    .map_err(|e| serde::de::Error::custom(format!("Unable to deserialize Bitfield: {:?}", e)))
            }
        }
    };
}

serde_bitfield_impls!(Variable);
serde_bitfield_impls!(Fixed);

/// Example:
/// ```
/// assert_eq!(get_true_bit_at(3), 0b0000_1000)
///```
fn get_true_bit_at(pos: usize) -> u8 {
    1 << (pos % 8) as u8
}

fn get_false_bit_at(pos: usize) -> u8 {
    !get_true_bit_at(pos)
}

fn bytes_required(bits_len: usize) -> usize {
    std::cmp::max(1, (bits_len + 7) / 8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_true_bit_at() {
        assert_eq!(get_true_bit_at(3), 0b0000_1000)
    }

    #[test]
    fn test_get_false_bit_at() {
        assert_eq!(get_false_bit_at(5), 0b1101_1111)
    }
}