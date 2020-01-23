pub use typenum;

mod impls;
mod vendor;

pub use vendor::{BitList, BitVector, Bitfield, Error, FixedVector, VariableList};

pub mod length {
    pub use crate::vendor::{Fixed, Variable};
}

pub mod serde_hex {
    pub use crate::vendor::{encode, HexVisitor, PrefixedHexVisitor};
}
