use crate::network::server::Handle as ServerHandle;

use log::info;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time;
use std::time::SystemTime;

use std::thread;
use std::sync::{Arc, Mutex};

use crate::transaction::{self, Transaction, SignedTransaction, Mempool,Input, Output, StateChain, State};
use ring::signature::{Ed25519KeyPair, KeyPair};
use crate::crypto::hash::{H256, H160, Hashable};
use crate::blockchain::Blockchain;
use crate::network::message::Message;
use rand::seq::SliceRandom; 


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
    keypair: Ed25519KeyPair,
    addresses: Vec<H160>,
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
    keypair: Ed25519KeyPair,
    addresses: Vec<H160>
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        server: server.clone(),
        blockchain: blockchain.clone(),
        mempool: Arc::clone(mempool),
        statechain: Arc::clone(statechain),
        keypair: keypair,
        addresses: addresses,
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
            .name("generator".to_string())
            .spawn(move || {
                self.generator_loop();
            })
            .unwrap();
        info!("Generator initialized into paused mode");
    }

    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Exit => {
                info!("Generator shutting down");
                self.operating_state = OperatingState::ShutDown;
            }
            ControlSignal::Start(i) => {
                info!("Generator starting in continuous mode with lambda {}", i);
                self.operating_state = OperatingState::Run(i);
            }
        }
    }

    fn generator_loop(&mut self) {
        // main mining loop
        let loop_begin = SystemTime::now();

        // Define self address and addresses of other peers in the network
        let self_address = H160::from(H256::from(self.keypair.public_key().as_ref()));
        let index = self.addresses.iter().position(|x| *x == self_address).unwrap();
        let mut other_address = self.addresses.clone();
        other_address.remove(index);

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
                    Err(TryRecvError::Disconnected) => panic!("Generator control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }
            

            // generate several transaction over time
            let current_tip_hash = self.blockchain.lock().unwrap().tip_hash;
            let current_state = State{data: self.statechain.lock().unwrap().data.get(&current_tip_hash).unwrap().clone()};
            let mut self_coins: Vec<(H256, usize, u64)> = Vec::new();
            for (k, v) in current_state.data{
                if v.1 != self_address{
                    continue;
                }
                self_coins.push((k.0, k.1, v.0));
            }
            // select a random address to send a random coin without more value than the coin
            let recipient = other_address.choose(&mut rand::thread_rng()).unwrap().clone();
            let input_coin = self_coins.choose(&mut rand::thread_rng()).unwrap().clone();
            let input: Vec<Input> = vec![Input{tx_hash: input_coin.0, index: input_coin.1, coin_base: false}];
            let output: Vec<Output> = vec![Output{address: recipient, value: input_coin.2 /2}, 
                            Output{address: self_address, value: input_coin.2 - input_coin.2 /2}];
            let t = Transaction{in_put: input, out_put: output};
            let signed_t = SignedTransaction{transaction: t.clone(), signature: transaction::sign(&t, &self.keypair),
                                                            pub_key: self.keypair.public_key().as_ref().to_vec()};
            
            // TODO: actual transaction generation

            let mut mempool = self.mempool.lock().unwrap();
            mempool.insert(&signed_t);
            std::mem::drop(mempool);
            self.server.broadcast(Message::NewTransactionHashes(vec![signed_t.hash()]));

            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
            }

            let loop_duration = SystemTime::now().duration_since(loop_begin).unwrap().as_secs();
            if loop_duration > 50{
                println!("Generating transactions finished");
                break;
            }
        }

        let mut loop_duration = SystemTime::now().duration_since(loop_begin).unwrap().as_secs();
        while loop_duration < 60 {
            loop_duration = SystemTime::now().duration_since(loop_begin).unwrap().as_secs();
            continue;
        }
    }
}
