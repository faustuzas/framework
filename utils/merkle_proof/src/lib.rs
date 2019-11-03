#[macro_use]
extern crate lazy_static;

use eth2_hashing::hash;
use ethereum_types::H256;

const MAX_TREE_DEPTH: usize = 32;
const EMPTY_SLICE: &[H256] = &[];

lazy_static! {
    static ref ZERO_HASHES: Vec<H256> = {
        let mut hashes = vec![H256::from([0; 32]); MAX_TREE_DEPTH + 1];

        for i in 0..MAX_TREE_DEPTH {
            hashes[i + 1] = hash_concat(hashes[i], hashes[i]);
        }

        hashes
    };

    static ref ZERO_NODES: Vec<MerkleTree> = {
        (0..=MAX_TREE_DEPTH).map(MerkleTree::Zero).collect()
    };
}
