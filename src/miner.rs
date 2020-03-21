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
use crate::transaction::{Transaction, SignedTransaction, Mempool,tests};
use crate::crypto::merkle::MerkleTree;
use crate::crypto::hash::Hashable;
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
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}



pub fn new(
    server: &ServerHandle, 
    blockchain: &Arc<Mutex<Blockchain>>
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        server: server.clone(),
        blockchain: Arc::clone(blockchain),
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
            
            // TODO: actual mining
            let blockchain = self.blockchain.lock().unwrap();
            let parent = blockchain.tip();
            let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
            let difficulty = blockchain.data.get(&parent).unwrap().block_content.header.difficulty;
            std::mem::drop(blockchain);

            let mut default_transaction: Vec<Transaction> = Vec::new();
            let t = tests::generate_random_transaction();
            default_transaction.push(t);

            let merkle_tree = MerkleTree::new(& default_transaction);

            let mut rng = thread_rng();
            loop{
                let nonce = rng.gen();
                let header = Header{parent: parent, nonce: nonce, difficulty: difficulty, timestamp: timestamp, merkle_root: merkle_tree.root()};
                let content = Content{content: default_transaction.clone()};
                let block = Block{header: header, content: content};
                if Hashable::hash(&block) <= difficulty{
                    let mut blockchain = self.blockchain.lock().unwrap();
                    blockchain.insert(&block);
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

            let loop_duration = SystemTime::now().duration_since(loop_begin).unwrap().as_secs();
            if loop_duration > 100{
                println!("Blocks minded is {}", block_mined);
                break;
            }
        }

        let mut loop_duration = SystemTime::now().duration_since(loop_begin).unwrap().as_secs();
        while loop_duration < 110 {
            loop_duration = SystemTime::now().duration_since(loop_begin).unwrap().as_secs();
            continue;
        }
        let blockchain = self.blockchain.lock().unwrap();
        println!("BlockChain Length is {}", blockchain.total_size);
        println!("BlockChain height is {}", blockchain.tip_height);
    }
}
