#[derive(Debug)]
pub struct File {
    id: u32,
    contents: Vec<i32>,
}

impl File {
    pub fn new(id: u32, contents: Vec<i32>) -> File {
        File { id, contents }
    }
}
