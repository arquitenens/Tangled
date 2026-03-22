use std::any::TypeId;
use std::cell::UnsafeCell;
use std::collections::{BTreeMap, HashMap};
use std::iter::Once;
use std::ops::{Deref, DerefMut};
use std::process::id;
use std::ptr::NonNull;
use std::thread::JoinHandle;
use std::time::Instant;
use crate::tangled_inner::TangledInner;
use config::config::{Config, ConfigInner};
use crate::commands::{ReqOrder, TangledCommands};
use crate::commands::IndexType;
use crossbeam_channel::unbounded;
use crossbeam_channel::{Sender, Receiver};
use core_types::borrow_state::BorrowState;
use core_types::indexing_mode::IndexingMode;
use core_types::inner_vec::InnerVecWrapper;
use crate::tangled_indexing::TangledIndex;
use crate::worker::Worker;

#[derive(Debug)]
#[repr(transparent)]
pub(crate) struct TangledHandle<T>{
    pub(crate) cmd_tx: Sender<TangledCommands<T>>,
}
pub struct RealTangled<T>{
    pub inners: Vec<TangledInner<T>>,
    handles: Vec<JoinHandle<()>>,
    //cached vectors
    cached: HashMap<usize, Vec<T>>,


    tasks: BTreeMap<Instant, Task<T>>,
    //check the borrow state of a pointer
    //usize is the index in the pointer_vec
    borrow_state: UnsafeCell<HashMap<InnerVecWrapper<T>, (BorrowState, usize)>>,
    pub(crate) indexing: TangledIndex<T>,
    pub(crate) receiver: Receiver<TangledCommands<T>>,
    pub(crate) sender: TangledHandle<T>,

    pub(crate) thread_count: usize,
    global_config: Config<T>,
}

pub struct Tangled<T>{
    inner: UnsafeCell<RealTangled<T>>,
}

impl<T> Deref for Tangled<T> {
    type Target = RealTangled<T>;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.get() }
    }
}

impl<T> DerefMut for Tangled<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner.get() }
    }
}

impl<T: 'static + std::marker::Send + std::clone::Clone + std::fmt::Debug> Default for Tangled<T> {
    fn default() -> Self {
        return Tangled::new(Config::default())
    }
}


struct InnerTask<T>(Option<Box<dyn FnOnce(&mut Tangled<T>)>>);

impl<T> Default for InnerTask<T>{
    fn default() -> Self {
        return InnerTask(None);
    }
}
unsafe impl<T> Send for InnerTask<T> {}
enum Task<T>{
    Task(InnerTask<T>),
    Resolved
}


impl<T: Send + 'static + Clone + std::fmt::Debug> Tangled<T> {
    pub fn new(config: Config<T>) -> Self{
        let (sender, receiver) = unbounded();
        return Self {
            inner: UnsafeCell::new(
                RealTangled{
                    cached: HashMap::new(),
                    borrow_state: UnsafeCell::new(HashMap::new()),
                    indexing: TangledIndex::new(IndexingMode::AppendHeavy),
                    inners: Vec::new(),
                    handles: Vec::new(),
                    tasks: BTreeMap::new(),
                    receiver,
                    thread_count: 0,
                    sender: TangledHandle{cmd_tx: sender},
                    global_config: config
                }
            )
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
                        TangledCommands::Get { index, reply,
                            request_requirements, order} => {

                            let task = move |i_self: &mut Tangled<T>| {
                                let index = if let IndexType::Direct(i) = index {i} else {
                                    unreachable!()
                                };
                                let idx = i_self.indexing.flat_to_indextype(index);
                                let (IndexType::Rough(rough), IndexType::Direct(direct)) = idx else {
                                    unreachable!()
                                };

                                let value = i_self.inners[rough].data[direct].clone();
                                match reply.send(Some(value)){
                                    Ok(_) => {},
                                    Err(SendError) => {}
                                };
                            };

                            let ord = match order {
                                ReqOrder::Strict(x) => x,
                                ReqOrder::Relaxed => Instant::now()
                            };
                            self.tasks.insert(ord,
                                                  Task::Task(InnerTask(Some(Box::new(task)))));
                        },
                        TangledCommands::Push{value, request_requirements, order } => {
                            let task = |i_self: &mut Tangled<T>| {

                                let last_index = i_self.indexing.last_index;
                                let (rough, _) = i_self.indexing.flat_to_indextype(last_index);
                                if let IndexType::Rough(rough) = rough{
                                    i_self.indexing.last_index += 1;
                                    let inner_vec = &mut i_self.inners[rough];
                                    inner_vec.total_elements += 1;
                                    inner_vec.data.push(value);
                                }
                            };
                            //println!("order: {:?}", order);

                            let ord = match order {
                                ReqOrder::Strict(x) => x,
                                ReqOrder::Relaxed => Instant::now()
                            };

                            self.tasks.insert(ord,
                                                  Task::Task(InnerTask(Some(Box::new(task)))));
                        }
                        TangledCommands::PushVec {value, request_requirements, order} => {
                            let task = |i_self: &mut Tangled<T>| {
                                let last_index = i_self.indexing.last_index;
                                let (rough, _) = i_self.indexing.flat_to_indextype(last_index);
                                if let IndexType::Rough(rough) = rough{
                                    i_self.indexing.last_index += value.len();
                                    let inner_vec = &mut i_self.inners[rough];
                                    inner_vec.total_elements += value.len();
                                    inner_vec.data.extend(value);
                                }
                            };
                            let ord = match order {
                                ReqOrder::Strict(x) => x,
                                ReqOrder::Relaxed => Instant::now()
                            };

                            self.tasks.insert(ord,
                                                  Task::Task(InnerTask(Some(Box::new(task)))));
                        }

                        TangledCommands::RawIndex(rough, direct) => {
                            todo!()
                        }
                        TangledCommands::Insert{index, value,
                            request_requirements, order} => {
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
                            self.tasks.insert(Instant::now(),
                                                  Task::Resolved);
                        }
                    };
                self.process_task();
                self.tasks.clear()
            }
        });
        return handle;
    }

    pub fn process_task(&mut self) {
        let task_buffer = unsafe {&mut *std::ptr::from_mut(&mut self.tasks)};
        for (_,task) in task_buffer.iter_mut(){
            match task {
                Task::Task(inner) => {
                    if let Some(F) = std::mem::take(&mut inner.0){

                        F(self)
                    }

                }
                Task::Resolved => {}
            }
        }
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



