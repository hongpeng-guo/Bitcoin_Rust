use crate::network::server::Handle as ServerHandle;

use log::info;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time;
use std::time::SystemTime;

use std::thread;
use std::sync::{Arc, Mutex};

use crate::transaction::{Transaction, SignedTransaction, Mempool,tests};
use crate::network::message::Message;
use crate::crypto::hash::Hashable;


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
    mempool: Arc<Mutex<Mempool>>,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}



pub fn new(
    server: &ServerHandle, 
    mempool: &Arc<Mutex<Mempool>>
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        server: server.clone(),
        mempool: Arc::clone(mempool),
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
                    Err(TryRecvError::Disconnected) => panic!("Generator control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }
            
            // TODO: actual transaction generation
            let mut mempool = self.mempool.lock().unwrap();
            let t = tests::generate_random_signedtransaction();
            mempool.insert(&t);
            std::mem::drop(mempool);
            self.server.broadcast(Message::NewTransactionHashes(vec![t.hash()]));

            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
            }

            let loop_duration = SystemTime::now().duration_since(loop_begin).unwrap().as_secs();
            if loop_duration > 200{
                println!("Generating transactions finished");
                break;
            }
        }

        let mut loop_duration = SystemTime::now().duration_since(loop_begin).unwrap().as_secs();
        while loop_duration < 210 {
            loop_duration = SystemTime::now().duration_since(loop_begin).unwrap().as_secs();
            continue;
        }
    }
}
