mod tree_hash;
mod variable_list;
mod fixed_vector;
mod bitfield;

use bitfield::{Bitfield, Variable, Fixed};

/// Exported types
pub type BitList<N> = Bitfield<Variable<N>>;
pub type BitVector<N> = Bitfield<Fixed<N>>;
pub use variable_list::VariableList;
pub use fixed_vector::FixedVector;

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
