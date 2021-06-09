use std::error::Error;
use std::rc::Rc;

use super::super::parse::parse_text;
use super::file::File;
use super::instruction::Instruction;
use super::Permissions;
use super::{Host, Shared};

struct Register {
    permissions: Permissions,
    value: i16,
}

struct Registers {
    X: Register,
    T: Register,
    GX: Register,
    GY: Register,
    GZ: Register,
    GP: Register,
    CI: Register,
    CO: Register,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            X: Register {
                permissions: Permissions::ReadWrite,
                value: 0,
            },
            T: Register {
                permissions: Permissions::ReadWrite,
                value: 0,
            },
            GX: Register {
                permissions: Permissions::Denied,
                value: 0,
            },
            GY: Register {
                permissions: Permissions::Denied,
                value: 0,
            },
            GZ: Register {
                permissions: Permissions::Denied,
                value: 0,
            },
            GP: Register {
                permissions: Permissions::Denied,
                value: 0,
            },
            CI: Register {
                permissions: Permissions::Denied,
                value: 0,
            },
            CO: Register {
                permissions: Permissions::Denied,
                value: 0,
            },
        }
    }

    pub fn new_redshift() -> Registers {
        Registers {
            X: Register {
                permissions: Permissions::ReadWrite,
                value: 0,
            },
            T: Register {
                permissions: Permissions::ReadWrite,
                value: 0,
            },
            GX: Register {
                permissions: Permissions::Denied,
                value: 0,
            },
            GY: Register {
                permissions: Permissions::Denied,
                value: 0,
            },
            GZ: Register {
                permissions: Permissions::Denied,
                value: 0,
            },
            GP: Register {
                permissions: Permissions::WriteOnly,
                value: 0,
            },
            CI: Register {
                permissions: Permissions::ReadOnly,
                value: 0,
            },
            CO: Register {
                permissions: Permissions::Denied,
                value: 0,
            },
        }
    }
}

enum Mode {
    Local,
    Global,
}

pub struct Exa<'a> {
    name: String,
    registers: Registers,
    pc: u16,
    instructions: Vec<Instruction>,
    mode: Mode,
    file_pointer: u16,
    file: Option<Rc<File>>,
    host: Shared<Host<'a>>,
}

impl<'a> Exa<'a> {
    /// Spawn an Exa in the specified Host, if there is available space.
    pub fn spawn(
        host: Shared<Host<'a>>,
        name: String,
        script: &str,
    ) -> Result<Exa<'a>, Box<dyn Error>> {
        host.borrow_mut().reserve_slot()?;
        Ok(Exa {
            name,
            registers: Registers::new(),
            pc: 0,
            instructions: parse_text(script).unwrap(),
            mode: Mode::Global,
            file_pointer: 0,
            file: None,
            host: host,
        })
    }
}
