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
    pub index:  usize,
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
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Vec<u8> {
    let t_serialized: Vec<u8> = bincode::serialize(&t).unwrap();
    key.sign(&t_serialized).as_ref().to_vec()
}

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: Vec<u8>, signature: Vec<u8>) -> bool {
    let t_serialized: Vec<u8> = bincode::serialize(&t).unwrap();
    let peer_public_key_bytes = &public_key[..];
    let peer_public_key = ring::signature::UnparsedPublicKey::new(&signature::ED25519, peer_public_key_bytes);
    peer_public_key.verify(&t_serialized, &signature[..]).is_ok()
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


pub struct State {
    pub data: HashMap<(H256, usize), (u64, H160)>,
}

impl State{
    pub fn new() -> Self {
        State{data: HashMap::new()}
    }

    pub fn update(&mut self, transactions: Vec<SignedTransaction>) {
        for signed_tx in transactions{
            // signature checks of the transaction
            if verify(&signed_tx.transaction, signed_tx.clone().pub_key, signed_tx.clone().signature) == false{
                continue;
            }
            // double spend checks of the transaction
            if self.data.contains_key(&(signed_tx.transaction.in_put[0].tx_hash, signed_tx.transaction.in_put[0].index)) == false{
                continue;
            }
            self.data.remove(&(signed_tx.transaction.in_put[0].tx_hash, signed_tx.transaction.in_put[0].index));
            for (i, output) in signed_tx.transaction.out_put.iter().enumerate(){
                self.data.insert((signed_tx.hash(), i),(output.value, output.address));
            }
        } 
    }
}


pub fn ico3_proc(self_pubkey_vec: Vec<[u8; 32]>) -> State{
    let mut ico_state = State::new();
    ico_state.data.insert((H256::from([0; 32]), 0), (10000, H160::from(self_pubkey_vec[0])));
    ico_state.data.insert((H256::from([0; 32]), 1), (10000, H160::from(self_pubkey_vec[1])));
    ico_state.data.insert((H256::from([0; 32]), 2), (10000, H160::from(self_pubkey_vec[2])));
    ico_state
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
        let signature = sign(&t, &key);
        let pub_key = key.public_key().as_ref().to_vec();
        SignedTransaction{transaction: t, signature: signature, pub_key: pub_key}
    }

    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(&t, key.public_key().as_ref().to_vec(), signature));
    }
}
