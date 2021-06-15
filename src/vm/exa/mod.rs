mod cycle;

use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

use super::super::parse::parse_text;
use super::bus::MessageBus;
use super::error::ExaError;
use super::file::File;
use super::instruction::Instruction;
use super::register::Register;
use super::Permissions;
use super::{Host, Shared, VM};

use cycle::CycleResult;

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
pub enum Mode {
    Local,
    Global,
}

#[derive(Debug)]
pub struct Exa<'a> {
    pub name: String,
    registers: Registers,
    pc: usize,
    instructions: Vec<Instruction>,
    // Map of label name to index in self.instructions
    labels: HashMap<String, usize>,
    pub mode: Mode,
    file_pointer: u16,
    file: Option<Rc<File>>,
    global_bus: Shared<MessageBus>,
    pub host: Shared<Host<'a>>,
    pub error: Option<Box<dyn Error>>,
    result: CycleResult,
}

impl PartialEq for Exa<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Exa<'_> {}

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
        let mut insts = parse_text(script).unwrap();
        let labels = Exa::extract_labels(&mut insts);
        let e = Rc::new(RefCell::new(Exa {
            name,
            registers: Registers::new(),
            pc: 0,
            instructions: insts,
            labels: labels,
            mode: Mode::Global,
            file_pointer: 0,
            file: None,
            global_bus: vm.bus.clone(),
            host: host,
            error: None,
            result: CycleResult::new(),
        }));
        vm.register_exa(e.clone());
        Ok(e)
    }

    fn extract_labels(instructions: &mut Vec<Instruction>) -> HashMap<String, usize> {
        let mut m = HashMap::new();

        let mut idx = 0;
        while idx < instructions.len() {
            let inst = &instructions[idx];
            match inst {
                Instruction::Mark(label) => {
                    m.insert(label.to_string(), idx);
                    instructions.remove(idx);
                }
                _ => idx += 1,
            }
        }

        m
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

    pub fn is_frozen(&self) -> bool {
        match &self.error {
            None => false,
            Some(e) => match e.downcast_ref::<ExaError>() {
                Some(e) => match *e {
                    ExaError::Freezing(_) => true,
                    _ => false,
                },
                _ => false,
            },
        }
    }
}

impl fmt::Display for Exa<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Exa {}", self.name)?;
        if let Some(e) = &self.error {
            write!(f, " pc:{} (error: {})", self.pc, e)?;
        }
        if self.pc < self.instructions.len() - 1 {
            write!(f, "\t{:?}", self.instructions[self.pc])?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_labels() {
        let mut insts = vec![
            Instruction::Mark("first".into()),
            Instruction::Noop,
            Instruction::Mark("second".into()),
            Instruction::Mark("third".into()),
            Instruction::Noop,
        ];
        let extracted = Exa::extract_labels(&mut insts);
        assert_eq!(insts, vec![Instruction::Noop, Instruction::Noop]);
        assert_eq!(*extracted.get("first").expect("not found"), 0);
        assert_eq!(*extracted.get("second").expect("not found"), 1);
        assert_eq!(*extracted.get("third").expect("not found"), 1);
    }

    #[test]
    fn extract_labels_at_end() {
        let mut insts = vec![
            Instruction::Mark("first".into()),
            Instruction::Noop,
            Instruction::Mark("second".into()),
            Instruction::Mark("third".into()),
        ];
        let extracted = Exa::extract_labels(&mut insts);
        assert_eq!(insts, vec![Instruction::Noop]);
        assert_eq!(*extracted.get("first").expect("not found"), 0);
        assert_eq!(*extracted.get("second").expect("not found"), 1);
        assert_eq!(*extracted.get("third").expect("not found"), 1);
    }
}
