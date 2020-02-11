use crate::block::Block;
use std::collections::HashMap;
use crate::crypto::hash::{H256, Hashable};
use rand::Rng;
use crate::block::test::generate_random_block;

pub struct Blockchain {
data: HashMap<H256, BlockStruct>,
tip_hash: H256,
tip_height: u32,
}

struct BlockStruct {
block_content: Block,
block_height: u32,
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let root_array:[u8; 32]=[rng.gen(); 32];
        let root_hash: H256 = root_array.into();
        let mut genesis_block: Block = generate_random_block(&root_hash);
        let mut data_new = HashMap::new();
        data_new.insert(Hashable::hash(&genesis_block), BlockStruct{block_content: genesis_block.clone(), block_height: 0});
        Blockchain{data: data_new, tip_hash: Hashable::hash(&genesis_block), tip_height: 0}
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
        let this_height =(self.data[&block.header.parent]).block_height+1;
        self.data.insert(Hashable::hash(block), BlockStruct{block_content: (*block).clone(), block_height: this_height});
        if(this_height > self.tip_height){
            self.tip_height = self.tip_height+1;
            self.tip_hash =Hashable::hash(block);
        }
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        self.tip_hash
    }

    /// Get the last block's hash of the longest chain
    #[cfg(any(test, test_utilities))]
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        unimplemented!()
    }
}

#[cfg(any(test, test_utilities))]
mod tests {
    use super::*;
    use crate::block::test::generate_random_block;
    use crate::crypto::hash::Hashable;

    #[test]
    fn insert_one() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);
        blockchain.insert(&block);
        assert_eq!(blockchain.tip(), block.hash());

    }
}
