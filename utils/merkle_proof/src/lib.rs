use eth2_hashing::hash;
use ethereum_types::H256;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::FromIterator;

#[derive(Debug, PartialEq)]
pub enum MerkleProofError {
    /// Params of not equal length were given
    InvalidParamLength { len_first: usize, len_second: usize },
}

#[macro_use]
// logoritmic function
macro_rules! log_of {
    ($val:expr, $base:expr, $type:ty) => {
        ($val as f32).log($base) as $type
    };
}

//concats 2 vectors
fn concat(mut vec1: Vec<u8>, mut vec2: Vec<u8>) -> Vec<u8> {
    vec1.append(&mut vec2);
    vec1
}

// concats and then hashes 2 vectors
fn hash_and_concat(h1: H256, h2: H256) -> H256 {
    H256::from_slice(&hash(&concat(
        h1.as_bytes().to_vec(),
        h2.as_bytes().to_vec(),
    )))
}
//returns previous power of 2
fn get_previous_power_of_two(x: usize) -> usize {
    if x <= 2 {
        x
    } else {
        2 * get_previous_power_of_two(x / 2)
    }
}

//returns next power of 2
fn get_next_power_of_two(x: usize) -> usize {
    if x <= 2 {
        x
    } else {
        2 * get_next_power_of_two((x + 1) / 2)
    }
}

// length of path
fn get_generalized_index_length(index: usize) -> usize {
    log_of!(index, 2., usize)
}

const fn get_generalized_index_bit(index: usize, position: usize) -> bool {
    ((index >> position) & 0x01) > 0
}

//get index sibling
const fn generalized_index_sibling(index: usize) -> usize {
    index ^ 1
}

// get index child
fn generalized_index_child(index: usize, right_side: bool) -> usize {
    let is_right = if right_side { 1 } else { 0 };
    index * 2 + is_right
}

//get index parent
const fn generalized_index_parent(index: usize) -> usize {
    index / 2
}

// get indices of sister chunks
fn get_branch_indices(tree_index: usize) -> Vec<usize> {
    let mut branch = vec![generalized_index_sibling(tree_index)];
    while branch.last() > Some(&1_usize) {
        let index = branch.last().cloned().expect("Something is wrong");
        let mut next_index = vec![generalized_index_sibling(generalized_index_parent(index))];
        branch.append(&mut next_index);
    }
    branch
}

// get path indices
fn get_path_indices(tree_index: usize) -> Vec<usize> {
    let mut path = vec![tree_index];
    while path.last() > Some(&1_usize) {
        let index = path.last().cloned().expect("Something is wrong");
        path.append(&mut vec![generalized_index_parent(index)]);
    }
    path
}

//get all indices of all indices needed for the proof
fn get_helper_indices(indices: &[usize]) -> Vec<usize> {
    let mut all_helper_indices: Vec<usize> = vec![];
    let mut all_path_indices: Vec<usize> = vec![];
    for index in indices.iter() {
        all_helper_indices.append(&mut get_branch_indices(*index).clone());
        all_path_indices.append(&mut get_path_indices(*index).clone());
    }

    let pre_answer = hashset(&all_helper_indices);
    let pre_answer_2 = hashset(&all_path_indices);

    let mut hash_answer: HashSet<usize> = pre_answer.difference(&pre_answer_2).cloned().collect();
    let mut vector_answer: Vec<usize> = Vec::with_capacity(hash_answer.len());

    for i in hash_answer.drain() {
        vector_answer.push(i);
    }

    vector_answer.sort();
    reverse_vector(&vector_answer)
}

//reverts the vector
fn reverse_vector(data: &[usize]) -> Vec<usize> {
    data.iter().rev().cloned().collect()
}

//vector to hashset
fn hashset(data: &[usize]) -> HashSet<usize> {
    HashSet::from_iter(data.iter().cloned())
}

// merkle proof
pub fn verify_merkle_proof(
    leaf: H256,
    proof: &[H256],
    _depth: usize, // not needed
    index: usize,
    root: H256,
) -> Result<bool, MerkleProofError> {
    match calculate_merkle_root(leaf, proof, index) {
        Ok(calculated_root) => Ok(calculated_root == root),
        Err(err) => Err(err),
    }
}

fn calculate_merkle_root(
    leaf: H256,
    proof: &[H256],
    index: usize,
) -> Result<H256, MerkleProofError> {
    if proof.len() != get_generalized_index_length(index) {
        return Err(MerkleProofError::InvalidParamLength {
            len_first: proof.len(),
            len_second: get_generalized_index_length(index),
        });
    }
    let mut root = leaf.as_bytes().to_vec();

    for (i, leaf) in proof.iter().enumerate() {
        if get_generalized_index_bit(index, i) {
            //select how leaf's are concated
            let input = concat(leaf.as_bytes().to_vec(), root);
            root = hash(&input);
        } else {
            let mut input = root;
            input.extend_from_slice(leaf.as_bytes());
            root = hash(&input);
        }
    }
    Ok(H256::from_slice(&root))
}

pub fn verify_merkle_multiproof(
    leaves: &[H256],
    proof: &[H256],
    indices: &[usize],
    root: H256,
) -> Result<bool, MerkleProofError> {
    match calculate_multi_merkle_root(leaves, proof, indices) {
        Ok(calculated_root) => Ok(calculated_root == root),
        Err(err) => Err(err),
    }
}

fn calculate_multi_merkle_root(
    leaves: &[H256],
    proof: &[H256],
    indices: &[usize],
) -> Result<H256, MerkleProofError> {
    let mut index_leave_map = HashMap::new();
    let mut helper_proof_map = HashMap::new();

    if leaves.len() != indices.len() {
        return Err(MerkleProofError::InvalidParamLength {
            len_first: leaves.len(),
            len_second: indices.len(),
        });
    }

    let helper_indices = get_helper_indices(indices);

    for (index, leave) in indices.iter().zip(leaves.iter()) {
        index_leave_map.insert(*index, *leave);
    }

    for (helper_step, proof_step) in helper_indices.iter().zip(proof.iter()) {
        helper_proof_map.insert(*helper_step, *proof_step);
    }

    index_leave_map.extend(helper_proof_map);

    let mut keys: Vec<usize> = vec![];

    for key in index_leave_map.keys() {
        keys.push(key.clone());
    }

    keys.sort();
    keys = reverse_vector(&keys);
    let mut biggest: usize = *keys.get(0_usize).clone().expect("No keys");

    while biggest > 0 {
        if !keys.contains(&biggest) {
            keys.push(biggest);
        }
        biggest -= 1;
    }

    keys.sort();
    keys = reverse_vector(&keys);

    let mut position = 1_usize;

    while position < keys.len() {
        // Safe because keys vector is filled above.
        let k = keys[position];
        let contains_itself: bool = index_leave_map.contains_key(&k);
        let contains_sibling: bool = index_leave_map.contains_key(&(k ^ 1));
        let contains_parent: bool = index_leave_map.contains_key(&(k / 2));

        if contains_itself && contains_sibling && !contains_parent {
            let index_first: usize = (k | 1) ^ 1; //right
            let index_second: usize = k | 1; //left

            index_leave_map.insert(
                k / 2,
                hash_and_concat(
                    index_leave_map[&index_first],
                    index_leave_map[&index_second],
                ),
            );
        }
        position += 1;
    }

    // Safe because keys vector is full and value is inserted in those indeces.
    // index_leave_map.remove(&1usize);
    Ok(index_leave_map[&1_usize])
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
    fn get_generalized_index_length_test() {
        assert_eq!(get_generalized_index_length(4), 2);
        assert_eq!(get_generalized_index_length(7), 2);
        assert_eq!(get_generalized_index_length(9), 3);
    }

    #[test]
    fn get_generalized_index_bit_test() {
        assert_eq!(true, get_generalized_index_bit(2_usize, 1_usize));
        assert_eq!(false, get_generalized_index_bit(3, 2));
    }

    #[test]
    fn generalized_index_sibling_test() {
        assert_eq!(generalized_index_sibling(3), 2);
    }

    #[test]
    fn generalized_index_child_test() {
        assert_ne!(generalized_index_child(3, false), 7);
        assert_eq!(generalized_index_child(5, true), 11);
    }

    #[test]
    fn get_branch_indices_test() {
        assert_eq!(get_branch_indices(5_usize), vec!(4_usize, 3_usize, 0_usize));
        assert_eq!(
            get_branch_indices(9_usize),
            vec!(8_usize, 5_usize, 3_usize, 0_usize)
        );
    }

    #[test]
    fn get_path_indices_test() {
        assert_eq!(
            get_path_indices(9_usize),
            vec!(9_usize, 4_usize, 2_usize, 1_usize)
        );
        assert_eq!(
            get_path_indices(10_usize),
            vec!(10_usize, 5_usize, 2_usize, 1_usize)
        );
    }

    #[test]
    fn get_helper_indices_test() {
        assert_eq!(
            get_helper_indices(&[9_usize, 4_usize, 2_usize, 1_usize]),
            vec!(8_usize, 5_usize, 3_usize, 0_usize)
        );
        assert_eq!(
            get_helper_indices(&[10_usize, 5_usize, 2_usize, 1_usize]),
            vec!(11_usize, 4_usize, 3_usize, 0_usize)
        );
    }

    #[test]
    fn verify_merkle_proof_test() {
        let leaf_b00 = H256::from([0xAA; 32]); //4
        let leaf_b01 = H256::from([0xBB; 32]); //5
        let leaf_b10 = H256::from([0xCC; 32]); //6
        let leaf_b11 = H256::from([0xDD; 32]); //7

        let node_b0x = hash_and_concat(leaf_b00, leaf_b01); //3
        let node_b1x = hash_and_concat(leaf_b10, leaf_b11); //2

        let root = hash_and_concat(node_b0x, node_b1x); //1

        assert_eq!(
            verify_merkle_proof(leaf_b00, &[leaf_b01, node_b1x], 0, 4, root)
                .expect("verification failed!"),
            true
        );

        assert_eq!(
            verify_merkle_proof(leaf_b01, &[leaf_b00, node_b1x], 0, 5, node_b1x),
            Ok(false)
        );

        assert_eq!(
            verify_merkle_proof(leaf_b01, &[leaf_b01, leaf_b00, node_b1x], 0, 5, node_b1x),
            Err(MerkleProofError::InvalidParamLength {
                len_first: 3,
                len_second: 2
            })
        );

        assert_eq!(
            verify_merkle_proof(leaf_b01, &[leaf_b01], 0, 5, node_b1x),
            Err(MerkleProofError::InvalidParamLength {
                len_first: 1,
                len_second: 2
            })
        );

        assert_eq!(
            verify_merkle_proof(leaf_b00, &[node_b1x, leaf_b01], 0, 4, root)
                .expect("verification failed!"),
            false
        );

        assert_eq!(
            verify_merkle_proof(leaf_b01, &[leaf_b00, node_b1x], 0, 5, root)
                .expect("verification failed!"),
            true
        );

        assert_eq!(
            verify_merkle_proof(leaf_b10, &[leaf_b11, node_b0x], 0, 6, root)
                .expect("verification failed!"),
            true
        );

        assert_eq!(
            verify_merkle_proof(leaf_b11, &[leaf_b10, node_b0x], 0, 7, root)
                .expect("verification failed!"),
            true
        );

        assert_eq!(
            verify_merkle_proof(leaf_b11, &[leaf_b10], 0, 3, node_b1x)
                .expect("verification failed!"),
            true
        );

        assert_eq!(
            verify_merkle_proof(leaf_b01, &[], 0, 1, root).expect("verification failed!"),
            false
        );

        assert_eq!(
            verify_merkle_proof(leaf_b01, &[node_b1x, leaf_b00], 0, 5, root)
                .expect("verification failed!"),
            false
        );

        assert_eq!(
            verify_merkle_proof(leaf_b01, &[leaf_b00], 0, 2, root).expect("verification failed!"),
            false
        );

        assert_eq!(
            verify_merkle_proof(leaf_b01, &[leaf_b00, node_b1x], 0, 4, root)
                .expect("verification failed!"),
            false
        );

        assert_eq!(
            verify_merkle_proof(leaf_b01, &[leaf_b00, node_b1x], 0, 5, node_b1x)
                .expect("verification failed!"),
            false
        );
    }

    #[test]
    fn verify_merkle_multiproof_first_test() {
        let leaf_b00 = H256::from([0xAA; 32]); //4
        let leaf_b01 = H256::from([0xBB; 32]); //5
        let leaf_b10 = H256::from([0xCC; 32]); //6
        let leaf_b11 = H256::from([0xDD; 32]); //7

        let node_b0x = hash_and_concat(leaf_b00, leaf_b01); // 3
        let node_b1x = hash_and_concat(leaf_b10, leaf_b11); //2

        let root = hash_and_concat(node_b0x, node_b1x); //1

        assert_eq!(
            verify_merkle_multiproof(
                &[leaf_b00, leaf_b01, leaf_b11],
                &[leaf_b10, node_b1x],
                &[4, 5, 7],
                root
            )
            .expect("verification of multiproof failed!"),
            true
        );

        assert_eq!(
            verify_merkle_multiproof(
                &[leaf_b00, leaf_b01, leaf_b10, leaf_b11],
                &[],
                &[4, 5, 6, 7],
                root
            )
            .expect("verification of multiproof failed!"),
            true
        );

        assert_eq!(
            verify_merkle_multiproof(
                &[leaf_b00, leaf_b01, leaf_b10],
                &[leaf_b10, node_b1x],
                &[4, 5, 7],
                root
            )
            .expect("verification of multiproof failed!"),
            false
        );

        assert_eq!(
            verify_merkle_multiproof(
                &[leaf_b00, leaf_b10, leaf_b01],
                &[leaf_b11, node_b1x],
                &[4, 5, 6],
                root
            )
            .expect("verification of multiproof failed!"),
            false
        );

        assert_eq!(
            verify_merkle_multiproof(
                &[leaf_b00, leaf_b01, leaf_b10],
                &[leaf_b11, node_b1x],
                &[4, 5, 6],
                root
            )
            .expect("verification of multiproof failed!"),
            true
        );

        assert_eq!(
            verify_merkle_multiproof(&[leaf_b00, leaf_b01], &[node_b1x, node_b1x], &[4, 5], root)
                .expect("verification of multiproof failed!"),
            true
        );

        assert_eq!(
            verify_merkle_multiproof(&[leaf_b00], &[leaf_b01, node_b1x], &[4], root)
                .expect("verification of multiproof failed!"),
            true
        );

        assert_eq!(
            verify_merkle_multiproof(&[leaf_b01], &[leaf_b00, node_b1x], &[5], root)
                .expect("verification of multiproof failed!"),
            true
        );
    }

    #[test]
    fn verify_merkle_multiproof_second_test() {
        let leaf_b00 = H256::from([0xAA; 32]); //4
        let leaf_b01 = H256::from([0xBB; 32]); //5
        let leaf_b10 = H256::from([0xCC; 32]); //6
        let leaf_b11 = H256::from([0xDD; 32]); //7

        let node_b0x = hash_and_concat(leaf_b00, leaf_b01); // 3
        let node_b1x = hash_and_concat(leaf_b10, leaf_b11); //2

        let root = hash_and_concat(node_b0x, node_b1x); //1

        assert_eq!(
            verify_merkle_multiproof(&[leaf_b10], &[leaf_b11, node_b0x], &[6], root)
                .expect("verification of multiproof failed!"),
            true
        );

        assert_eq!(
            verify_merkle_multiproof(&[leaf_b11], &[leaf_b10, node_b0x], &[7], root)
                .expect("verification of multiproof failed!"),
            true
        );

        assert_eq!(
            verify_merkle_multiproof(&[leaf_b11], &[leaf_b10], &[3], node_b1x)
                .expect("verification of multiproof failed!"),
            true
        );

        assert_eq!(
            verify_merkle_multiproof(&[leaf_b01], &[], &[1], root)
                .expect("verification of multiproof failed!"),
            false
        );

        assert_eq!(
            verify_merkle_multiproof(&[leaf_b01], &[node_b1x, leaf_b00], &[5], root)
                .expect("verification of multiproof failed!"),
            false
        );

        assert_eq!(
            verify_merkle_multiproof(&[leaf_b01], &[leaf_b00], &[2], root)
                .expect("verification of multiproof failed!"),
            false
        );

        assert_eq!(
            verify_merkle_multiproof(&[leaf_b01], &[leaf_b00, node_b1x], &[4], root)
                .expect("verification of multiproof failed!"),
            false
        );

        assert_eq!(
            verify_merkle_multiproof(&[leaf_b01], &[leaf_b00, node_b1x], &[5], node_b1x)
                .expect("verification of multiproof failed!"),
            false
        );

        assert_eq!(
            verify_merkle_multiproof(&[leaf_b01, node_b0x], &[leaf_b00, node_b1x], &[5], node_b1x),
            Err(MerkleProofError::InvalidParamLength {
                len_first: 2,
                len_second: 1
            })
        );

        assert_eq!(
            verify_merkle_multiproof(&[leaf_b11, leaf_b10], &[node_b0x], &[7, 6], root),
            Ok(true)
        );
    }

    #[test]
    fn verify_merkle_proof_bigger_test() {
        let leaf_b000 = H256::from([0xAA; 32]); //8
        let leaf_b001 = H256::from([0xBB; 32]); //9
        let leaf_b010 = H256::from([0xCC; 32]); //10
        let leaf_b011 = H256::from([0xDD; 32]); //11

        let node_b00x = hash_and_concat(leaf_b000, leaf_b001); //4
        let node_b01x = hash_and_concat(leaf_b010, leaf_b011); //5

        let leaf_b100 = H256::from([0xAA; 32]); //12
        let leaf_b101 = H256::from([0xBB; 32]); //13
        let leaf_b110 = H256::from([0xCC; 32]); //14
        let leaf_b111 = H256::from([0xDD; 32]); //15

        let node_b10x = hash_and_concat(leaf_b100, leaf_b101); //6
        let node_b11x = hash_and_concat(leaf_b110, leaf_b111); //7

        let node_b0xx = hash_and_concat(node_b00x, node_b01x); //2
        let node_b1xx = hash_and_concat(node_b10x, node_b11x); //3

        let root = hash_and_concat(node_b0xx, node_b1xx); //1

        assert_eq!(
            get_path_indices(15_usize),
            vec!(15_usize, 7_usize, 3_usize, 1_usize)
        );

        assert_eq!(
            verify_merkle_proof(leaf_b000, &[leaf_b001, node_b01x, node_b1xx], 0, 8, root),
            Ok(true)
        );

        assert_eq!(
            verify_merkle_proof(leaf_b000, &[leaf_b001, node_b01x, node_b1xx], 0, 9, root),
            Ok(false)
        );

        assert_eq!(
            verify_merkle_proof(
                leaf_b000,
                &[leaf_b001, node_b01x, node_b1xx, node_b00x],
                0,
                9,
                root
            ),
            Err(MerkleProofError::InvalidParamLength {
                len_first: 4,
                len_second: 3
            })
        );
    }
}
