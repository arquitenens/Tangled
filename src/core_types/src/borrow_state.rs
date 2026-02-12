#[derive(Debug)]
pub enum BorrowState{
    Exclusive,
    Shared(usize),
    Dropped,
}

//Todo replace enum with isize if performance is needed, 0 = unshared, n = shared, -1 = exclusive
