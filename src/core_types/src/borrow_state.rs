#[derive(Debug)]
pub enum BorrowState{
    Exclusive,
    Shared(usize),
    Dropped,
}
