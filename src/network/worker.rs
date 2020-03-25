use super::message::Message;
use super::peer;
use crate::network::server::Handle as ServerHandle;
use crossbeam::channel;
use log::{debug, warn};
use std::time::SystemTime;

use std::thread;
use std::sync::{Arc, Mutex};
use crate::crypto::hash::{H256, H160, Hashable};
use crate::blockchain::Blockchain;
use crate::block::Block;
use crate::transaction::{self, Mempool, State, StateChain};

#[derive(Clone)]
pub struct Context {
    msg_chan: channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Mempool>>,
    statechain: Arc<Mutex<StateChain>>,
}

pub fn new(
    num_worker: usize,
    msg_src: channel::Receiver<(Vec<u8>, peer::Handle)>,
    server: &ServerHandle,
    blockchain: &Arc<Mutex<Blockchain>>,
    mempool: &Arc<Mutex<Mempool>>,
    statechain: &Arc<Mutex<StateChain>>
) -> Context {
    Context {
        msg_chan: msg_src,
        num_worker,
        server: server.clone(),
        blockchain: Arc::clone(blockchain),
        mempool: Arc::clone(mempool),
        statechain: Arc::clone(statechain),
    }
}


impl Context {
    pub fn start(self) {
        let num_worker = self.num_worker;
        for i in 0..num_worker {
            let cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
                warn!("Worker thread {} exited", i);
            });
        }
    }

    fn worker_loop(&self) {
        let mut orphan_buffer: Vec<Block> = Vec::new();
        let mut delay_list: Vec<u128> = Vec::new();
        loop {
            // println!("{}", self.blockchain.lock().unwrap().tip_hash);
            let msg = self.msg_chan.recv().unwrap();
            let (msg, peer) = msg;
            let msg: Message = bincode::deserialize(&msg).unwrap();
            match msg {
                Message::Ping(nonce) => {
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }
                Message::Pong(nonce) => {
                    debug!("Pong: {}", nonce);
                }
                Message::NewBlockHashes(vec_hashes) => {
                    debug!("NewBlockHashes: {}", vec_hashes[0]);
                    let blockchain = self.blockchain.lock().unwrap();
                    let mut ret_hashes = Vec::new();
                    for blockhash in vec_hashes{
                        if blockchain.data.contains_key(&blockhash){
                            continue;
                        }
                        ret_hashes.push(blockhash);
                    }
                    if ret_hashes.len() > 0 {
                        peer.write(Message::GetBlocks(ret_hashes));
                    }
                }
                Message::GetBlocks(vec_hashes) => {
                    debug!("GetBlocks: {}", vec_hashes[0]);
                    let blockchain = self.blockchain.lock().unwrap();
                    let mut ret_blocks = Vec::new();
                    for blockhash in vec_hashes{
                        if blockchain.data.contains_key(&blockhash) == false{
                            continue;
                        }
                        ret_blocks.push(blockchain.data.get(&blockhash).unwrap().block_content.clone());
                    }
                    if ret_blocks.len() > 0 {
                        peer.write(Message::Blocks(ret_blocks));
                    }
                }
                Message::Blocks(vec_blocks) => {
                    debug!("Blocks: {}", "place_holder");
                    let mut blockchain = self.blockchain.lock().unwrap();
                    let mut inv_hashes = Vec::new();
                    let mut new_block_hashes = Vec::new();
                    for block in vec_blocks.clone() {
                        new_block_hashes.push(block.clone().hash());
                        self.server.broadcast(Message::NewBlockHashes(new_block_hashes.clone()));
                    }
                    for block in vec_blocks {
                        if blockchain.data.contains_key(&block.hash()){
                            continue;
                        }
                        if blockchain.data.contains_key(&block.header.parent){
                            if block.hash() <= block.header.difficulty && block.header.difficulty == 
                                blockchain.data[&block.header.parent].block_content.header.difficulty{
                                // before insert new block, first update corresponding state and statechain
                                let mut statechain = self.statechain.lock().unwrap();
                                let mut parent_state = State{data: statechain.data.get(&block.header.parent).unwrap().clone()};
                                parent_state.update(block.clone().content.content);
                                statechain.insert(block.hash(), parent_state);
                                // now insert the received block into the blockchain
                                blockchain.insert(&block);
                                let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
				                delay_list.push(now - block.header.timestamp);
        			            println!("Delays are {:?}", delay_list);
                                let block_serialized: Vec<u8> = bincode::serialize(&block).unwrap();
        			            println!("Block size is {}", block_serialized.len());
                            }
                            let mut new_block_list: Vec<Block> = Vec::new();
	                        new_block_list.push(block.clone());
                            while !new_block_list.is_empty(){
	                            let mut new_block_list_future: Vec<Block> = Vec::new();
                                let mut counter:usize = 0;
	                            for orphan_block in orphan_buffer.clone(){
	                                for new_block in new_block_list.clone(){
	                                    if orphan_block.header.parent == new_block.hash() {
                                            // before insert new block, first update corresponding state and statechain
                                            let mut statechain = self.statechain.lock().unwrap();
                                            let mut parent_state = State{data: statechain.data.get(&orphan_block.header.parent).unwrap().clone()};
                                            parent_state.update(orphan_block.clone().content.content);
                                            statechain.insert(orphan_block.hash(), parent_state);
                                            // now insert the received block into the blockchain
	                                        blockchain.insert(&orphan_block);
                                            let block_serialized: Vec<u8> = bincode::serialize(&block).unwrap();
        			                        println!("Block size is {}", block_serialized.len());
                                            let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
				                            delay_list.push(now-block.header.timestamp);
	                                        println!("Delays are {:?}", delay_list);
						                    new_block_list_future.push(orphan_block);
                                            orphan_buffer.remove(counter);
                                            counter = counter - 1;
                                            break;
	                                    }
	                                }
                                    counter = counter + 1;
	                            }
	                            new_block_list = new_block_list_future;
                            }
                        }
                        else {
                            orphan_buffer.push(block.clone());
                        }
                        if blockchain.tip_hash == Hashable::hash(&block){
                            inv_hashes.push(blockchain.tip_hash);
                        }
                    }
                    if inv_hashes.len() > 0 {
                        self.server.broadcast(Message::NewBlockHashes(inv_hashes));
                    }

                }
                Message::NewTransactionHashes(vec_hashes) => {
                    let mempool = self.mempool.lock().unwrap();
                    let mut ret_hashes = Vec::new();
                    for tx_hash in vec_hashes.clone(){
                        if mempool.data.contains_key(&tx_hash){
                            continue;
                        }
                        ret_hashes.push(tx_hash);
                    }
                    if ret_hashes.len() > 0 {
                        debug!("NewTransactionHashes: {}, Mempool Size {}", vec_hashes.clone()[0], mempool.total_size);
                        peer.write(Message::GetTransaction(ret_hashes));
                    }
                }
                Message::GetTransaction(vec_hashes) => {
                    let mempool = self.mempool.lock().unwrap();
                    let mut ret_txs = Vec::new();
                    for tx_hash in vec_hashes.clone(){
                        if mempool.data.contains_key(&tx_hash) == false{
                            continue;
                        }
                        ret_txs.push(mempool.data.get(&tx_hash).unwrap().clone());
                    }
                    if ret_txs.len() > 0 {
                        debug!("GetTransaction: {}, Mempool Size {}", vec_hashes.clone()[0], mempool.total_size);
                        peer.write(Message::Transactions(ret_txs));
                    }
                }
                Message::Transactions(vec_txs) => {
                    debug!("Transactions: {}", "place_holder");
                    let mut inv_hashes = Vec::new();
                    let state = self.statechain.lock().unwrap().data.get(& self.blockchain.lock().unwrap().tip_hash).unwrap().clone();
                    for tx in vec_txs {
                        let mut mempool = self.mempool.lock().unwrap();
                        if mempool.data.contains_key(&tx.hash()){
                            continue;
                        }
                        // check if the transaction is signed correctly
                        if transaction::verify(&tx.transaction, tx.clone().pub_key, 
                            tx.clone().signature) == false{
                            println!("Transaction sign verification failed");
                            continue;
                        }
                        // check if double spend error may occur
                        if state.contains_key(
                            &(tx.transaction.in_put[0].tx_hash, tx.transaction.in_put[0].index)) == false{
                            println!("Double spend checking failed");
                            continue;
                        }
                        // check if the input of tx is the person who make the tx
                        let (_value, addr) = state.get(
                            &(tx.transaction.in_put[0].tx_hash, tx.transaction.in_put[0].index)).unwrap().clone();
                        if ( addr == H160::from(H256::from(&tx.pub_key[..])) ) == false{
                            println!("Same sender input owner check failed");
                            continue;
                        }
                        inv_hashes.push(tx.hash());
                        mempool.insert(&tx);
                    }
                    if inv_hashes.len() > 0 {
                        self.server.broadcast(Message::NewTransactionHashes(inv_hashes));
                        debug!("After include new TX, Mempool size is {}", self.mempool.lock().unwrap().total_size);
                    }
                }
            }
        }
    }
}
