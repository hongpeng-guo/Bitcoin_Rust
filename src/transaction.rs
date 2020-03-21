use serde::{Serialize,Deserialize};
use ring::signature::{self, Ed25519KeyPair, Signature, KeyPair};
use crate::crypto::hash::{H256, H160, Hashable};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Default, Clone, Hash)]
pub struct Transaction {
    pub in_put:  Vec<Input>,
    pub out_put:  Vec<Output>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Hash)]
pub struct Input {
    // tx_hash is the hash value of previous transaction
    pub tx_hash:  H256,
    // index refers to a specific output number in pre tx
    pub index:  u8,
    // bool variable indicates if this tx is generated w block
    pub coin_base: bool,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Hash)]
pub struct Output {
    pub address:  H160,
    pub value:  u64,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Hash)]
pub struct SignedTransaction {
    pub transaction: Transaction,
    pub signature: Vec<u8>,
    pub pub_key: Vec<u8>,
}

impl Hashable for SignedTransaction {
    fn hash(&self) -> H256 {
        Hashable::hash(&self.transaction)
    }
}


/// Create digital signature of a transaction
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
    let t_serialized: Vec<u8> = bincode::serialize(&t).unwrap();
    key.sign(&t_serialized)
}

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: &<Ed25519KeyPair as KeyPair>::PublicKey, signature: &Signature) -> bool {
    let t_serialized: Vec<u8> = bincode::serialize(&t).unwrap();
    let peer_public_key_bytes = public_key.as_ref();
    let peer_public_key = ring::signature::UnparsedPublicKey::new(&signature::ED25519, peer_public_key_bytes);
    peer_public_key.verify(&t_serialized, signature.as_ref()).is_ok()
}


pub struct Mempool {
    pub data: HashMap<H256, SignedTransaction>,
    pub total_size: u32,
}

impl Mempool{
    pub fn new() -> Self {
        let data_new = HashMap::new();
        Mempool{data: data_new, total_size: 0}
    }

    pub fn insert(&mut self, transaction: &SignedTransaction) {
        self.data.insert(Hashable::hash(transaction), transaction.clone());
        self.total_size += 1;
    }
}

// #[cfg(any(test, test_utilities))]
pub mod tests {
    use super::*;
    use crate::crypto::key_pair;

    pub fn generate_random_transaction() -> Transaction {
        let input: Vec<Input> = vec![Input{tx_hash: H256::from([0; 32]), index: 0, coin_base: false}];
        let output: Vec<Output> = vec![Output{address: H160::from([0; 32]), value: 50}];
        Transaction{in_put: input, out_put: output}
    }

    pub fn generate_random_signedtransaction() -> SignedTransaction{
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key).as_ref().to_vec();
        let pub_key = key.public_key().as_ref().to_vec();
        SignedTransaction{transaction: t, signature: signature, pub_key: pub_key}
    }

    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(&t, &(key.public_key()), &signature));
    }
}
