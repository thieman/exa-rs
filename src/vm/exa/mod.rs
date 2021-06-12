mod cycle;

use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

use super::super::parse::parse_text;
use super::error::ExaError;
use super::file::File;
use super::instruction::Instruction;
use super::register::Register;
use super::Permissions;
use super::{Host, Shared, VM};

#[derive(Debug, PartialEq, Eq)]
struct Registers {
    x: Shared<Register>,
    t: Shared<Register>,
    gx: Shared<Register>,
    gy: Shared<Register>,
    gz: Shared<Register>,
    gp: Shared<Register>,
    ci: Shared<Register>,
    co: Shared<Register>,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            x: Register::new_shared(Permissions::ReadWrite, 0),
            t: Register::new_shared(Permissions::ReadWrite, 0),
            gx: Register::new_shared(Permissions::Denied, 0),
            gy: Register::new_shared(Permissions::Denied, 0),
            gz: Register::new_shared(Permissions::Denied, 0),
            gp: Register::new_shared(Permissions::Denied, 0),
            ci: Register::new_shared(Permissions::Denied, 0),
            co: Register::new_shared(Permissions::Denied, 0),
        }
    }

    pub fn new_redshift() -> Registers {
        Registers {
            x: Register::new_shared(Permissions::ReadWrite, 0),
            t: Register::new_shared(Permissions::ReadWrite, 0),
            gx: Register::new_shared(Permissions::ReadWrite, 0),
            gy: Register::new_shared(Permissions::ReadWrite, 0),
            gz: Register::new_shared(Permissions::ReadWrite, 0),
            gp: Register::new_shared(Permissions::WriteOnly, 0),
            ci: Register::new_shared(Permissions::ReadOnly, 0),
            co: Register::new_shared(Permissions::ReadWrite, 0),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Mode {
    Local,
    Global,
}

#[derive(Debug)]
pub struct Exa<'a> {
    pub name: String,
    registers: Registers,
    pc: usize,
    instructions: Vec<Instruction>,
    mode: Mode,
    file_pointer: u16,
    file: Option<Rc<File>>,
    pub host: Shared<Host<'a>>,
    pub error: Option<Box<dyn Error>>,
}

impl PartialEq for Exa<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<'a> Exa<'a> {
    /// Spawn an Exa in the specified Host, if there is available space.
    pub fn spawn(
        vm: &mut VM<'a>,
        host: Shared<Host<'a>>,
        name: String,
        script: &str,
    ) -> Result<Shared<Exa<'a>>, Box<dyn Error>> {
        // TODO: VM check on name uniqueness
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
            error: None,
        }));
        vm.register_exa(e.clone());
        Ok(e)
    }

    pub fn is_fatal(&self) -> bool {
        match &self.error {
            None => false,
            Some(e) => match e.downcast_ref::<ExaError>() {
                Some(e) => match *e {
                    ExaError::Fatal(_) => true,
                    _ => false,
                },
                _ => false,
            },
        }
    }
}

impl fmt::Display for Exa<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Exa {}", self.name);
        if let Some(e) = &self.error {
            write!(f, " (error: {})", e);
        }
        Ok(())
    }
}
