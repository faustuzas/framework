pub use typenum;

mod impls;
mod vendor;

pub use vendor::{BitList, BitVector, Bitfield, FixedVector, VariableList};

pub mod length {
    pub use crate::vendor::{Fixed, Variable};
}
