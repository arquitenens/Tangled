use std::cell::UnsafeCell;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
type InnerVec<T> = ManuallyDrop<NonNull<UnsafeCell<Box<Vec<T>>>>>;
#[derive(Debug)]
pub struct InnerVecWrapper<T> {
    inner: InnerVec<T>,
}
unsafe impl<T> Sync for InnerVecWrapper<T>{
    //time for fun!
}
