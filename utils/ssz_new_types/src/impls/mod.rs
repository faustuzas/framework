use super::*;
use ssz;
use typenum::Unsigned;

/// This module will implement custom ssz serialization/deserialization
/// for `ssz_types`
mod bitfield;
mod fixed_vector;
mod variable_list;
