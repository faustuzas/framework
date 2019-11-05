#[macro_use]
extern crate lazy_static;

use eth2_hashing::hash;
use ethereum_types::H256;
// pub struct H256(pub [u8; 32]);
const MAX_TREE_DEPTH: usize = 32;
const EMPTY_SLICE: &[H256] = &[];

lazy_static! {
    static ref ZERO_HASHES: Vec<H256> = {
        let mut hashes = vec![H256::from([0; 32]); MAX_TREE_DEPTH + 1];

        for i in 0..MAX_TREE_DEPTH {
            hashes[i + 1] = hash_and_concat(hashes[i], hashes[i]);
        }

        hashes
    };

    static ref ZERO_NODES: Vec<MerkleTree> = {
        (0..=MAX_TREE_DEPTH).map(MerkleTree::Zero).collect()
    };
}

#[derive(Debug)]
pub enum MerkleTree {
    Leaf(H256),
    Node(H256, Box<Self>, Box<Self>),
    Zero(usize),
}

impl MerkleTree {
    pub fn create(leaves: &[H256], depth: usize) -> Self {
        use MerkleTree::*;
        if leaves.is_empty() {
             return Zero(depth); 
        }

        if depth == 0 {
                assert_eq!(leaves.len(), 1);
                Leaf(leaves[0])
        } else {
                let capacity = get_next_power_of_two(depth-1);
                let (l_leaves, r_leaves) = if leaves.len() <= capacity { (leaves, EMPTY_SLICE) } else { leaves.split_at(capacity)};
                let l_subtree = MerkleTree::create(l_leaves, depth - 1);
                let r_subtree = MerkleTree::create(r_leaves, depth - 1);
                let hash = hash_and_concat(l_subtree.hash(), r_subtree.hash());
                Node(hash, Box::new(l_subtree), Box::new(r_subtree))
        }
    }

    pub fn hash(&self) -> H256 { 
        match *self {
            MerkleTree::Leaf(h) => h,
            MerkleTree::Node(h, _, _) => h,
            MerkleTree::Zero(depth) => ZERO_HASHES[depth],
        }
    }

    pub fn left_and_right_branches(&self) -> Option<(&Self, &Self)> {
        match *self {
            MerkleTree::Leaf(_) | MerkleTree::Zero(0) => None,
            MerkleTree::Node(_, ref l, ref r) => Some((l, r)),
            MerkleTree::Zero(depth) => Some((&ZERO_NODES[depth - 1], &ZERO_NODES[depth - 1])),
        }
    }

    pub fn is_leaf(&self) -> bool {
        match self {
            MerkleTree::Leaf(_) => true,
            _ => false,
        }
    }

    pub fn make_proof(&self, index: usize, depth: usize) -> (H256, Vec<H256>) { // no tests made 
        let mut proof = vec![];
        let mut current_node = self;
        let mut current_depth = depth;
        while current_depth > 0 {
            let (left, right) = current_node.left_and_right_branches().unwrap();
            if get_generalized_index_bit(index, current_depth-1) {
                proof.push(left.hash());
                current_node = right;
            } else {
                proof.push(right.hash());
                current_node = left;
            }
            current_depth -= 1;
        }

        debug_assert_eq!(proof.len(), depth);
        debug_assert!(current_node.is_leaf());

        proof.reverse();

        (current_node.hash(), proof)
    }
}

pub fn verify_merkle_proof( 
    leaf: H256,
    proof: &[H256],
    depth: usize,
    index: usize,
    root: H256,) -> bool {

    if proof.len() == depth {
        calculate_merkle_root(leaf, proof, depth, index) == root
    } else {
        false
    }
}

fn calculate_merkle_root(
    leaf: H256,
     proof: &[H256],
      depth: usize,
       index: usize,) -> H256 { 

    assert_eq!(proof.len(), depth, "proof length should equal depth");

    let mut root = leaf.as_bytes().to_vec();

    for (i, leaf) in proof.iter().enumerate().take(depth) {
        // if ((index >> i) & 0x01) == 1 {
        if get_generalized_index_bit(index, i) {    
            let input = concat(leaf.as_bytes().to_vec(), root);
            root = hash(&input);
        } else {
            let mut input = root;
            input.extend_from_slice(leaf.as_bytes());
            root = hash(&input);
        }
    }

    H256::from_slice(&root)
}

fn concat(mut vec1: Vec<u8>, mut vec2: Vec<u8>) -> Vec<u8> {
    vec1.append(&mut vec2);
    return vec1;
}

fn get_generalized_index_bit(index: usize, i: usize) -> bool {
    ((index >> i) & 0x01) == 1
}


fn hash_and_concat(h1: H256, h2: H256) -> H256 {
    H256::from_slice(&hash(&concat(
        h1.as_bytes().to_vec(),
        h2.as_bytes().to_vec(),
    )))
}

fn get_next_power_of_two(depth: usize) -> usize {
    2usize.pow(depth as u32)      
}

//paklausti ar reik:
// kokiu funkciju man reik?-nera dokumentacijoj;
// verify_merkle_multiproof? yra dokumentacijoj

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_zero_correct() {
        let depth = 2;
        let zero = H256::from([0x00; 32]);
        let dense_tree = MerkleTree::create(&[zero, zero, zero, zero], depth);
        let sparse_tree = MerkleTree::create(&[], depth);
        assert_eq!(dense_tree.hash(), sparse_tree.hash());
    }

    #[test]
    fn create_small_example() {
        // Construct a small merkle tree manually and check that it's consistent with the MerkleTree type.
        let leaf_b00 = H256::from([0xAA; 32]);
        let leaf_b01 = H256::from([0xBB; 32]);
        let leaf_b10 = H256::from([0xCC; 32]);
        let leaf_b11 = H256::from([0xDD; 32]);

        let node_b0x = hash_and_concat(leaf_b00, leaf_b01);
        let node_b1x = hash_and_concat(leaf_b10, leaf_b11);

        let root = hash_and_concat(node_b0x, node_b1x);

        let tree = MerkleTree::create(&[leaf_b00, leaf_b01, leaf_b10, leaf_b11], 2);
        assert_eq!(tree.hash(), root);
    }
 #[test]
    fn verify_small_example() {
        // Construct a small merkle tree manually
        let leaf_b00 = H256::from([0xAA; 32]);
        let leaf_b01 = H256::from([0xBB; 32]);
        let leaf_b10 = H256::from([0xCC; 32]);
        let leaf_b11 = H256::from([0xDD; 32]);

        let node_b0x = hash_and_concat(leaf_b00, leaf_b01);
        let node_b1x = hash_and_concat(leaf_b10, leaf_b11);

        let root = hash_and_concat(node_b0x, node_b1x);

        // Run some proofs
        assert!(verify_merkle_proof(
            leaf_b00,
            &[leaf_b01, node_b1x],
            2,
            0b00,
            root
        ));
        assert!(verify_merkle_proof(
            leaf_b01,
            &[leaf_b00, node_b1x],
            2,
            0b01,
            root
        ));
        assert!(verify_merkle_proof(
            leaf_b10,
            &[leaf_b11, node_b0x],
            2,
            0b10,
            root
        ));
        assert!(verify_merkle_proof(
            leaf_b11,
            &[leaf_b10, node_b0x],
            2,
            0b11,
            root
        ));
        assert!(verify_merkle_proof(
            leaf_b11,
            &[leaf_b10],
            1,
            0b11,
            node_b1x
        ));
// tests that should fail
        assert!(!verify_merkle_proof(leaf_b01, &[], 2, 0b01, root));

        assert!(!verify_merkle_proof(
            leaf_b01,
            &[node_b1x, leaf_b00],
            2,
            0b01,
            root
        ));

        assert!(!verify_merkle_proof(leaf_b01, &[leaf_b00], 2, 0b01, root));

        assert!(!verify_merkle_proof(
            leaf_b01,
            &[leaf_b00, node_b1x],
            2,
            0b10,
            root
        ));

        assert!(!verify_merkle_proof(
            leaf_b01,
            &[leaf_b00, node_b1x],
            2,
            0b01,
            node_b1x
        ));
    }

    #[test]
    fn verify_zero_depth() {
        let leaf = H256::from([0xD6; 32]);
        let junk = H256::from([0xD7; 32]);
        assert!(verify_merkle_proof(leaf, &[], 0, 0, leaf));
        assert!(!verify_merkle_proof(leaf, &[], 0, 7, junk));
    }
}