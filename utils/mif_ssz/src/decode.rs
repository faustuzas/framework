use super::*;

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

