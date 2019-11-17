pub mod impls;

use super::*;

#[derive(Debug, PartialEq)]
pub enum DecodeError {
    /// The bytes supplied were too short to be decoded into the specified type.
    InvalidByteLength { len: usize, expected: usize },

    /// The given bytes were too short to be read as a length prefix.
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

    /// De-serializes the bytes into `Self`.
    /// Bytes has to be exact length required to decode `Self`.
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError>;
}

#[derive(Copy, Clone, Debug)]
pub struct Offset {
    position: usize,
    offset: usize,
}

/// Builds `SszDecoder` by splitting SSZ bytes into slices which can be decoded into object
/// instances by built decoder.
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

    /// Declare that the next instance will be of type `T`
    pub fn register_type<T: Decode>(&mut self) -> Result<(), DecodeError> {
        if T::is_ssz_fixed_len() {
            let start_index = self.items_index;
            self.items_index += T::ssz_fixed_len();

            let slice = self.bytes.get(start_index..self.items_index).ok_or_else(|| {
                DecodeError::InvalidByteLength {
                    len: self.bytes.len(),
                    expected: self.items_index,
                }
            })?;

            self.items.push(slice);
        } else {
            let current_offset = next_offset(&self.bytes[self.items_index..])?;

            let previous_offset = self
                .offsets
                .last()
                .and_then(|o| Some(o.offset))
                .unwrap_or_else(|| BYTES_PER_LENGTH_OFFSET);

            // check if current offset does not exceed total bytes length
            if (previous_offset > current_offset) || (current_offset > self.bytes.len()) {
                return Err(DecodeError::OutOfBoundsByte { i: current_offset });
            }

            self.offsets.push(Offset {
                position: self.items.len(),
                offset: current_offset,
            });

            // Put a placeholder value into items because we need to parse all bytes first to know
            // all positions of fixed-sized items to be able to parse variable-sized ones.
            self.items.push(&[]);

            self.items_index += BYTES_PER_LENGTH_OFFSET;
        }

        Ok(())
    }

    pub fn build(mut self) -> Result<SszDecoder<'a>, DecodeError> {
        if !self.offsets.is_empty() {
            // First offset has to point to the byte following the fixed-length bytes
            if self.offsets[0].offset != self.items_index {
                return Err(DecodeError::OutOfBoundsByte {
                    i: self.offsets[0].offset,
                });
            }

            // Take two subsequent offsets and copy the data between them
            for limits in self.offsets.windows(2) {
                let start = limits[0];
                let end = limits[1];

                let bytes = self.bytes.get(start.offset..end.offset)
                    .ok_or_else(|| DecodeError::OutOfBoundsByte { i: start.offset })?;
                self.items[start.position] = bytes;
            }

            // Copy data from the last offset to the end
            if let Some(last_offset) = self.offsets.last() {
                let bytes = self.bytes.get(last_offset.offset..)
                    .ok_or_else(|| DecodeError::OutOfBoundsByte { i: last_offset.offset })?;

                self.items[last_offset.position] = bytes;
            }
        }

        Ok(SszDecoder { items: self.items })
    }
}

/// Allows to decode from SSZ format into ordered series of unrelated objects
pub struct SszDecoder<'a> {
    items: Vec<&'a [u8]>,
}

impl<'a> SszDecoder<'a> {
    pub fn decode_next<T: Decode>(&mut self) -> Result<T, DecodeError> {
        T::from_ssz_bytes(self.items.remove(0))
    }
}

pub fn read_union_index(bytes: &[u8]) -> Result<usize, DecodeError> {
    next_offset(bytes)
}

fn next_offset(bytes: &[u8]) -> Result<usize, DecodeError> {
    let offset_bytes = bytes.get(0..BYTES_PER_LENGTH_OFFSET)
        .ok_or_else(|| {
            DecodeError::InvalidLengthPrefix {
                len: bytes.len(),
                expected: BYTES_PER_LENGTH_OFFSET
            }
        })?;

    let mut holder_array: [u8; BYTES_PER_LENGTH_OFFSET] = [0; BYTES_PER_LENGTH_OFFSET];
    holder_array.clone_from_slice(offset_bytes);

    Ok(u32::from_le_bytes(holder_array) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_offset() {
        let bytes = [0b1111_0110, 0b1101_0010, 0, 0];

        assert_eq!(next_offset(&bytes), Ok(54006));
    }

    #[test]
    fn test_too_little_bytes_for_offset() {
        let bytes = [1, 2, 3];

        assert_eq!(next_offset(&bytes), Err(DecodeError::InvalidLengthPrefix {
            expected: BYTES_PER_LENGTH_OFFSET,
            len: 3
        }))
    }
}

