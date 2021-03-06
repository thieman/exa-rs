mod cycle;
pub mod sprite;

use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::rc::Rc;
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};

use super::super::parse::parse_text;
use super::bus::MessageBus;
use super::error::ExaError;
use super::file::File;
use super::instruction::{Instruction, Target};
use super::register::Register;
use super::Permissions;
use super::{Host, Shared, VM};

use cycle::CycleResult;
use sprite::Sprite;

#[derive(Clone, Debug, PartialEq, Eq)]
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
            ci: Register::new_shared(Permissions::ReadOnly, -9999),
            co: Register::new_shared(Permissions::ReadWrite, 0),
        }
    }

    /// "True" clone that creates new registers for the descendant EXA,
    /// not new references to the original (which is what happens with
    /// a normal clone given we're working with Rcs)
    pub fn clone_for_repl(&self) -> Registers {
        Registers {
            x: Register::new_shared(
                self.x.borrow().permissions.clone(),
                self.x.borrow().value.clone(),
            ),
            t: Register::new_shared(
                self.t.borrow().permissions.clone(),
                self.t.borrow().value.clone(),
            ),
            gx: Register::new_shared(
                self.gx.borrow().permissions.clone(),
                self.gx.borrow().value.clone(),
            ),
            gy: Register::new_shared(
                self.gy.borrow().permissions.clone(),
                self.gy.borrow().value.clone(),
            ),
            gz: Register::new_shared(
                self.gz.borrow().permissions.clone(),
                self.gz.borrow().value.clone(),
            ),
            gp: Register::new_shared(self.gp.borrow().permissions.clone(), 0),
            ci: Register::new_shared(self.ci.borrow().permissions.clone(), -9999),
            co: Register::new_shared(
                self.co.borrow().permissions.clone(),
                self.co.borrow().value.clone(),
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Local,
    Global,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mode::Local => write!(f, "Local"),
            _ => write!(f, "Global"),
        }
    }
}

#[derive(Debug)]
pub struct Exa<'a> {
    base_name: String,
    spawn_id: u32,
    pub name: String,

    registers: Registers,
    result: CycleResult,
    spawn_counter: Rc<AtomicU32>,
    file_counter: Rc<AtomicI32>,

    pc: usize,
    instructions: Vec<Instruction>,
    // Map of label name to index in self.instructions
    labels: HashMap<String, usize>,

    pub mode: Mode,
    global_bus: Shared<MessageBus>,

    pub host: Shared<Host<'a>>,
    pub error: Option<Box<dyn Error>>,

    file_pointer: isize,
    pub file: Option<File>,

    pub sprite: Sprite,

    pub ran_test_mrd_this_cycle: bool,
    pub waiting: bool,
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
        redshift: bool,
        script: &str,
    ) -> Result<Shared<Exa<'a>>, Box<dyn Error>> {
        // TODO: VM check on name uniqueness
        host.borrow_mut().reserve_slot()?;
        let mut insts = parse_text(script).unwrap();
        let data_file = Exa::extract_data(&mut insts, vm.file_counter.clone());
        let labels = Exa::extract_labels(&mut insts);
        let e = Rc::new(RefCell::new(Exa {
            base_name: name.clone(),
            spawn_id: 0,
            name,
            registers: if redshift {
                Registers::new_redshift()
            } else {
                Registers::new()
            },
            pc: 0,
            instructions: insts,
            labels: labels,
            mode: Mode::Global,
            file_pointer: 0,
            file: data_file,
            global_bus: vm.bus.clone(),
            host: host,
            error: None,
            result: CycleResult::new(),
            spawn_counter: Rc::new(AtomicU32::new(1)),
            file_counter: vm.file_counter.clone(),
            sprite: Sprite::empty(),
            ran_test_mrd_this_cycle: false,
            waiting: false,
        }));
        vm.register_exa(e.clone());
        Ok(e)
    }

    pub fn inner_repl(&self, vm: &mut VM<'a>, pc: usize) -> Result<(), Box<dyn Error>> {
        self.host.borrow_mut().reserve_slot()?;

        let (name, spawn_id) = self.name_and_id_for_repl();

        let e = Rc::new(RefCell::new(Exa {
            base_name: self.base_name.clone(),
            spawn_id: spawn_id,
            name: name,
            registers: self.registers.clone_for_repl(),
            pc,
            instructions: self.instructions.clone(),
            labels: self.labels.clone(),
            mode: self.mode,
            file_pointer: 0,
            file: None,
            global_bus: self.global_bus.clone(),
            host: self.host.clone(),
            error: None,
            result: CycleResult::new(),
            spawn_counter: self.spawn_counter.clone(),
            file_counter: self.file_counter.clone(),
            sprite: self.sprite.clone(),
            ran_test_mrd_this_cycle: false,
            waiting: false,
        }));
        vm.register_exa(e);
        Ok(())
    }

    fn name_and_id_for_repl(&self) -> (String, u32) {
        let num = self.spawn_counter.fetch_add(1, Ordering::Relaxed);
        let mut name = self.base_name.clone();
        name.push_str(":");
        name.push_str(&num.to_string());
        return (name, num);
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

    fn extract_data(
        instructions: &mut Vec<Instruction>,
        file_counter: Rc<AtomicI32>,
    ) -> Option<File> {
        let mut contents: Vec<i32> = vec![];

        let mut idx = 0;
        while idx < instructions.len() {
            let inst = &instructions[idx];
            match inst {
                Instruction::Data(data) => {
                    contents.extend(data);
                    instructions.remove(idx);
                }
                _ => idx += 1,
            }
        }

        if contents.len() == 0 {
            return None;
        }

        let file_id = file_counter.fetch_add(1, Ordering::Relaxed);
        Some(File::new(file_id, contents))
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

    pub fn will_kill_this_cycle(&self) -> bool {
        if self.instructions.len() == 0 {
            return false;
        }
        match &self.instructions[self.pc] {
            Instruction::Kill => true,
            _ => false,
        }
    }

    pub fn will_test_mrd_this_cycle(&self) -> bool {
        if self.pc >= self.instructions.len() {
            return false;
        }
        match &self.instructions[self.pc] {
            Instruction::TestMrd => true,
            _ => false,
        }
    }

    fn has_ci_target(targets: &[&Target]) -> bool {
        for target in targets.iter() {
            match target {
                Target::Register(ref r) => {
                    if r == "ci" {
                        return true;
                    }
                }
                _ => (),
            }
        }
        false
    }

    pub fn will_use_ci_this_cycle(&self) -> bool {
        if self.pc >= self.instructions.len() {
            return false;
        }

        match &self.instructions[self.pc].clone() {
            Instruction::Link(ref dest) => Exa::has_ci_target(&[dest]),
            Instruction::Copy(ref src, ref dest) => Exa::has_ci_target(&[src, dest]),
            Instruction::Addi(ref left, ref right, ref dest) => {
                Exa::has_ci_target(&[left, right, dest])
            }
            Instruction::Subi(ref left, ref right, ref dest) => {
                Exa::has_ci_target(&[left, right, dest])
            }
            Instruction::Muli(ref left, ref right, ref dest) => {
                Exa::has_ci_target(&[left, right, dest])
            }
            Instruction::Divi(ref left, ref right, ref dest) => {
                Exa::has_ci_target(&[left, right, dest])
            }
            Instruction::Swiz(ref input, ref mask, ref dest) => {
                Exa::has_ci_target(&[input, mask, dest])
            }
            Instruction::Test(ref left, _, ref right) => Exa::has_ci_target(&[left, right]),
            Instruction::Grab(ref file_target) => Exa::has_ci_target(&[file_target]),
            Instruction::Seek(ref target) => Exa::has_ci_target(&[target]),
            Instruction::File(ref target) => Exa::has_ci_target(&[target]),
            Instruction::Rand(ref lo, ref hi, ref dest) => Exa::has_ci_target(&[lo, hi, dest]),
            _ => false,
        }
    }

    pub fn descendant_of(&self, other: &Exa<'a>) -> bool {
        self.base_name == other.base_name && self.spawn_id > other.spawn_id
    }

    pub fn ancestor_of(&self, other: &Exa<'a>) -> bool {
        self.base_name == other.base_name && self.spawn_id < other.spawn_id
    }

    // Returns (x,y) vector of currently enabled pixels
    pub fn pixels(&self) -> Vec<(usize, usize)> {
        let x = self.registers.gx.borrow().value;
        let y = self.registers.gy.borrow().value;
        let mut v = vec![];
        for (idx, pixel) in self.sprite.pixels.iter().enumerate() {
            if *pixel {
                let this_x = (idx as i32 % 10) + x;
                let this_y = (idx as i32 / 10) + y;
                if 0 <= this_x && 119 >= this_x && 0 <= this_y && 99 >= this_y {
                    v.push((this_x as usize, this_y as usize));
                }
            }
        }
        v
    }
}

impl fmt::Display for Exa<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\tExa {} pc:{} fp:{} mode:{}",
            self.name, self.pc, self.file_pointer, self.mode,
        )?;

        if let Some(e) = &self.error {
            write!(f, " (error: {})", e)?;
        } else {
            write!(f, " (error: None)")?;
        }

        if self.pc < self.instructions.len() {
            write!(f, "\n\tInst: {:?}", self.instructions[self.pc])?;
        }

        write!(
            f,
            "\n\tX: {} T: {} GX: {} GY: {} GZ: {} CI: {} CO: {}",
            &self.registers.x.borrow().value,
            &self.registers.t.borrow().value,
            &self.registers.gx.borrow().value,
            &self.registers.gy.borrow().value,
            &self.registers.gz.borrow().value,
            &self.registers.ci.borrow().value,
            &self.registers.co.borrow().value,
        )?;

        if let Some(file) = &self.file {
            write!(f, "\nHeld: {}", file)?;
        }

        write!(f, "\n")?;
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

    #[test]
    fn has_ci_target() {
        assert_eq!(true, Exa::has_ci_target(&[&Target::Register("ci".into())]));
        assert_eq!(
            true,
            Exa::has_ci_target(&[&Target::Literal(123), &Target::Register("ci".into())])
        );
        assert_eq!(false, Exa::has_ci_target(&[&Target::Literal(123)]));
        assert_eq!(false, Exa::has_ci_target(&[&Target::Register("co".into())]));
        assert_eq!(false, Exa::has_ci_target(&[]));
    }
}
