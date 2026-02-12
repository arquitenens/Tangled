use core_types::inner_vec::InnerVecWrapper;

#[derive(Debug)]
pub(crate) struct TangledIndex<T>{
    //list of stored pointers
    pub(crate) pointers: Vec<Option<InnerVecWrapper<T>>>,

    //keeps the size of each added vector, needed for log(n) random indexing via binary search
    pub(crate) prefix_vec: Vec<usize>,

    pub(crate) last_index: usize, //last index
}
impl<T> TangledIndex<T> {
    pub(crate) fn new() -> Self {
        Self{
            pointers: Vec::new(),
            prefix_vec: Vec::new(),
            last_index: 0,
        }
    }
}
