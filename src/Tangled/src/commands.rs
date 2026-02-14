use crossbeam_channel::Sender;
use crate::tangled_inner::TangledInner;

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

pub(crate) enum TangledCommands<T>{
    //direct index
    Get{
        request_requirements: RequestRequirements,
        index: IndexType,
        reply: Sender<Option<T>>
    },
    Insert{
        request_requirements: RequestRequirements,
        index: IndexType,
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
        //reply: Sender<Option<T>>
    },
    PushVec{
        value: Vec<T>,
        request_requirements: RequestRequirements
    },
    PrintData,
}

unsafe impl<T> Send for TangledCommands<T>{}