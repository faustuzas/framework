use super::*;
use eth2_hashing::hash;

pub const MAX_TREE_DEPTH: usize = 48;

lazy_static! {
    static ref ZERO_HASHES: Vec<Vec<u8>> = {
        let mut hashes = vec![vec![0; 32]; MAX_TREE_DEPTH + 1];

        for i in 0..MAX_TREE_DEPTH {
            hashes[i + 1] = hash_concat(&hashes[i], &hashes[i]);
        }

        hashes
    };
}

pub fn merkleize(bytes: &[u8]) -> Vec<u8>{
    // if bytes does not exceed the length of bytes per chunk, it does not need merkleization
    if bytes.len() <= BYTES_PER_CHUNK {
        let mut root = bytes.to_vec();

        // pad value with zeroes
        root.resize(BYTES_PER_CHUNK, 0);

        return root;
    }

    // Number of leaves with the value
    let leaves_with_value_count = (bytes.len() + BYTES_PER_CHUNK - 1) / BYTES_PER_CHUNK;

    // Number of parents the leaves with value will have
    let parents_with_value_count = std::cmp::max(1, next_even(leaves_with_value_count));

    // Number of leaves including padding ones
    let total_leaves_count = leaves_with_value_count.next_power_of_two();

    // Buffer to hold created chunks
    let mut chunks = ChunksHolder::for_chunks(parents_with_value_count);

    // The height of the tree
    let height = total_leaves_count.trailing_zeros() as usize + 1;

    // Fill chunks object with first level chunks made from leaves
    for i in 0..parents_with_value_count {
        let start_offset = i * BYTES_PER_CHUNK * 2;

        // take two chunks sized bytes from value provided
        let hash = match bytes.get(start_offset..start_offset + BYTES_PER_CHUNK * 2) {
            Some(bytes_slice) => hash(bytes_slice),
            None => {
                let mut leftover_bytes = bytes
                    .get(start_offset..)
                    .expect("Leftover cannot be zero length")
                    .to_vec();

                // pad to required length
                leftover_bytes.resize(BYTES_PER_CHUNK * 2, 0);

                hash(&leftover_bytes)
            }
        };

        // store hashed two leaves
        chunks.set(i, &hash)
            .expect("Buffer has allocated enough space");
    }

    for height in 1..height - 1 {
        let child_nodes_count = chunks.chunks_stored();
        let parent_nodes_count = next_even(child_nodes_count);

        for parent_index in 0..parent_nodes_count {
            let left_child = match chunks.get(parent_index * 2) {
                Ok(child) => child,
                _ => panic!("Parent have to have left child")
            };

            let right_child = match chunks.get(parent_index * 2 + 1) {
                Ok(child) => child,
                _ => zero_hash_for_height(height)
            };

            let parent_hash = hash_concat(left_child, right_child);

            // Store a parent hash
            chunks.set(parent_index, &parent_hash)
                .expect("Buffer has allocated enough space");
        }

        // The size will shrink by the factor of two every time
        chunks.truncate(parent_nodes_count);
    }

    let root_hash = chunks.into_vec();

    if root_hash.len() == BYTES_PER_CHUNK {
        panic!("Merkle root hash calculated incorrectly")
    }

    root_hash
}

struct ChunksHolder {
    bytes: Vec<u8>
}

impl ChunksHolder {
    fn for_chunks(chunks_count: usize) -> Self {
        Self {
            bytes: vec![0; chunks_count * BYTES_PER_CHUNK]
        }
    }

    fn set(&mut self, chunk: usize, value: &[u8]) -> Result<(), ()> {
        if chunk < self.chunks_stored() && value.len() == BYTES_PER_CHUNK {
            let slice = &mut self.bytes[chunk_left_offset(chunk)..chunk_right_offset(chunk)];
            slice.copy_from_slice(value);
            Ok(())
        } else {
            Err(())
        }
    }

    fn get(&self, chunk: usize) -> Result<&[u8], ()> {
        if chunk < self.chunks_stored() {
            Ok(&self.bytes[chunk_left_offset(chunk)..chunk_right_offset(chunk)])
        } else {
            Err(())
        }
    }

    fn truncate(&mut self, to_chunks: usize) {
        self.bytes.truncate(to_chunks * BYTES_PER_CHUNK)
    }

    fn into_vec(self) -> Vec<u8> {
        self.bytes
    }

    fn chunks_stored(&self) -> usize {
        self.bytes.len() / BYTES_PER_CHUNK
    }
}

fn zero_hash_for_height(height: usize) -> &'static [u8] {
    if height <= MAX_TREE_DEPTH {
        &ZERO_HASHES[height]
    } else {
        panic!("Tree exceeds MAX_TREE_DEPTH of {}", MAX_TREE_DEPTH)
    }
}

fn chunk_left_offset(chunk: usize) -> usize {
    chunk * BYTES_PER_CHUNK
}

fn chunk_right_offset(chunk: usize) -> usize {
    chunk * BYTES_PER_CHUNK + BYTES_PER_CHUNK
}

fn next_even(n: usize) -> usize {
    n + n % 2
}

pub fn hash_concat(h1: &[u8], h2: &[u8]) -> Vec<u8> {
    hash(&concat_vecs(h1.to_vec(), h2.to_vec()))
}

fn concat_vecs(mut vec1: Vec<u8>, mut vec2: Vec<u8>) -> Vec<u8> {
    vec1.append(&mut vec2);
    vec1
}