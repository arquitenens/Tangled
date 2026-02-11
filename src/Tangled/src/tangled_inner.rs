use std::any::TypeId;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::ptr::NonNull;
use std::sync::mpsc;
use crossbeam_channel::{unbounded, Receiver, Sender};
use config::config::ConfigInner;
use core_types::inner_vec::InnerVecWrapper;
use core_types::borrow_state::BorrowState;
use crate::borrow::{BorrowedTangled, MutBorrowedTangled};
use crate::commands::TangledCommands;
use crate::tangled::Tangled;
use crate::tangled::TangledHandle;


#[derive(Debug)]
pub struct TangledInner<T>{
    
    //internal data
    data: Vec<T>,

    total_elements: usize,

    pub(crate) sender: Sender<TangledCommands<T>>,
    pub(crate) parent_receiver: Sender<TangledCommands<T>>,
    pub(crate) receiver: Receiver<TangledCommands<T>>,

    per_config: ConfigInner<T>
}



impl<T> TangledInner<T>{
    pub(crate) fn new(per_config: ConfigInner<T>, send_to: Sender<TangledCommands<T>>) -> Self{
        let (sender, receiver) = unbounded();
        return Self{
            data: Vec::new(),
            sender,
            receiver,
            parent_receiver: send_to,
            total_elements: 0,
            per_config,
        };
    }

    pub fn borrow(&self) -> BorrowedTangled<'_, T>{
        BorrowedTangled::new(self)
    }

    pub fn borrow_mut(&mut self) -> MutBorrowedTangled<'_, T>{
        MutBorrowedTangled::new(self)
    }
}

unsafe impl<T> Send for TangledInner<T>{}