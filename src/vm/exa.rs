use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

use super::super::parse::parse_text;
use super::file::File;
use super::instruction::Instruction;
use super::Permissions;
use super::{Host, Shared, VM};

#[derive(Debug)]
struct Register {
    permissions: Permissions,
    value: i16,
}

#[derive(Debug)]
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

#[derive(Debug)]
enum Mode {
    Local,
    Global,
}

#[derive(Debug)]
pub struct Exa<'a> {
    pub name: String,
    registers: Registers,
    pc: u16,
    instructions: Vec<Instruction>,
    mode: Mode,
    file_pointer: u16,
    file: Option<Rc<File>>,
    pub host: Shared<Host<'a>>,
}

impl<'a> Exa<'a> {
    /// Spawn an Exa in the specified Host, if there is available space.
    pub fn spawn(
        vm: &mut VM<'a>,
        host: Shared<Host<'a>>,
        name: String,
        script: &str,
    ) -> Result<Shared<Exa<'a>>, Box<dyn Error>> {
        host.borrow_mut().reserve_slot()?;
        let e = Rc::new(RefCell::new(Exa {
            name,
            registers: Registers::new(),
            pc: 0,
            instructions: parse_text(script).unwrap(),
            mode: Mode::Global,
            file_pointer: 0,
            file: None,
            host: host,
        }));
        vm.register_exa(e.clone());
        Ok(e)
    }
}

impl fmt::Display for Exa<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Exa {}", self.name)
    }
}
