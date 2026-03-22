use std::borrow::Borrow;
use std::ptr;
use crossbeam_channel::Sender;
use crate::tangled_inner::TangledInner;
use crate::tangled::Tangled;
use crate::tangled_indexing;
use crate::borrow::*;
use crate::commands::TangledCommands;
use crate::tangled_indexing::TangledIndex;
use crate::tangled::TangledHandle;

pub struct Worker<T>{
    inner: *const TangledIndex<T>,
    pub(crate) parent_receiver: Sender<TangledCommands<T>>
}
impl<T> Worker<T>{
    pub(crate) fn new(inner: &Tangled<T>) -> Self{
        let receiver = inner.sender.cmd_tx.clone();
        let parent = ptr::from_ref(&inner.indexing);
        Worker{inner: parent, parent_receiver: receiver}
    }
    pub fn borrow(&'_ self) -> BorrowedWorker<'_, T> {
        BorrowedWorker::new(self)
    }
    pub fn borrow_mut(&'_ mut self) -> MutBorrowedWorker<'_, T> {
        MutBorrowedWorker::new(self)
    }
    pub(crate) fn get_methods<'a>(&self) -> &'a TangledIndex<T>{
        unsafe { &*self.inner }
    }
}

unsafe impl<T> Send for Worker<T>{}