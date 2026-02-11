use std::any::TypeId;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::iter::Once;
use std::ptr::NonNull;
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
    //cached vectors
    cached: HashMap<usize, Vec<T>>,

    //check the borrow state of a pointer
    borrow_state: UnsafeCell<HashMap<InnerVecWrapper<T>, (BorrowState, usize)>>, //usize is the index in the pointer_vec
    indexing: TangledIndex<T>,
    pub(crate) receiver: Receiver<TangledCommands<T>>,
    pub(crate) sender: TangledHandle<T>,
    global_config: Config<T>,
}


impl<T> Default for Tangled<T> {
    fn default() -> Self {
        return Tangled::new(Config::default())
    }
}


impl<T> Tangled<T> {
    pub fn new(config: Config<T>) -> Self{
        let (sender, receiver) = unbounded();
        return Self {
            cached: HashMap::new(),
            borrow_state: UnsafeCell::new(HashMap::new()),
            indexing: TangledIndex::new(),
            inners: Vec::new(),
            receiver,
            sender: TangledHandle{cmd_tx: sender},
            global_config: config
        }
    }

    pub fn add_child(&mut self) {
            let mut inner = TangledInner::new(ConfigInner::default());
            inner.parent = NonNull::from_ref(&self);
            self.inners.push(inner);
    }

    pub fn handle_request(&mut self){
        let command = match self.receiver.recv(){
            Ok(command) => {
                command
            },
            Err(_) => return,
        };
        match command {
            TangledCommands::Get { index: index, reply: reply } => {
                println!("hi");
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
            }
        }
    }
}



