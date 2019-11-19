mod variable_list;
mod fixed_vector;
mod bitfield;

/// Returned when an item encounters an error.
#[derive(PartialEq, Debug)]
pub enum Error {
    OutOfBounds {
        i: usize,
        len: usize,
    },
    /// A `BitList` does not have a set bit, therefore it's length is unknowable.
    MissingLengthInformation,
    /// A `BitList` has excess bits set to true.
    ExcessBits,
    /// A `BitList` has an invalid number of bytes for a given bit length.
    InvalidByteCount {
        given: usize,
        expected: usize,
    }
}
