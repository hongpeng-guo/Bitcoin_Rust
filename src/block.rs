use serde::{Serialize, Deserialize};
use crate::crypto::hash::{H256, Hashable};
extern crate rand;
use rand::Rng;
use crate::transaction::{Transaction, tests};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
pub header: Header,
pub content: Content,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
pub parent: H256,
pub nonce: u32,
pub difficulty: H256,
pub timestamp: u128,
pub merkle_root: H256,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
pub content: Vec<Transaction>,
}

impl Hashable for Transaction {
    fn hash(&self) -> H256 {
        let t_serialized: Vec<u8> = bincode::serialize(self).unwrap();
        ring::digest::digest(&ring::digest::SHA256, &t_serialized).into()
    }
}

impl Hashable for Header {
    fn hash(&self) -> H256 {
        let h_serialized: Vec<u8> = bincode::serialize(self).unwrap();
        ring::digest::digest(&ring::digest::SHA256, &h_serialized).into()
    }
}

impl Hashable for Block {
    fn hash(&self) -> H256 {
        Hashable::hash(&self.header)
    }
}

pub mod test {
    use super::*;
    use crate::crypto::hash::H256;

    pub fn generate_random_block(parent: &H256) -> Block {
        let mut rng = rand::thread_rng();
        let mut default_transaction: Vec<Transaction> = Vec::new();
        let t = tests::generate_random_transaction();
        default_transaction.push(t);
        let mut difficulty_array:[u8; 32]=[0; 32];
        difficulty_array[2] = 64;
        let default_merkle_root: H256 = Hashable::hash(&default_transaction[0]);
        let random_header = Header{parent: *parent, nonce: rng.gen(), difficulty: difficulty_array.into(), timestamp: rng.gen(), merkle_root: default_merkle_root};
        let random_content = Content{content: default_transaction};
        let random_block = Block{header: random_header, content: random_content};
        random_block
    }

    pub fn generate_static_block(parent: &H256) -> Block {
        let mut default_transaction: Vec<Transaction> = Vec::new();
        let t = tests::generate_random_transaction();
        default_transaction.push(t);
        let mut difficulty_array:[u8; 32]=[0; 32];
        difficulty_array[2] = 128;
        let default_merkle_root: H256 = Hashable::hash(&default_transaction[0]);
        let static_header = Header{parent: *parent, nonce: 1, difficulty: difficulty_array.into(), timestamp: 12345, merkle_root: default_merkle_root};
        let static_content = Content{content: default_transaction};
        let static_block = Block{header: static_header, content: static_content};
        static_block
    }
}
