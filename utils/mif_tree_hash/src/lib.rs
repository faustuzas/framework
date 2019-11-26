mod merkleize;
use merkleize::*;

pub const BYTES_PER_CHUNK: usize = 32;
pub const HASH_SIZE: usize = 32;
pub const MERKLE_HASH_CHUNK: usize = 2 * BYTES_PER_CHUNK;

pub fn merkle_root(bytes: &[u8]) -> Vec<u8> {
    merkleize(bytes)
}