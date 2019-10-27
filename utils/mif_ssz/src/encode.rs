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

