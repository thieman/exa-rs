#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    Copy(Target, Target),
    Addi(Target, Target, Target),
    Subi(Target, Target, Target),
    Muli(Target, Target, Target),
    Divi(Target, Target, Target),
    Modi(Target, Target, Target),
    Swiz(Target, Target, Target),
    Mark(Label),
    Jump(Label),
    Tjmp(Label),
    Fjmp(Label),
    Test(Target, Comparator, Target),
    Repl(Label),
    Halt,
    Kill,
    Link(Target),
    Host(Target),
    Mode,
    VoidM,
    TestMrd,
    Make,
    Grab(Target),
    File(Target),
    Seek(Target),
    VoidF,
    Drop,
    Wipe,
    TestEof,
    Noop,
    Rand(Target, Target, Target),
    Wait,
    Data(Vec<i32>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Literal(i32),
    Register(String),
}

pub type Label = String;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Comparator {
    Equal,
    GreaterThan,
    LessThan,
}
