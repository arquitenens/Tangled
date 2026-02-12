use std::sync::mpsc;
use crossbeam_channel::{unbounded, Receiver, Sender};
use crate::tangled::Tangled;
use crate::commands::{IndexType, RequestRequirements, TangledCommands};
use crate::handle::RefHandle;
use crate::tangled::TangledHandle;
use crate::tangled_inner::TangledInner;
use crate::worker::Worker;

fn handle_reply<V>(rx: Receiver<Option<V>>) -> Option<V> {
    let reply = rx.recv().unwrap_or_else(|_| None);
    return reply
}

pub struct BorrowedWorker<'outer, T> {
    inner: &'outer Worker<T>,
}
impl<'outer, T> BorrowedWorker<'outer, T> {
    pub(crate) fn new(inner: &'outer Worker<T>) -> Self {
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
            reply: tx,
            request_requirements: RequestRequirements::None,
        };
        sender.send(command).expect("failed to send message");
        let reply = handle_reply(rx);
        return reply
    }
}
pub struct MutBorrowedWorker<'outer, T>{
    inner: &'outer mut Worker<T>,
}

impl<'outer, T> MutBorrowedWorker<'outer, T> {
    pub(crate) fn new(inner: &'outer mut Worker<T>) -> Self {
        Self { inner }
    }

    pub(crate) fn get_sender(&self) -> Sender<TangledCommands<T>>{
        self.inner.parent_receiver.clone()
    }
    pub fn push(&self, value: T) {
        let sender = self.get_sender();
        let (tx, rx) = unbounded::<Option<T>>();
        let parent = self.inner.get_methods();
        let command = TangledCommands::Push {
            value,
            request_requirements: RequestRequirements::None
        };
        sender.send(command).expect("failed to send message");
        let _ = handle_reply(rx);

    }
}