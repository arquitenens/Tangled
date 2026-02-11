use std::sync::mpsc;
use crossbeam_channel::{unbounded, Receiver, Sender};
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
        self.inner.parent_receiver.clone()
    }

    pub fn get(&self, index: usize) -> Option<T> {
        let sender = self.get_sender();
        let (tx, rx) = unbounded::<Option<T>>();
        let command = TangledCommands::Get {
            index: IndexType::Direct(index),
            reply: tx
        };
        sender.send(command).expect("failed to send message");
        let reply = match rx.recv(){
            Ok(reply) => reply,
            Err(_) => None
        };
        return reply

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