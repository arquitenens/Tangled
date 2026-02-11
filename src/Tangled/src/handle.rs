use core_types::borrow_state::BorrowState;
use crate::tangled::Tangled;

pub struct RefHandle<'parent, 'value, T> where 'parent: 'value {
    parent: &'parent Tangled<T>,
    borrow_state: BorrowState,
    value: &'value T,
}

impl<'parent, 'value, T> RefHandle<'parent, 'value, T> {
    pub(crate) fn new(parent: &'parent Tangled<T>, borrow_state: BorrowState, value: &'value T) -> Self {
        Self{parent, borrow_state, value}
    }
}
