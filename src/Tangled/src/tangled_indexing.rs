use core_types::inner_vec::InnerVecWrapper;

#[derive(Debug)]
pub(crate) struct TangledIndex<T>{
    //list of stored pointers
    pointers: Vec<Option<InnerVecWrapper<T>>>,

    //keeps the size of each added vector, needed for log(n) random indexing via binary search
    prefix_vec: Vec<usize>,

    last_index: usize, //last index
}
impl<T> TangledIndex<T> {
    pub(crate) fn new() -> Self {
        todo!()
    }
}