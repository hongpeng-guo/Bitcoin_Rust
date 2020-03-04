use super::message::Message;
use super::peer;
use crate::network::server::Handle as ServerHandle;
use crossbeam::channel;
use log::{debug, warn};

use std::thread;
use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use crate::block::Block;
use crate::crypto::hash::Hashable;

#[derive(Clone)]
pub struct Context {
    msg_chan: channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
}

pub fn new(
    num_worker: usize,
    msg_src: channel::Receiver<(Vec<u8>, peer::Handle)>,
    server: &ServerHandle,
    blockchain: &Arc<Mutex<Blockchain>>
) -> Context {
    Context {
        msg_chan: msg_src,
        num_worker,
        server: server.clone(),
        blockchain: Arc::clone(blockchain),
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
                    for block in vec_blocks {
                        ////Zhijian's writing something here
                        if blockchain.data.contains_key(&block.header.parent){
                            if block.hash() <= block.header.difficulty && block.header.difficulty == blockchain.data[&block.header.parent].block_content.header.difficulty{
                                blockchain.insert(&block);
                            }
                            let mut new_block_list: Vec<Block> = Vec::new();
	                    new_block_list.push(block.clone());
                            while(!new_block_list.is_empty()){
	                            let mut new_block_list_future: Vec<Block> = Vec::new();
                                    let mut counter:usize = 0;
	                            for orphan_block in orphan_buffer.clone(){
	                                for new_block in new_block_list.clone(){
	                                    if orphan_block.header.parent == new_block.hash() {
	                                         blockchain.insert(&orphan_block);
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

                        ////////////
                        
                        if blockchain.tip_hash == Hashable::hash(&block){
                            inv_hashes.push(blockchain.tip_hash);
                        }
                    }
                    if inv_hashes.len() > 0 {
                        self.server.broadcast(Message::NewBlockHashes(inv_hashes));
                    }

                }
            }
        }
    }
}
