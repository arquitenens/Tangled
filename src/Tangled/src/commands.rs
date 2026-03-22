use crossbeam_channel::Sender;
use crate::tangled_inner::TangledInner;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub(crate) enum IndexType{
    Rough(usize),
    Direct(usize),
}



pub(crate) enum RequestRequirements{
    CalculateSelf,
    None,
    //todo CalculatePublicQueue maybe later
}

pub(crate) enum CalculateOptions{
    Index,
    Size,
    //some calculation options
}

#[derive(Debug)]
pub enum ReqOrder{
    Strict(Instant),
    Relaxed,
}

pub(crate) enum TangledCommands<T>{
    //direct index
    Get{
        request_requirements: RequestRequirements,
        index: IndexType,
        reply: Sender<Option<T>>,
        order: ReqOrder,
    },
    Insert{
        request_requirements: RequestRequirements,
        index: IndexType,
        order: ReqOrder,
        value: T,
    },

    //both
    //       rough,     direct
    RawIndex(IndexType, IndexType),

    //rough index
    GetVec(IndexType),
    InsertVec(IndexType),
    Drop(IndexType),

    //misc
    Sync,
    Push{
        value: T,
        request_requirements: RequestRequirements,
        order: ReqOrder,
        //reply: Sender<Option<T>>
    },
    PushVec{
        value: Vec<T>,
        order: ReqOrder,
        request_requirements: RequestRequirements
    },
    PrintData,
}

unsafe impl<T> Send for TangledCommands<T>{}