use std::marker::PhantomData;
use core_types::indexing_mode::*;
pub use crate::config_inner::ConfigInner;
use core_types::request_loop::*;
#[derive(Debug)]
pub struct Config<T>{
    loop_type: WakeType,
    thread_count: usize, //sender threads
    inner: ConfigInner<T>
}


impl<T> Default for Config<T>{
    fn default() -> Self{
        return Self{
            loop_type: WakeType::default(),
            thread_count: 2,
            inner: ConfigInner::default(),
        }
    }
}
impl<T> Config<T>{
    pub fn new(thread_count: usize, flavor: IndexingMode, loop_type: WakeType) -> Self{
        return Self{
            loop_type,
            thread_count,
            inner: ConfigInner::new(flavor),
        }
    }
}