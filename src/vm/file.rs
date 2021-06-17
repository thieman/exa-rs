#[derive(Debug, PartialEq, Eq)]
pub struct File {
    pub id: i32,
    pub contents: Vec<i32>,
}

impl File {
    pub fn new(id: i32, contents: Vec<i32>) -> File {
        File { id, contents }
    }
}
