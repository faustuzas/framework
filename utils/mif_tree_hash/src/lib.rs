mod merkleize;
use merkleize::*;

mod impls;

#[macro_use]
extern crate lazy_static;

pub const BYTES_PER_CHUNK: usize = 32;
pub const HASH_SIZE: usize = 32;
pub const MERKLE_HASH_CHUNK: usize = 2 * BYTES_PER_CHUNK;

pub fn merkle_root(bytes: &[u8], min_leaves: usize) -> Vec<u8> {
    merkleize(bytes, min_leaves)
}

#[derive(Debug, PartialEq, Clone)]
pub enum TreeHashType {
    Basic,
    Vector,
    List,
    Container,
}

pub trait TreeHash {
    fn tree_hash_type() -> TreeHashType;

    fn tree_hash_packed_encoding(&self) -> Vec<u8>;

    fn tree_hash_packing_factor() -> usize;

    fn tree_hash_root(&self) -> Vec<u8>;
}

pub trait SignedRoot: TreeHash {
    fn signed_root(&self) -> Vec<u8>;
}

pub fn mix_in_length(root: &[u8], length: usize) -> Vec<u8> {
    let mut length_bytes = length.to_le_bytes().to_vec();
    length_bytes.resize(BYTES_PER_CHUNK, 0);

    merkleize::hash_concat(root, &length_bytes)
}

#[macro_export]
macro_rules! tree_hash_ssz_encoding_as_list {
    ($type: ident) => {
        impl tree_hash::TreeHash for $type {
            fn tree_hash_type() -> tree_hash::TreeHashType {
                tree_hash::TreeHashType::List
            }

            fn tree_hash_packed_encoding(&self) -> Vec<u8> {
                unreachable!("List should never be packed.")
            }

            fn tree_hash_packing_factor() -> usize {
                unreachable!("List should never be packed.")
            }

            fn tree_hash_root(&self) -> Vec<u8> {
                ssz::ssz_encode(self).tree_hash_root()
            }
        }
    };
}

#[macro_export]
macro_rules! tree_hash_ssz_encoding_as_vector {
    ($type: ident) => {
        impl tree_hash::TreeHash for $type {
            fn tree_hash_type() -> tree_hash::TreeHashType {
                tree_hash::TreeHashType::Vector
            }

            fn tree_hash_packed_encoding(&self) -> Vec<u8> {
                unreachable!("Vector should never be packed.")
            }

            fn tree_hash_packing_factor() -> usize {
                unreachable!("Vector should never be packed.")
            }

            fn tree_hash_root(&self) -> Vec<u8> {
                tree_hash::merkle_root(&ssz::ssz_encode(self), 0)
            }
        }
    };
}