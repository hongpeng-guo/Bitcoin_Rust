use super::hash::{Hashable, H256};
pub fn concat_u8(first: &[u8], second: &[u8]) -> Vec<u8> {
    [first, second].concat()
}

/// A Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
        leaf_size: usize,
        height: usize,
        root: H256,
        all_entry: Vec<H256>,
}

impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self where T: Hashable, {
        let mut l_leaf_size = data.len();
        let mut l_height = 0;
        let mut l_all_entry: Vec<H256> = vec![<H256>::from([0;32]); l_leaf_size*l_leaf_size];
        let mut current_layer: Vec<H256>=Vec::new();
        let mut upper_layer: Vec<H256>=Vec::new();
        for i in 0..data.len(){
            current_layer.push(super::hash::Hashable::hash(&data[i]));
        }
        let mut concatenated:Vec<u8>=Vec::new();
        let mut vec_1:[u8; 32]=[0; 32];
        let mut vec_2:[u8; 32]=[0; 32];
        let mut v_vec_1 = Vec::new();
        let mut v_vec_2 = Vec::new();
        while(current_layer.len()>1){
	        if(current_layer.len()%2!=0){
	            current_layer.push(<H256>::from([0;32]));
	        }
                for i in 0..current_layer.len(){
                	l_all_entry[l_height*l_leaf_size+i]=current_layer[i].clone();
                }
                l_height = l_height + 1;
	        for i in 0..current_layer.len()/2{
	            vec_1=current_layer[i*2].into();
	            vec_2=current_layer[i*2+1].into();
	            v_vec_1 = vec_1.to_vec();
                    v_vec_2 = vec_2.to_vec();
                    v_vec_1.extend(v_vec_2);
                    concatenated = v_vec_1;
                    let mut c: &[u8] = &concatenated;
	            upper_layer.push(<H256>::from(ring::digest::digest(&ring::digest::SHA256, c)));
	        }
	        current_layer = upper_layer;
	        upper_layer = Vec::new();
        }
        let l_root = current_layer[0];
        MerkleTree{leaf_size: l_leaf_size, height: l_height, root: l_root, all_entry: l_all_entry}
    }

    pub fn root(&self) -> H256 {
        self.root
    }

    /// Returns the Merkle Proof of data at index i
    pub fn proof(&self, index: usize) -> Vec<H256> {
        let mut vec_proof: Vec<H256> = Vec::new();
        let mut index_current: usize = index;
        for i in 0..self.height{
            let mut index_append: usize = 0;
            if(index_current%2!=0){
                 index_append = index_current-1;
             }
             else{
                 index_append = index_current+1;
             }
            vec_proof.push(self.all_entry[i*self.leaf_size+index_append]);
            index_current = (index_current-index_current%2)/2;
        }
	    vec_proof
    }
}

/// Verify that the datum hash with a vector of proofs will produce the Merkle root. Also need the
/// index of datum and `leaf_size`, the total number of leaves.
pub fn verify(root: &H256, datum: &H256, proof: &[H256], index: usize, leaf_size: usize) -> bool {
    let mut length_proof = proof.len();
    let mut current_hash = *datum;
    for i in 0..length_proof{
        let mut vec_1:[u8; 32]=current_hash.into();
        let mut vec_2:[u8; 32]=proof[i].into();
        let mut v_vec_1:Vec<u8> = vec_1.to_vec();
        let mut v_vec_2:Vec<u8> = vec_2.to_vec();
        v_vec_1.extend(v_vec_2);
        let concatenated:Vec<u8>=v_vec_1;
        let mut c: &[u8] = &concatenated;
	current_hash = <H256>::from(ring::digest::digest(&ring::digest::SHA256, c));
   }
   current_hash==*root
}

#[cfg(test)]
mod tests {
    use crate::crypto::hash::H256;
    use super::*;

    macro_rules! gen_merkle_tree_data {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
            ]
        }};
    }

    #[test]
    fn root() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920")).into()
        );
        // "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        // "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
        // "6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        // the concatenation of these two hashes "b69..." and "965..."
        // notice that the order of these two matters
    }

    #[test]
    fn proof() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(proof,
                   vec![hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f").into()]
        );
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
    }

    #[test]
    fn verifying() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert!(verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 0, input_data.len()));
    }
}
