pub use typenum;

mod vendor;

pub use vendor::{
  FixedVector,
  VariableList,
  BitList, BitVector, Bitfield
};

pub mod length {
    pub use crate::vendor::{Fixed, Variable};
}

