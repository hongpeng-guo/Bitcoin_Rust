#[cfg(test)]
#[macro_use]
extern crate hex_literal;

pub mod api;
pub mod block;
pub mod blockchain;
pub mod crypto;
pub mod miner;
pub mod network;
pub mod transaction;
pub mod generator;

use clap::clap_app;
use crossbeam::channel;
use log::{error, info};
use api::Server as ApiServer;
use network::{server, worker};
use crypto::key_pair;
use std::net;
use std::process;
use std::thread;
use std::time;
use std::sync::{Arc, Mutex};
use ring::{rand, signature::{Ed25519KeyPair, KeyPair}};
use crate::crypto::hash::{H256, H160};

fn main() {
    // parse command line arguments
    let matches = clap_app!(Bitcoin =>
     (version: "0.1")
     (about: "Bitcoin client")
     (@arg verbose: -v ... "Increases the verbosity of logging")
     (@arg peer_addr: --p2p [ADDR] default_value("127.0.0.1:6000") "Sets the IP address and the port of the P2P server")
     (@arg api_addr: --api [ADDR] default_value("127.0.0.1:7000") "Sets the IP address and the port of the API server")
     (@arg known_peer: -c --connect ... [PEER] "Sets the peers to connect to at start")
     (@arg p2p_workers: --("p2p-workers") [INT] default_value("4") "Sets the number of worker threads for P2P server")
    )
    .get_matches();

    // init logger
    let verbosity = matches.occurrences_of("verbose") as usize;
    stderrlog::new().verbosity(verbosity).init().unwrap();

    // parse p2p server address
    let p2p_addr = matches
        .value_of("peer_addr")
        .unwrap()
        .parse::<net::SocketAddr>()
        .unwrap_or_else(|e| {
            error!("Error parsing P2P server address: {}", e);
            process::exit(1);
        });

    // parse api server address
    let api_addr = matches
        .value_of("api_addr")
        .unwrap()
        .parse::<net::SocketAddr>()
        .unwrap_or_else(|e| {
            error!("Error parsing API server address: {}", e);
            process::exit(1);
        });

    // create channels between server and worker
    let (msg_tx, msg_rx) = channel::unbounded();

    // start the p2p server
    let (server_ctx, server) = server::new(p2p_addr, msg_tx).unwrap();
    server_ctx.start().unwrap();

    // start a new blockchain. Note that this chain contains genesis block
    let blockchain = Arc::new(Mutex::new(blockchain::Blockchain::new()));

    // start a new mempool
    let mempool = Arc::new(Mutex::new(transaction::Mempool::new()));

    // preparing 3 worker settings
    let mut initial_bytes: Vec<Vec<u8>> = Vec::new();
    let mut initial_pubkey_hashes: Vec<H256> = Vec::new();
    let mut initial_addresses: Vec<H160> = Vec::new();
    for i in 0..3 {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let pkcs8_vec = pkcs8_bytes.as_ref().to_vec();
        initial_bytes.push(pkcs8_vec);
        let key_pair = Ed25519KeyPair::from_pkcs8(initial_bytes[i].as_slice().into()).unwrap();
        initial_pubkey_hashes.push(H256::from(key_pair.public_key().as_ref()));
        initial_addresses.push(H160::from(initial_pubkey_hashes[i]));
    }

    // Initial state ICO and start a new statechain
    let statechain = Arc::new(Mutex::new(transaction::StateChain::new()));
    statechain.lock().unwrap().insert(blockchain.lock().unwrap().tip_hash, transaction::ico3_proc(initial_pubkey_hashes.clone()));
    
    // Dispatching processes with corresponding keys
    let self_keypair = if p2p_addr.port() % 1000 < initial_bytes.len() as u16{
        Ed25519KeyPair::from_pkcs8(initial_bytes[(p2p_addr.port() % 1000) as usize].as_slice().into()).unwrap()
    }else{
        key_pair::random()
    };

    // start the worker
    let p2p_workers = matches
        .value_of("p2p_workers")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing P2P workers: {}", e);
            process::exit(1);
        });
    let worker_ctx = worker::new(
        p2p_workers,
        msg_rx,
        &server,
        &blockchain,
        &mempool,
        &statechain,
    );
    worker_ctx.start();

    // start the miner
    let (miner_ctx, miner) = miner::new(
        &server,
        &blockchain,
        &mempool,
        &statechain,
    );
    miner_ctx.start();

    // start the generator
    let (generator_ctx, generator) = generator::new(
        &server,
        &blockchain,
        &mempool,
        &statechain,
        self_keypair,
        initial_addresses
    );
    generator_ctx.start();

    // connect to known peers
    if let Some(known_peers) = matches.values_of("known_peer") {
        let known_peers: Vec<String> = known_peers.map(|x| x.to_owned()).collect();
        let server = server.clone();
        thread::spawn(move || {
            for peer in known_peers {
                loop {
                    let addr = match peer.parse::<net::SocketAddr>() {
                        Ok(x) => x,
                        Err(e) => {
                            error!("Error parsing peer address {}: {}", &peer, e);
                            break;
                        }
                    };
                    match server.connect(addr) {
                        Ok(_) => {
                            info!("Connected to outgoing peer {}", &addr);
                            break;
                        }
                        Err(e) => {
                            error!(
                                "Error connecting to peer {}, retrying in one second: {}",
                                addr, e
                            );
                            thread::sleep(time::Duration::from_millis(1000));
                            continue;
                        }
                    }
                }
            }
        });
    }


    // start the API server
    ApiServer::start(
        api_addr,
        &miner,
        &generator,
        &server,
    );

    loop {
        std::thread::park();
    }
}
