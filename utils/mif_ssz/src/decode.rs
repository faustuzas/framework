pub mod impls;

use super::*;

#[derive(Debug, PartialEq)]
pub enum DecodeError {
    InvalidByteLength { len: usize, expected: usize },
    InvalidLengthPrefix { len: usize, expected: usize },

    /// A length offset pointed to a byte that was out-of-bounds (OOB).
    OutOfBoundsByte { i: usize },
    /// The given bytes were invalid for some application-level reason.
    BytesInvalid(String),
}

/// Trait for object deserialization from SSZ format
pub trait Decode: Sized {
    /// Checks if this object has a fixed sized length.
    ///
    /// If the object is container and has at least one variable-sized length item, it becomes
    /// variable-sized too.
    fn is_ssz_fixed_len() -> bool;

    /// Returns the number of bytes this object's fixed-sized part occupies.
    ///
    /// By default it returns the size of the offset for variable-sized objects.
    /// Fixed-length objects have to return the value of their length.
    fn ssz_fixed_len() -> usize {
        BYTES_PER_LENGTH_OFFSET
    }

    /// Deserializes the object
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError>;
}

#[derive(Copy, Clone, Debug)]
pub struct Offset {
    position: usize,
    offset: usize,
}

/// Splits SSZ bytes into slices.
pub struct SszDecoderBuilder<'a> {
    bytes: &'a [u8],
    items: Vec<&'a [u8]>,
    offsets: Vec<Offset>,
    items_index: usize,
}

impl<'a> SszDecoderBuilder<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            items: vec![],
            offsets: vec![],
            items_index: 0,
        }
    }

    /// Gets next item in byte stream
    pub fn register_type<T: Decode>(&mut self) -> Result<(), DecodeError> {
        if T::is_ssz_fixed_len() {
            let start = self.items_index;
            self.items_index += T::ssz_fixed_len();

            let slice = self.bytes.get(start..self.items_index).ok_or_else(|| {
                DecodeError::InvalidByteLength {
                    len: self.bytes.len(),
                    expected: self.items_index,
                }
            })?;

            self.items.push(slice);
        } else {
            let offset = read_offset(&self.bytes[self.items_index..])?;

            let previous_offset = self
                .offsets
                .last()
                .and_then(|o| Some(o.offset))
                .unwrap_or_else(|| BYTES_PER_LENGTH_OFFSET);

            if (previous_offset > offset) || (offset > self.bytes.len()) {
                return Err(DecodeError::OutOfBoundsByte { i: offset });
            }

            self.offsets.push(Offset {
                position: self.items.len(),
                offset,
            });

            // Push an empty slice into items; it will be replaced later.
            self.items.push(&[]);

            self.items_index += BYTES_PER_LENGTH_OFFSET;
        }

        Ok(())
    }

    fn finalize(&mut self) -> Result<(), DecodeError> {
        if !self.offsets.is_empty() {
            if self.offsets[0].offset != self.items_index {
                return Err(DecodeError::OutOfBoundsByte {
                    i: self.offsets[0].offset,
                });
            }

            for pair in self.offsets.windows(2) {
                let a = pair[0];
                let b = pair[1];

                self.items[a.position] = &self.bytes[a.offset..b.offset];
            }

            if let Some(last) = self.offsets.last() {
                self.items[last.position] = &self.bytes[last.offset..]
            }
        } else {
            if self.items_index != self.bytes.len() {
                return Err(DecodeError::InvalidByteLength {
                    len: self.bytes.len(),
                    expected: self.items_index,
                });
            }
        }

        Ok(())
    }

    pub fn build(mut self) -> Result<SszDecoder<'a>, DecodeError> {
        self.finalize()?;

        Ok(SszDecoder { items: self.items })
    }
}

/// Allows to decode from SSZ format into ordered series of unrelated objects
pub struct SszDecoder<'a> {
    items: Vec<&'a [u8]>,
}

impl<'a> SszDecoder<'a> {
    /// If there are more items than actually exist, runtime error is raised
    pub fn decode_next<T: Decode>(&mut self) -> Result<T, DecodeError> {
        T::from_ssz_bytes(self.items.remove(0))
    }
}

fn read_offset(bytes: &[u8]) -> Result<usize, DecodeError> {
    decode_offset(bytes.get(0..BYTES_PER_LENGTH_OFFSET).ok_or_else(|| {
        DecodeError::InvalidLengthPrefix {
            len: bytes.len(),
            expected: BYTES_PER_LENGTH_OFFSET,
        }
    })?)
}

pub fn read_union_index(bytes: &[u8]) -> Result<usize, DecodeError> {
    read_offset(bytes)
}

fn decode_offset(bytes: &[u8]) -> Result<usize, DecodeError> {
    let len = bytes.len();
    let expected = BYTES_PER_LENGTH_OFFSET;

    if len != expected {
        Err(DecodeError::InvalidLengthPrefix { len, expected })
    } else {
        let mut array: [u8; BYTES_PER_LENGTH_OFFSET] = std::default::Default::default();
        array.clone_from_slice(bytes);

        Ok(u32::from_le_bytes(array) as usize)
    }
}

