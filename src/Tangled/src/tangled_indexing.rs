use std::collections::BTreeMap;
use std::ptr;
use config::config::ConfigInner;
use core_types::indexing_mode::IndexingMode;
use core_types::inner_vec::InnerVecWrapper;
use crate::commands::IndexType;
use crate::commands::IndexType::Rough;
use crate::tangled_inner::TangledInner;

#[derive(Debug)]
pub(crate) struct TangledIndex<T>{
    //list of stored pointers
    pub(crate) pointers: Vec<Option<InnerVecWrapper<T>>>,

    //keeps the size of each added vector, needed for log(n) random indexing via binary search
    pub(crate) prefix_vec: (Option<Vec<usize>>, Option<BTreeMap<usize, usize>>),

    pub(crate) last_index: usize, //last index
}

impl<T> TangledIndex<T> {
    pub(crate) fn new(indexing_mode: IndexingMode) -> Self {
        Self{
            pointers: Vec::new(),
            prefix_vec: match indexing_mode {
                IndexingMode::InsertionHeavy => (None, Some(BTreeMap::new())),
                IndexingMode::AppendHeavy => (Some(Vec::new()), None),
            },
            last_index: 0,
        }
    }
    pub(crate) fn get_config(parent_ptr: *const TangledInner<T>) -> *const ConfigInner<T>{
        let parent = unsafe {&*parent_ptr};
        return ptr::from_ref(&parent.per_config)
    }
    pub(crate) fn flat_to_indextype(&self, index: usize) -> (IndexType, IndexType){
        let insertion_heavy = &self.prefix_vec.1;
        let append_heavy= &self.prefix_vec.0;

        if let Some(append) = append_heavy {
            let target = index;
            return if !append.is_empty() {
                let rough = match append.binary_search(&target) {
                    Ok(index) => index,
                    Err(_) => panic!("Index wasnt found"),
                };
                let direct = if rough == 0 {
                    index
                } else {
                    index - self.prefix_vec.0.clone().unwrap()[rough - 1]
                };
                (IndexType::Rough(rough), IndexType::Direct(direct))
            } else {
                (IndexType::Rough(0), IndexType::Direct(index))
            }
        }else {
            todo!() //if Append heavy is none then it has to be insertion heavy;
        }

    }
}
