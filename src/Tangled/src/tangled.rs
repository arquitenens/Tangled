use std::any::TypeId;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::iter::Once;
use std::ptr::NonNull;
use std::thread::JoinHandle;
use crate::tangled_inner::TangledInner;
use config::config::{Config, ConfigInner};
use crate::commands::TangledCommands;
use crate::commands::IndexType;
use crossbeam_channel::unbounded;
use crossbeam_channel::{Sender, Receiver};
use core_types::borrow_state::BorrowState;
use core_types::indexing_mode::IndexingMode;
use core_types::inner_vec::InnerVecWrapper;
use crate::tangled_indexing::TangledIndex;
use crate::worker::Worker;

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
    //usize is the index in the pointer_vec
    borrow_state: UnsafeCell<HashMap<InnerVecWrapper<T>, (BorrowState, usize)>>,
    pub(crate) indexing: TangledIndex<T>,
    pub(crate) receiver: Receiver<TangledCommands<T>>,
    pub(crate) sender: TangledHandle<T>,

    pub(crate) thread_count: usize,
    global_config: Config<T>,
}


impl<T: 'static + std::marker::Send + std::clone::Clone + std::fmt::Debug> Default for Tangled<T> {
    fn default() -> Self {
        return Tangled::new(Config::default())
    }
}


impl<T: Send + 'static + Clone + std::fmt::Debug> Tangled<T> {
    pub fn new(config: Config<T>) -> Self{
        let (sender, receiver) = unbounded();
        return Self {
            cached: HashMap::new(),
            borrow_state: UnsafeCell::new(HashMap::new()),
            indexing: TangledIndex::new(IndexingMode::AppendHeavy),
            inners: Vec::new(),
            handles: Vec::new(),
            receiver,
            thread_count: 0,
            sender: TangledHandle{cmd_tx: sender},
            global_config: config
        }
    }

    ///each worker is a new os thread
    pub fn add_worker<F>(&'_ mut self, function: F) where F: FnOnce(Worker<T>) + Send + 'static {

        self.thread_count += 1;
        let inner = TangledInner::new(ConfigInner::default(), self.sender.cmd_tx.clone());
        self.inners.push(inner);
        let worker = Worker::new(self);
        let handle = std::thread::spawn(move || {
            function(worker);
        });
        self.handles.push(handle);

    }

    pub fn start(mut self) -> JoinHandle<()> {
        
        let receiver = self.receiver.clone();
        let handle = std::thread::spawn(move ||{
            loop {
                let receiver_result = receiver.clone().recv();
                let Ok(receiver) = receiver_result else { return; };
                    match receiver {
                        TangledCommands::Get { index: index, reply, request_requirements} => {
                            let some_value = None;

                            reply.send(some_value).expect("TODO: panic message");
                        },
                        TangledCommands::Push{value, request_requirements} => {
                            let last_index = self.indexing.last_index;
                            let (rough, _) = self.indexing.flat_to_indextype(last_index);
                            if let IndexType::Rough(rough) = rough{
                                let inner_vec = &mut self.inners[rough];
                                self.indexing.last_index += 1;
                                inner_vec.total_elements += 1;
                                inner_vec.data.push(value);
                            }
                            

                        }
                        TangledCommands::PushVec {value, request_requirements} => {
                            let last_index = self.indexing.last_index;
                            let (rough, _) = self.indexing.flat_to_indextype(last_index);
                            if let IndexType::Rough(rough) = rough{
                                let inner_vec = &mut self.inners[rough];
                                self.indexing.last_index += value.len();
                                inner_vec.total_elements += value.len();
                                inner_vec.data.extend(value);
                            }
                        }

                        TangledCommands::RawIndex(rough, direct) => {
                            todo!()
                        }
                        TangledCommands::Insert{index, value, request_requirements} => {
                            todo!()
                        },
                        TangledCommands::Drop(index) => {
                            todo!()
                        }
                        TangledCommands::GetVec(index) => {
                            todo!()
                        },
                        TangledCommands::Sync => {
                            todo!()
                        },
                        TangledCommands::InsertVec(_) => todo!(),
                        TangledCommands::PrintData => {
                            let data = &self.inners;
                            println!("data: {:#?}", data);
                        }
                    }
            }
        });
        return handle;
    }
    pub fn stop(&mut self, join_handle: JoinHandle<()>){
        return match join_handle.join(){
            Ok(()) => (),
            Err(e) => {
                panic!("Tangled: failed to join thread: {:?}", e);
            },
        }
    }
}



