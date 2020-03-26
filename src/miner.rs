use crate::network::server::Handle as ServerHandle;

use log::info;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time;
use std::time::SystemTime;

use std::thread;
use std::sync::{Arc, Mutex};
use rand::{thread_rng, Rng};

use crate::blockchain::Blockchain;
use crate::block::{Block, Header, Content};
use crate::transaction::{Mempool, State, StateChain};
use crate::crypto::merkle::MerkleTree;
use crate::crypto::hash::{Hashable, H256, H160};
use crate::network::message::Message;


enum ControlSignal {
    Start(u64), // the number controls the lambda of interval between block generation
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Mempool>>,
    statechain: Arc<Mutex<StateChain>>,
    self_address: H160
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}



pub fn new(
    server: &ServerHandle, 
    blockchain: &Arc<Mutex<Blockchain>>,
    mempool: &Arc<Mutex<Mempool>>,
    statechain: &Arc<Mutex<StateChain>>,
    self_address: H160
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        server: server.clone(),
        blockchain: Arc::clone(blockchain),
        mempool: Arc::clone(mempool),
        statechain: Arc::clone(statechain),
        self_address: self_address,
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle)
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, lambda: u64) {
        self.control_chan
            .send(ControlSignal::Start(lambda))
            .unwrap();
    }

}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("miner".to_string())
            .spawn(move || {
                self.miner_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Exit => {
                info!("Miner shutting down");
                self.operating_state = OperatingState::ShutDown;
            }
            ControlSignal::Start(i) => {
                info!("Miner starting in continuous mode with lambda {}", i);
                self.operating_state = OperatingState::Run(i);
            }
        }
    }

    fn miner_loop(&mut self) {
        // main mining loop
        
        let loop_begin = SystemTime::now();
        let mut block_mined = 0;
        let tx_block: usize = 10; 

        loop {
            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    self.handle_control_signal(signal);
                    continue;
                }
                OperatingState::ShutDown => {
                    return;
                }
                _ => match self.control_chan.try_recv() {
                    Ok(signal) => {
                        self.handle_control_signal(signal);
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Miner control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }
            
            let loop_duration = SystemTime::now().duration_since(loop_begin).unwrap().as_secs();
            if loop_duration > 80{
                info!("Blocks minded is {}", block_mined);
                break;
            }

            // TODO: actual mining

            let blockchain = self.blockchain.lock().unwrap();
            let parent = blockchain.tip();
            let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
            let difficulty = blockchain.data.get(&parent).unwrap().block_content.header.difficulty;

            let current_tip_hash = blockchain.tip_hash;
            std::mem::drop(blockchain);
            let mut state = State{ data: self.statechain.lock().unwrap().data.get(& current_tip_hash).unwrap().clone()};

            // Adding real transaction implementations
            let mut mempool = self.mempool.lock().unwrap();
            let tx_vec = mempool.retrieve_vec(tx_block);
            // retrieve transactions until enough
            if tx_vec.len() == 0 {
                continue;
            }

            // state update and all the checks
            let mut state_copy = state.clone();
            let (accept_vec, _abort_vec) = state_copy.update(tx_vec);

            // cases when there are tx being aborted
            // if _abort_vec.len() > 0{
            //     mempool.insert_vec(_abort_vec);
            // }
            if accept_vec.len() == 0{
                continue;
            }
            state.update(accept_vec.clone());
            std::mem::drop(mempool);
            
            let merkle_tree = MerkleTree::new(& accept_vec);

            let mut rng = thread_rng();
            loop{
                let nonce = rng.gen();
                let header = Header{parent: parent, nonce: nonce, difficulty: difficulty, timestamp: timestamp, merkle_root: merkle_tree.root()};
                let content = Content{content: accept_vec.clone()};
                let block = Block{header: header, content: content};
                if Hashable::hash(&block) <= difficulty{
                    let mut blockchain = self.blockchain.lock().unwrap();
                    let mut statechain = self.statechain.lock().unwrap();
                    statechain.insert(block.hash(), state);
                    blockchain.insert(&block);
                    // log info for receiving transaction value  
                    for signed_tx in block.clone().content.content{
                        for output in signed_tx.transaction.out_put{
                            if output.address != self.self_address{
                                continue;
                            }
                            info!("{} receives {} value from {}", self.self_address,
                                output.value, H160::from(H256::from(&signed_tx.pub_key[..])));
                        }
                    }
                    block_mined += 1;
                    self.server.broadcast(Message::NewBlockHashes(vec![Hashable::hash(&block)]));
                    break;
                } 
            }

            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
            }
        }

        let mut loop_duration = SystemTime::now().duration_since(loop_begin).unwrap().as_secs();
        while loop_duration < 90 {
            loop_duration = SystemTime::now().duration_since(loop_begin).unwrap().as_secs();
            continue;
        }
        let blockchain = self.blockchain.lock().unwrap();
        info!("BlockChain Length is {}", blockchain.total_size);
        info!("BlockChain height is {}", blockchain.tip_height);
    }
}
