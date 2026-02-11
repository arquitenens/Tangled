#[derive(Debug)]
///AppendHeavy uses a prefix vec and is thus O(1) append but O(n) delete and insert
///InsertionHeavy uses a rope algorithm and is O(log n) for append, insert and remove, thus, more balanced
pub enum IndexingMode{
    AppendHeavy, //prefix vec
    InsertionHeavy, //rope algorithm
}