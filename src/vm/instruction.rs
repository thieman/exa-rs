#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Link(Target),
    Copy(Target, Target),
    VoidM(),
    NotFound(),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Target {
    Literal(i32),
    Register(String),
}
