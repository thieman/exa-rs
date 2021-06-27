use std::fmt;

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

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "File {} (len: {}) {:?}",
            self.id,
            self.contents.len(),
            self.contents,
        )
    }
}
