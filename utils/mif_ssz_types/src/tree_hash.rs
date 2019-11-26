use tree_hash::{merkle_root, TreeHash, TreeHashType, BYTES_PER_CHUNK};
use typenum::Unsigned;

pub fn vec_tree_hash_root<T: TreeHash, N: Unsigned>(vec: &[T]) -> Vec<u8> {
    let (leaves, minimum_chunks) = match T::tree_hash_type() {
        TreeHashType::Basic => {
            let mut leaves =
                Vec::with_capacity((BYTES_PER_CHUNK / T::tree_hash_packing_factor()) * vec.len());

            for el in vec {
                leaves.append(&mut el.tree_hash_packed_encoding());
            }

            let values_per_chunk = T::tree_hash_packing_factor();
            let minimum_chunks = (N::to_usize() + values_per_chunk - 1) / values_per_chunk;

            (leaves, minimum_chunks)
        },
        _ => {
            let mut leaves = Vec::with_capacity(vec.len() * BYTES_PER_CHUNK);

            for el in vec {
                leaves.append(&mut el.tree_hash_root())
            }

            let minimum_chunks = N::to_usize();

            (leaves, minimum_chunks)
        }
    };

    merkle_root(&leaves, minimum_leaves)
}

pub fn bitfield_bytes_tree_hash_root<N: Unsigned>(bytes: &[u8]) -> Vec<u8> {
    let byte_size = (N::to_usize() + 7) / 8;
    let minimum_chunk_count = (byte_size + BYTES_PER_CHUNK - 1) / BYTES_PER_CHUNK;

    merkle_root(bytes, minimum_chunk_count)
}
