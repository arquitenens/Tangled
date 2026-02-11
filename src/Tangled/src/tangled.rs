use std::any::TypeId;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::iter::Once;
use std::ptr::NonNull;
use std::thread::JoinHandle;
use crate::tangled_inner::TangledInner;
use config::config::{Config, ConfigInner};
use crate::borrow::{BorrowedTangled, MutBorrowedTangled};
use crate::commands::TangledCommands;
use crate::commands::IndexType;
use crossbeam_channel::unbounded;
use crossbeam_channel::{Sender, Receiver};
use core_types::borrow_state::BorrowState;
use core_types::inner_vec::InnerVecWrapper;
use crate::tangled_indexing::TangledIndex;

#[derive(Debug)]
pub(crate) struct TangledHandle<T>{
    pub(crate) cmd_tx: Sender<TangledCommands<T>>,
}
#[derive(Debug)]
pub struct Tangled<T>{
    pub inners: Vec<TangledInner<T>>,
    handles: Vec<JoinHandle<()>>,
    //cached vectors
    cached: HashMap<usize, Vec<T>>,

    //check the borrow state of a pointer
    borrow_state: UnsafeCell<HashMap<InnerVecWrapper<T>, (BorrowState, usize)>>, //usize is the index in the pointer_vec
    indexing: TangledIndex<T>,
    pub(crate) receiver: Receiver<TangledCommands<T>>,
    pub(crate) sender: TangledHandle<T>,
    global_config: Config<T>,
}


impl<T: 'static> Default for Tangled<T> {
    fn default() -> Self {
        return Tangled::new(Config::default())
    }
}


impl<T: 'static> Tangled<T> {
    pub fn new(config: Config<T>) -> Self{
        let (sender, receiver) = unbounded();
        return Self {
            cached: HashMap::new(),
            borrow_state: UnsafeCell::new(HashMap::new()),
            indexing: TangledIndex::new(),
            inners: Vec::new(),
            handles: Vec::new(),
            receiver,
            sender: TangledHandle{cmd_tx: sender},
            global_config: config
        }
    }

    pub fn add_child(&mut self) {
            let inner = TangledInner::new(ConfigInner::default(), self.sender.cmd_tx.clone());
            self.inners.push(inner);
    }

    pub fn start(&mut self) -> JoinHandle<()> {
        
        let receiver = self.receiver.clone();
        let handle = std::thread::spawn(move ||{
            loop {
                let receiver_result = receiver.recv();
                let Ok(receiver) = receiver_result else { return; };
                    match receiver {
                        TangledCommands::Get { index: index, reply: reply } => {
                            reply.send(None).expect("TODO: panic message");
                        },
                        TangledCommands::RawIndex(rough, direct) => {
                            todo!()
                        }
                        TangledCommands::Write(index, value) => {
                            todo!()
                        },
                        TangledCommands::Drop(index) => {
                            todo!()
                        }
                        TangledCommands::GetVec(index) => {
                            todo!()
                        },
                    }
            }
        });
        return handle;
    }
}



