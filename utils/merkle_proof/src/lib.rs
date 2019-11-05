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
                assert_eq!(leaves.len(), 1);//?
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

    pub fn hash(&self) -> H256 { //pakeist if
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

    pub fn generate_proof(&self, index: usize, depth: usize) -> (H256, Vec<H256>) {
        let mut proof = vec![];
        let mut current_node = self;
        let mut current_depth = depth;
        while current_depth > 0 {
            let ith_bit = (index >> (current_depth - 1)) & 0x01;
            let (left, right) = current_node.left_and_right_branches().unwrap();
            if ith_bit == 1 {
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
    root: H256,
) -> bool {
    if proof.len() == depth {
        calculate_merkle_root(leaf, proof, depth, index) == root
    } else {
        false
    }
}

fn calculate_merkle_root(leaf: H256, proof: &[H256], depth: usize, index: usize) -> H256 { 
    assert_eq!(proof.len(), depth, "proof length should equal depth");

    let mut root = leaf.as_bytes().to_vec();

    for (i, leaf) in proof.iter().enumerate().take(depth) {
        if ((index >> i) & 0x01) == 1 {
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

fn hash_and_concat(h1: H256, h2: H256) -> H256 {
    H256::from_slice(&hash(&concat(
        h1.as_bytes().to_vec(),
        h2.as_bytes().to_vec(),
    )))
}

fn get_next_power_of_two(depth: usize) -> usize {
    2usize.pow(depth as u32)      
}

