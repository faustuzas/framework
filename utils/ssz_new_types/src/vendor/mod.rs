/// These data structures' code is taken from Lighthouse implementation
mod tree_hash;

mod variable_list;
pub use variable_list::VariableList;

mod fixed_vector;
pub use fixed_vector::FixedVector;

mod bitfield;
pub use bitfield::{BitList, BitVector, Bitfield};
pub use bitfield::{Fixed, Variable};

mod error;
pub use error::Error;
