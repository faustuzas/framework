mod impls;

use super::*;

/// Trait for object serialization into SSZ format
pub trait Encode {

    /// Checks if this object has a fixed sized length.
    ///
    /// If the object is container and has at least one variable-sized length item, it becomes
    /// variable-sized too.
    fn is_ssz_fixed_len() -> bool;

    /// Appends serialized `self` to the provided buffer.
    ///
    /// Variable-sized object has to append only their data portion, not the offset.
    fn ssz_append(&self, buf: &mut Vec<u8>);

    /// Returns the number of bytes this object's fixed-sized part occupies.
    ///
    /// By default it returns the size of the offset for variable-sized objects.
    /// Fixed-length objects have to return the value of their length.
    fn ssz_fixed_len() -> usize {
        BYTES_PER_LENGTH_OFFSET
    }

    /// Returns the total size when `self` is serialized
    fn ssz_bytes_len(&self) -> usize;

    /// Serializes the object
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];

        self.ssz_append(&mut buf);

        buf
    }
}

/// Allows to encode ordered series of unrelated objects into SSZ format
pub struct SszEncoder<'a> {
    /// size of the fixed-length variables portion
    offset: usize,

    /// serialized bytes buffer
    buf: &'a mut Vec<u8>,

    /// serialized variable-size objects
    variable_bytes: Vec<u8>
}

impl<'a> SszEncoder<'a> {
    /// Identical to `Self::container`
    pub fn list(buf: &'a mut Vec<u8>, num_fixed_bytes: usize) -> Self {
        Self::container(buf, num_fixed_bytes)
    }

    /// Creates an encoder for a SSZ container
    pub fn container(buf: &'a mut Vec<u8>, num_fixed_bytes: usize) -> Self {
        // allocate space for the fixed-length items
        buf.reserve(num_fixed_bytes);

        Self {
            offset: num_fixed_bytes,
            buf,
            variable_bytes: vec![]
        }
    }

    /// Append a serialized item to the buffer
    pub fn append<T: Encode>(&mut self, item: &T) {
        // if item is fixed-size, simply append its contents to fixed-size part
        if T::is_ssz_fixed_len() {
            item.ssz_append(&mut self.buf);
        } else {
            // add offset into fixed size part
            let total_offset = self.offset + self.variable_bytes.len();
            self.buf.append(&mut encode_length(total_offset));

            // append serialized data to variable-size part
            item.ssz_append(&mut self.variable_bytes);
        }
    }

    /// Append the variable bytes to main buffer and return encoded data
    ///
    /// This method has to be called after all `append` operations.
    ///
    /// Encoder becomes unusable after this call.
    pub fn finalize(&mut self) -> &mut Vec<u8> {
        self.buf.append(&mut self.variable_bytes);

        &mut self.buf
    }
}

pub fn encode_length(len: usize) -> Vec<u8> {
    // if length is larger than max allow, raise debug assert
    debug_assert!(len <= MAX_LENGTH_VALUE);

    len.to_le_bytes()[0..BYTES_PER_LENGTH_OFFSET].to_vec()
}

pub fn encode_union_index(index: usize) -> Vec<u8> {
    encode_length(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_length() {
        assert_eq!(encode_length(0), vec![0, 0, 0, 0]);

        assert_eq!(encode_length(1), vec![1, 0, 0, 0]);

        assert_eq!(encode_length(400), vec![144, 1, 0, 0]);

        assert_eq!(
            encode_length(MAX_LENGTH_VALUE),
            vec![255; BYTES_PER_LENGTH_OFFSET]
        );
    }

    #[test]
    fn test_encode_union_index() {
        assert_eq!(encode_union_index(0), vec![0, 0, 0, 0]);

        assert_eq!(encode_union_index(1), vec![1, 0, 0, 0]);

        assert_eq!(encode_union_index(400), vec![144, 1, 0, 0]);

        assert_eq!(
            encode_union_index(MAX_LENGTH_VALUE),
            vec![255; BYTES_PER_LENGTH_OFFSET]
        );
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn test_encode_length_above_max_debug_panics() {
        encode_length(MAX_LENGTH_VALUE + 1);
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn test_encode_length_above_max_not_debug_does_not_panic() {
        assert_eq!(encode_length(MAX_LENGTH_VALUE + 1), vec![0; 4]);
    }
}