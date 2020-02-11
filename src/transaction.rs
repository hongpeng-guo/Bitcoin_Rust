use serde::{Serialize,Deserialize};
use ring::signature::{self, Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};
use rand::{self, Rng};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
pub in_put:  Vec<u8> ,
pub out_put:  Vec<u8> ,
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

#[cfg(any(test, test_utilities))]
mod tests {
    use super::*;
    use crate::crypto::key_pair;

    pub fn generate_random_transaction() -> Transaction {
        let mut rng = rand::thread_rng();
        let input: Vec<u8> = (0..100).map(|_| {rng.gen_range(0, 255)}).collect();
        let output: Vec<u8> = (0..100).map(|_| {rng.gen_range(0, 255)}).collect();
        Transaction{in_put: input, out_put: output}
    }

    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(&t, &(key.public_key()), &signature));
    }
}
