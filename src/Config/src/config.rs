use std::marker::PhantomData;
use core_types::indexing_mode::*;
pub use crate::config_inner::ConfigInner;
#[derive(Debug)]
pub struct Config<T>{
    thread_count: usize, //sender threads
    inner: ConfigInner<T>
}


impl<T> Default for Config<T>{
    fn default() -> Self{
        return Self{
            thread_count: 2,
            inner: ConfigInner::default(),
        }
    }
}
impl<T> Config<T>{
    pub fn new(thread_count: usize, flavor: IndexingMode) -> Self{
        return Self{
            thread_count,
            inner: ConfigInner::new(flavor),
        }
    }
}