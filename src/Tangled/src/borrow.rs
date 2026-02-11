use std::sync::mpsc;
use crossbeam_channel::{Receiver, Sender};
use crate::tangled::Tangled;
use crate::commands::{IndexType, TangledCommands};
use crate::handle::RefHandle;
use crate::tangled::TangledHandle;
use crate::tangled_inner::TangledInner;

pub struct BorrowedTangled<'b, T>{
    inner: &'b TangledInner<T>,
}
impl<'b, T> BorrowedTangled<'b, T> {
    pub(crate) fn new(inner: &'b TangledInner<T>) -> Self {
        Self { inner }
    }

    pub(crate) fn get_sender(&self) -> Sender<TangledCommands<T>>{
        unsafe {
            self.inner.parent.as_ref().sender.cmd_tx.clone()
        }
    }

    pub fn get(&self, index: usize) -> Receiver<Option<T>> {
        let sender = self.get_sender();
        println!("sender : {:?}", sender);
        todo!()
    }
}
pub struct MutBorrowedTangled<'b, T>{
    inner: &'b mut TangledInner<T>,
}

impl<'b, T> MutBorrowedTangled<'b, T> {
    pub(crate) fn new(inner: &'b mut TangledInner<T>) -> Self {
        Self { inner }
    }
}