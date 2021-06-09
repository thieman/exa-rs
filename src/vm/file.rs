pub struct File {
    id: u16,
    contents: Vec<i16>,
}

impl File {
    pub fn new(id: u16, contents: Vec<i16>) -> File {
        File { id, contents }
    }
}
