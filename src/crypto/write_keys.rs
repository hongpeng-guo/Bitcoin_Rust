use ring::rand;
use ring::signature::Ed25519KeyPair;
use std::fs::OpenOptions;
use std::io::Write;

pub fn write_key(){
    for i in 0..10{
        let path = format!("/home/hongpeng/Desktop/Spring20/ECE598/bitcoin_midterm/src/keys/{}.key", i);
        let mut f = OpenOptions::new().write(true).create(true).open(path).expect("cannot open file");
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        f.write(pkcs8_bytes.as_ref()).unwrap();
    }
}