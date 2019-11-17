#[macro_use]
extern crate lazy_static;
macro_rules! log_of {
    ($val:expr, $base:expr, $type:ty) => {
         ($val as f32).log($base) as $type
    }
}
use std::collections::HashMap;
use math::round;
use eth2_hashing::hash;
use ethereum_types::H256;
// pub struct H256(pub [u8; 32]);
const MAX_TREE_DEPTH: usize = 32;
const EMPTY_SLICE: &[H256] = &[];
const fn num_bits<T>() -> usize { std::mem::size_of::<T>() * 8 }

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

// pub fn zero_hash(depth: usize) -> Vec<H256>{
//     let mut hashes = vec![H256::from([0; 32]); MAX_TREE_DEPTH + 1];

//     for i in 0..MAX_TREE_DEPTH {
//         hashes[i + 1] = hash_and_concat(hashes[i], hashes[i]);
//     }

//     hashes
// }

#[derive(Debug, Clone)]
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

    pub fn make_proof(&self, index: usize, depth: usize) -> (H256, Vec<H256>) { 
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

    assert_eq!(proof.len(), get_generalized_index_length(index), "proof length should equal depth");

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




fn hash_and_concat(h1: H256, h2: H256) -> H256 {
    H256::from_slice(&hash(&concat(
        h1.as_bytes().to_vec(),
        h2.as_bytes().to_vec(),
    )))
}

fn get_next_power_of_two(depth: usize) -> usize {
    2usize.pow(depth as u32)      
}
///---------
fn get_previous_power_of_two(x: usize) -> usize {
    if x <= 2 {
        return x;
    } else {
        return 2 * get_previous_power_of_two(x/2);
    }
}

fn maybe_get_next_power_of_two(x: usize) -> usize {
    if x <= 2 {
        return x;
    } else {
        return 2 * maybe_get_next_power_of_two((x+1)/2);
    }    
}

fn concat_generalized_indices(indices: &[usize]) -> usize {
    let o = 1usize;
    for index in indices.iter() {
        o = o * get_previous_power_of_two(*index) + (index - get_previous_power_of_two(*index));
    }
    return o;
}

fn get_generalized_index_length(index: usize) -> usize {
   return log_of!(index, 2., usize);
}



// fn log_2(x: i32) -> u32 {
//     assert!(x > 0);
//     num_bits::<i32>() as u32 - x.leading_zeros() - 1
// }

fn get_generalized_index_bit(index: usize, position: usize) -> bool {
    // ((index >> position) & 0x01) == 1 lighthouse 
    ((index >> position) & 0x01) > 0 //dokumentacija
}

fn generalized_index_sibling(index: usize) -> usize {
    return index^1;
}


fn generalized_index_child(index: usize, right_side: bool) -> usize {
    let is_right = if right_side {1} else {0};
    return index*2 + is_right;
}

fn generalized_index_parent(index: usize) -> usize {
    return round::floor(index / 2, 0);
}
    // return GeneralizedIndex(index // 2)
//----------------------------------------
fn get_branch_indices(tree_index: usize) -> Vec<usize> {
    // """
    // Get the generalized indices of the sister chunks along the path from the chunk with the
    // given tree index to the root.
    // """
    
    let mut o = vec![generalized_index_sibling(tree_index)];
    
    while o.last() > Some(&1usize) {
        let temporary_index = o.last().cloned().unwrap();
        let mut temporary = vec![generalized_index_sibling(generalized_index_parent(temporary_index))];
            o.append(&mut temporary);
    }
    return o;
}

fn get_path_indices(tree_index: usize) -> Vec<usize> {
    //    """
    // Get the generalized indices of the chunks along the path from the chunk with the
    // given tree index to the root.
    // """
    // o = [tree_index]
    
    let mut o = vec![tree_index];
    while o.last() > Some(&1usize) {
        let temporary_index = o.last().cloned().unwrap();
        o.append(&mut vec![generalized_index_parent(temporary_index)]);
    }
    return o; 
}

fn get_helper_indices(indices: &[usize]) -> Vec<usize> {
    
    // """
    // Get the generalized indices of all "extra" chunks in the tree needed to prove the chunks with the given
    // generalized indices. Note that the decreasing order is chosen deliberately to ensure equivalence to the
    // order of hashes in a regular single-item Merkle proof in the single-item case.
    // """

    let mut all_helper_indices: Vec<usize> = vec![];
    let mut all_path_indices: Vec<usize> = vec![];
    for index in indices.iter() {
        all_helper_indices.append(&mut get_branch_indices(*index).clone());
        all_path_indices.append(&mut get_path_indices(*index).clone());      
    }

    //let mut answer: Vec<usize> = vec![1];

   // answer = all_helper_indices.append(&mut all_path_indices);
    
    let answer: Vec<usize> = all_helper_indices.iter().zip(&all_path_indices).filter(|&(a, b)| a != b).collect();
   
    return  answer;
}   
//----------------------------------------
fn m_verify_merkle_proof(leaf: H256, proof: &[H256], index: usize, root: H256) -> bool {
    return m_calculate_merkle_root(leaf, proof, index) == root
}


fn m_calculate_merkle_root(leaf: H256, proof: &[H256], index: usize) -> H256 {
    assert_eq!( proof.len(), get_generalized_index_length(index), "OH SHIET");
    let mut root = leaf.as_bytes().to_vec();

    for (i, leaf) in proof.iter().enumerate() {
        if get_generalized_index_bit(index, i) {    
            let input = concat(leaf.as_bytes().to_vec(), root);
            root = hash(&input);
        } else {
            let mut input = root;
            input.extend_from_slice(leaf.as_bytes());
            root = hash(&input);
        }
    }
    return H256::from_slice(&root);
}

//----------------------------------------

fn m_verify_merkle_multiproof(leaves: &[H256],  proof: &[H256], indices: &[usize], root: H256) -> bool {
    return calculate_multi_merkle_root(leaves, proof, indices) == root
}


fn calculate_multi_merkle_root(leaves: &[H256], proof: &[H256], indices: &[usize]) -> H256 {
    let mut book_reviews = HashMap::new();
    let mut book_reviews = HashMap::new();

    // assert len(leaves) == len(indices)
    let helper_indices = get_helper_indices(indices);
    // assert len(proof) == len(helper_indices)
    let mut book_reviews = HashMap::new();

    objects = {
    **{index: node for index, node in zip(indices, leaves)},
    **{index: node for index, node in zip(helper_indices, proof)}
    }
    keys = sorted(objects.keys(), reverse=True)
    pos = 0
    while pos < len(keys) {
        k = keys[pos]
        if k in objects and k ^ 1 in objects and k // 2 not in objects:
        objects[GeneralizedIndex(k // 2)] = hash(
        objects[GeneralizedIndex((k | 1) ^ 1)] +
        objects[GeneralizedIndex(k | 1)]
        )
        keys.append(GeneralizedIndex(k // 2))
        pos += 1
    }
    // R5da554c325ff5
    return objects[GeneralizedIndex(1)]    
}

//-----------------------------------------






































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