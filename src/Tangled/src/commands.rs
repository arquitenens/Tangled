use crossbeam_channel::Sender;
use crate::tangled_inner::TangledInner;

pub(crate) enum IndexType{
    Rough(usize),
    Direct(usize),
}


pub(crate) enum TangledCommands<T>{
    //direct index
    Get{
        index: IndexType,
        reply: Sender<Option<T>>
    },
    Write(IndexType, T),

    //both
    //       rough,     direct
    RawIndex(IndexType, IndexType),

    //rough index
    GetVec(IndexType),
    Drop(IndexType),
}

unsafe impl<T> Send for TangledCommands<T>{}