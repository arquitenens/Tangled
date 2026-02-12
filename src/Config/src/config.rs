use std::marker::PhantomData;
use core_types::indexing_mode::*;
pub use crate::config_inner::ConfigInner;
use core_types::request_loop::*;
#[derive(Debug)]
pub struct Config<T>{
    loop_type: WakeType,
    max_thread_count: usize,
    inner: ConfigInner<T>
}


impl<T> Default for Config<T>{
    fn default() -> Self{
        return Self{
            loop_type: WakeType::default(),
            max_thread_count: std::thread::available_parallelism().unwrap().get(),
            inner: ConfigInner::default(),
        }
    }
}
impl<T> Config<T>{
    pub fn new(thread_count: usize, flavor: IndexingMode, loop_type: WakeType) -> Self{
        return Self{
            loop_type,
            max_thread_count: std::thread::available_parallelism().unwrap().get(),
            inner: ConfigInner::new(flavor),
        }
    }
}