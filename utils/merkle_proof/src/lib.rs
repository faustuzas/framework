use std::collections::HashMap;
use eth2_hashing::hash;
use ethereum_types::H256;
use std::collections::HashSet;
use std::iter::FromIterator;

#[derive(Debug, PartialEq)]
pub enum MerkleProofError {
    /// Params of not eqal length were given
    InvalidParamLength { len_first: usize, len_second: usize },
}

#[macro_use]
macro_rules! log_of {
    ($val:expr, $base:expr, $type:ty) => {
         ($val as f32).log($base) as $type
    }
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

fn get_previous_power_of_two(x: usize) -> usize {
    if x <= 2 {
        return x;
    } else {
        return 2 * get_previous_power_of_two(x/2);
    }
}

fn get_next_power_of_two(x: usize) -> usize {
    if x <= 2 {
        return x;
    } else {
        return 2 * get_next_power_of_two((x+1)/2);
    }    
}

fn concat_generalized_indices(indices: &[usize]) -> usize {
    let mut cancated_indices = 1usize;
    for index in indices.iter() {
        cancated_indices = cancated_indices * get_previous_power_of_two(*index) + (index - get_previous_power_of_two(*index));
    }
    return cancated_indices;
}

fn get_generalized_index_length(index: usize) -> usize {
   return log_of!(index, 2., usize);
}

fn get_generalized_index_bit(index: usize, position: usize) -> bool {
    ((index >> position) & 0x01) > 0 
}

fn generalized_index_sibling(index: usize) -> usize {
    return index^1;
}


fn generalized_index_child(index: usize, right_side: bool) -> usize {
    let is_right = if right_side {1} else {0};
    return index*2 + is_right;
}

fn generalized_index_parent(index: usize) -> usize {
    return index / 2;
}

fn get_branch_indices(tree_index: usize) -> Vec<usize> {
    let mut branch = vec![generalized_index_sibling(tree_index)];
    
    while branch.last() > Some(&1usize) {
        let index = branch.last().cloned().unwrap();
        let mut next_index = vec![generalized_index_sibling(generalized_index_parent(index))];
            branch.append(&mut next_index);
    }
    return branch;
}

fn get_path_indices(tree_index: usize) -> Vec<usize> {
    let mut path = vec![tree_index];
    while path.last() > Some(&1usize) {
        let index = path.last().cloned().unwrap();
        path.append(&mut vec![generalized_index_parent(index)]);
    }
    return path; 
}

fn get_helper_indices(indices: &[usize]) -> Vec<usize> {
    let mut all_helper_indices: Vec<usize> = vec![];
    let mut all_path_indices: Vec<usize> = vec![];
    for index in indices.iter() {
        all_helper_indices.append(&mut get_branch_indices(*index).clone());
        all_path_indices.append(&mut get_path_indices(*index).clone());      
    }

    let pre_answer = hashset(all_helper_indices);
    let pre_answer_2 = hashset(all_path_indices);

    let mut hash_answer: HashSet<usize> = pre_answer.difference(&pre_answer_2).cloned().collect();
    let mut vector_answer: Vec<usize> = Vec::with_capacity(hash_answer.len());

    for i in hash_answer.drain() {
        vector_answer.push(i);
    }

    vector_answer.sort();
    return reverse_vector(vector_answer);
}   

fn reverse_vector(data: Vec<usize> ) -> Vec<usize> {
    return data.iter().rev().cloned().collect();
}

fn hashset(data: Vec<usize> ) -> HashSet<usize> {
    HashSet::from_iter(data.iter().cloned())
}

fn verify_merkle_proof(leaf: H256, proof: &[H256], index: usize, root: H256) -> Result<bool, MerkleProofError> {
    match calculate_merkle_root(leaf, proof, index) {
        Ok(calculated_root) => Ok(calculated_root== root),
        Err(err) => Err(err),
    }
    // return calculate_merkle_root(leaf, proof, index) == root
}

fn calculate_merkle_root(leaf: H256, proof: &[H256], index: usize) -> Result<H256, MerkleProofError> {
    // assert_eq!( proof.len(), get_generalized_index_length(index), "Length of proof should equal generalized index depth");
    if proof.len() != get_generalized_index_length(index) {
        return Err(MerkleProofError::InvalidParamLength {
            len_first: proof.len(), 
            len_second:  get_generalized_index_length(index)
        });
    }
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
    Ok(H256::from_slice(&root))
    // return H256::from_slice(&root);
}

fn verify_merkle_multiproof(leaves: &[H256],  proof: &[H256], indices: &[usize], root: H256) -> Result<bool, MerkleProofError> {
    match calculate_multi_merkle_root(leaves, proof, indices) {
        Ok(calculated_root) => Ok(calculated_root== root),
        Err(err) => Err(err)
        //Err(MerkleProofError::InvalidParamLength) => Err(err),
    }
    //Ok(calculate_multi_merkle_root(leaves, proof, indices) == root)
}

fn calculate_multi_merkle_root(leaves: &[H256], proof: &[H256], indices: &[usize]) -> Result<H256, MerkleProofError> {
    let mut btree_first = HashMap::new();
    let mut btree_second = HashMap::new();

    if leaves.len() != indices.len() {
        return Err(MerkleProofError::InvalidParamLength {
            len_first: leaves.len(), 
            len_second: indices.len()
        });
    }

    let helper_indices = get_helper_indices(indices);
    
    for (index, leave) in indices.iter().zip(leaves.iter()) {
        btree_first.insert(*index, *leave);
    }

    for (helper_step, proof_step) in helper_indices.iter().zip(proof.iter()) {
        btree_second.insert(*helper_step, *proof_step);
    }

    btree_first.extend(btree_second);

    let mut keys: Vec<usize> = vec![];

    for(key, _value) in btree_first.iter_mut() {
        keys.push(key.clone());
    }

    keys.sort();
    keys = reverse_vector(keys);
    let mut position = 1usize;

    while position < keys.len() {
        let k = keys.get(position).unwrap();
        let contains_itself: bool = btree_first.contains_key(k);
        let contains_sibling: bool = btree_first.contains_key(&(k^1));
        let contains_parent: bool = btree_first.contains_key(&(k / 2));

        if contains_itself && contains_sibling && !contains_parent {
            let index_first: usize = (k | 1) ^ 1;
            let index_second: usize = k | 1;
            btree_first.insert(
                k / 2,
                hash_and_concat(
                    *btree_first.get(&index_first).unwrap(),
                    *btree_first.get(&index_second).unwrap())
            );
        }
        position += 1
    }

    return Ok(*btree_first.get(&1usize).unwrap());    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_previous_power_of_two_test() {
        let x: usize = 3;
        assert_eq!(get_previous_power_of_two(x), 2);
    }

    #[test]
    fn get_next_power_of_two_test() {
        let x: usize = 3;
        assert_eq!(get_next_power_of_two(x), 4);
    }

    #[test]
    fn concat_generalized_indices_test() {
        let general_indices = [1usize, 2usize];
        assert_eq!(2, concat_generalized_indices(&general_indices));
        let general_indices = [1usize, 2usize, 3usize];
        assert_eq!(5, concat_generalized_indices(&general_indices));
    }

    #[test]
    fn get_generalized_index_length_test() {
        assert_eq!(get_generalized_index_length(4),2);
        assert_eq!(get_generalized_index_length(7),2);
        assert_eq!(get_generalized_index_length(9),3);
    }
    
    #[test]
    fn get_generalized_index_bit_test() {
        assert_eq!(true, get_generalized_index_bit(2usize, 1usize));
        assert_eq!(false, get_generalized_index_bit(3, 2));
    }

    #[test]
    fn generalized_index_sibling_test() {
        assert_eq!(generalized_index_sibling(3),2);
    }

    #[test]
    fn generalized_index_child_test() {
        assert_ne!(generalized_index_child(3,false),7);
        assert_eq!(generalized_index_child(5,true),11);
    }

    #[test]
    fn get_branch_indices_test() {
        assert_eq!(get_branch_indices(5usize), vec!(4usize,3usize, 0usize));
        assert_eq!(get_branch_indices(9usize), vec!(8usize,5usize, 3usize, 0usize));
    }

    #[test]
    fn get_path_indices_test() {
        assert_eq!(get_path_indices(9usize), vec!(9usize, 4usize, 2usize, 1usize));
        assert_eq!(get_path_indices(10usize), vec!(10usize, 5usize, 2usize, 1usize));
    }

    #[test]
    fn get_helper_indices_test() {
        assert_eq!(get_helper_indices(&[9usize, 4usize, 2usize, 1usize]), vec!(8usize,5usize, 3usize, 0usize));
        assert_eq!(get_helper_indices(&[10usize, 5usize, 2usize, 1usize]), vec!(11usize, 4usize, 3usize, 0usize));
    }

    #[test]
    fn verify_merkle_proof_test() {
        let leaf_b00 = H256::from([0xAA; 32]);
        let leaf_b01 = H256::from([0xBB; 32]);
        let leaf_b10 = H256::from([0xCC; 32]);
        let leaf_b11 = H256::from([0xDD; 32]);

        let node_b0x = hash_and_concat(leaf_b00, leaf_b01);
        let node_b1x = hash_and_concat(leaf_b10, leaf_b11);

        let root = hash_and_concat(node_b0x, node_b1x);

        assert_eq!(verify_merkle_proof(
            leaf_b00,
            &[leaf_b01, node_b1x],
            4,
            root
        ).unwrap(), true);
        
        assert_eq!(verify_merkle_proof(
            leaf_b01,
            &[leaf_b00, node_b1x],
            5,
            root
        ).unwrap(), true);

        assert_eq!(verify_merkle_proof(
            leaf_b10,
            &[leaf_b11, node_b0x],
            6,
            root
        ).unwrap(), true);

        assert_eq!(verify_merkle_proof(
            leaf_b11,
            &[leaf_b10, node_b0x],
            7,
            root
        ).unwrap(), true);

        assert_eq!(verify_merkle_proof(
            leaf_b11,
            &[leaf_b10],
            3,
            node_b1x
        ).unwrap(), true);

        assert_eq!(verify_merkle_proof(leaf_b01, &[], 1, root).unwrap(), false);

        assert_eq!(verify_merkle_proof(
            leaf_b01,
            &[node_b1x, leaf_b00],
            5,
            root
        ).unwrap(), false);

        assert_eq!(verify_merkle_proof(leaf_b01, &[leaf_b00], 2, root).unwrap(), false);

        assert_eq!(verify_merkle_proof(
            leaf_b01,
            &[leaf_b00, node_b1x],
            4,
            root
        ).unwrap(), false);

        assert_eq!(verify_merkle_proof(
            leaf_b01,
            &[leaf_b00, node_b1x],
            5,
            node_b1x
        ).unwrap(), false);
    }

    #[test]
    fn verify_merkle_multiproof_test() {
        let leaf_b00 = H256::from([0xAA; 32]);
        let leaf_b01 = H256::from([0xBB; 32]);
        let leaf_b10 = H256::from([0xCC; 32]);
        let leaf_b11 = H256::from([0xDD; 32]);

        let node_b0x = hash_and_concat(leaf_b00, leaf_b01);
        let node_b1x = hash_and_concat(leaf_b10, leaf_b11);

        let root = hash_and_concat(node_b0x, node_b1x);

        assert_eq!(verify_merkle_multiproof(
            &[leaf_b00,leaf_b01],
            &[node_b1x, node_b1x],
            &[4, 5],
            root
        ).unwrap(), true);

        assert_eq!(verify_merkle_multiproof(
            &[leaf_b00],
            &[leaf_b01, node_b1x],
            &[4],
            root
        ).unwrap(), true);

        assert_eq!(verify_merkle_multiproof(
            &[leaf_b01],
            &[leaf_b00, node_b1x],
            &[5],
            root
        ).unwrap(), true);

        assert_eq!(verify_merkle_multiproof(
            &[leaf_b10],
            &[leaf_b11, node_b0x],
            &[6],
            root
        ).unwrap(), true);

        assert_eq!(verify_merkle_multiproof(
            &[leaf_b11],
            &[leaf_b10, node_b0x],
            &[7],
            root
        ).unwrap(), true);

        assert_eq!(verify_merkle_multiproof(
            &[leaf_b11],
            &[leaf_b10],
            &[3],
            node_b1x
        ).unwrap(), true);

        assert_eq!(verify_merkle_multiproof(&[leaf_b01], &[], &[1], root).unwrap(), false);

        assert_eq!(verify_merkle_multiproof(
            &[leaf_b01],
            &[node_b1x, leaf_b00],
            &[5],
            root
        ).unwrap(), false);

        assert_eq!(verify_merkle_multiproof(&[leaf_b01], &[leaf_b00], &[2], root).unwrap(), false);

        assert_eq!(verify_merkle_multiproof(
            &[leaf_b01],
            &[leaf_b00, node_b1x],
            &[4],
            root
        ).unwrap(), false);

        assert_eq!(verify_merkle_multiproof(
            &[leaf_b01],
            &[leaf_b00, node_b1x],
            &[5],
            node_b1x
        ).unwrap(), false);

        assert_eq!(verify_merkle_multiproof(
            &[leaf_b01, node_b0x],
            &[leaf_b00, node_b1x],
            &[5],
            node_b1x
        ), Err(MerkleProofError::InvalidParamLength { len_first: 2, len_second: 1 }));
    }
}