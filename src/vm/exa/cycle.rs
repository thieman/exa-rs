use std::error::Error;
use std::sync::atomic::Ordering;

use fastrand;

use super::super::error::ExaError;
use super::super::file::File;
use super::super::instruction::{Comparator, Instruction, Target};
use super::super::register::Register;
use super::super::VM;
use super::super::{Permissions, Shared};
use super::sprite::Sprite;
use super::{Exa, Mode};

fn clamp(value: i32, min: i32, max: i32) -> i32 {
    if value > max {
        max
    } else if value < min {
        min
    } else {
        value
    }
}

/// Splits a value into a vec of its digits. Throws away the sign, and
/// pads so that there are always exactly 4 digits.
fn int_to_digits(value: i32) -> Vec<u32> {
    let mut value_string = value.abs().to_string();

    if value_string.len() < 4 {
        let mut new_string = String::from("");
        for _ in 0..4 - value_string.len() {
            new_string.push_str("0");
        }
        new_string.push_str(&value_string);
        value_string = new_string;
    }

    value_string
        .chars()
        .map(|d| d.to_digit(10).unwrap())
        .collect::<Vec<u32>>()
}

type ExaResult = Result<(), Box<dyn Error>>;

/// Used to report any information from an EXA's cycle run
/// back up the chain to the VM.
#[derive(Debug, PartialEq, Eq)]
pub struct CycleResult {
    pub unfreeze_exa: Option<String>,
}

impl CycleResult {
    pub fn new() -> CycleResult {
        CycleResult { unfreeze_exa: None }
    }
}

impl<'a> Exa<'a> {
    pub fn run_cycle(&mut self, vm: &mut VM<'a>) -> &CycleResult {
        // Reset result struct to pass up to VM
        self.result = CycleResult::new();

        if self.instructions.len() == 0 {
            self.error = Some(ExaError::Fatal("out of instructions").into());
            return &self.result;
        }

        let result = match &self.instructions[self.pc].clone() {
            Instruction::Link(ref dest) => self.link(dest),
            Instruction::Copy(ref src, ref dest) => self.copy(src, dest),
            Instruction::Addi(ref left, ref right, ref dest) => self.addi(left, right, dest),
            Instruction::Subi(ref left, ref right, ref dest) => self.subi(left, right, dest),
            Instruction::Muli(ref left, ref right, ref dest) => self.muli(left, right, dest),
            Instruction::Divi(ref left, ref right, ref dest) => self.divi(left, right, dest),
            Instruction::Modi(ref left, ref right, ref dest) => self.modi(left, right, dest),
            Instruction::Swiz(ref input, ref mask, ref dest) => self.swiz(input, mask, dest),
            Instruction::Jump(ref label) => self.jump(label),
            Instruction::Tjmp(ref label) => self.tjmp(label),
            Instruction::Fjmp(ref label) => self.fjmp(label),
            Instruction::Test(ref left, ref comp, ref right) => self.test(left, comp, right),
            Instruction::Repl(ref label) => self.repl(vm, label),
            Instruction::Mode => {
                match self.mode {
                    Mode::Local => self.mode = Mode::Global,
                    Mode::Global => self.mode = Mode::Local,
                }
                Ok(())
            }
            Instruction::VoidM => self.read_register("m").map(|_| ()),
            Instruction::Make => self.make_file(),
            Instruction::Drop => self.drop_file(),
            Instruction::Wipe => self.wipe_file(),
            Instruction::Grab(ref file_target) => self.grab_file(file_target),
            Instruction::Halt => Err(ExaError::Fatal("explicit halt").into()),
            Instruction::Seek(ref target) => self.seek_file(target),
            Instruction::VoidF => self.void_file(),
            Instruction::File(ref target) => self.file_command(target),
            Instruction::TestEof => self.test_eof(),
            Instruction::Rand(ref lo, ref hi, ref dest) => self.rand(lo, hi, dest),
            Instruction::Noop => Ok(()),
            Instruction::Mark(_) => panic!("marks should have been preprocessed out"),
            // host is unsupported because we don't support keywords. convert to noop
            Instruction::Host(_) => Ok(()),
            // kills are handled in the VM's run_cycle, before everything else
            Instruction::Kill => Ok(()),
            // test mrd is handled in the VM's run_cycle, after everything else
            Instruction::TestMrd => Ok(()),
            // data is handled when EXAs are spawned at the beginning of the program
            Instruction::Data(_) => panic!("datas should have been preprocessed out"),
            // waits freeze until the draw routine unfreezes
            Instruction::Wait => {
                self.waiting = true;
                Err(ExaError::Freezing("waiting").into())
            }
        };

        self.error = match result {
            Ok(_) => None,
            Err(e) => Some(e),
        };

        if self.error.is_none() {
            self.pc += 1;

            if self.pc > self.instructions.len() - 1 {
                self.error = Some(ExaError::Fatal("out of instructions").into());
            }
        }

        return &self.result;
    }

    pub fn unfreeze(&mut self) {
        if !self.is_frozen() {
            panic!("cannot call unfreeze on {}, exa is not frozen", self.name);
        }

        self.error = None;
        self.pc += 1;
        if self.pc > self.instructions.len() - 1 {
            self.error = Some(ExaError::Fatal("out of instructions").into());
        }
    }

    fn link(&mut self, dest: &Target) -> ExaResult {
        let link_id = self.read_target(dest)?;

        let start_host = self.host.clone();
        let start_host_name = start_host.borrow().name.to_string();
        {
            let s = &mut start_host.borrow_mut();
            let links = &mut s.links;
            let link = links.get_mut(&link_id);

            if link.is_none() {
                return Err(ExaError::Fatal("invalid link id").into());
            }

            let l = link.unwrap();
            if l.traversed_this_cycle {
                return Err(ExaError::Blocking("link bandwidth exceeded").into());
            }

            let mut to_host = l.to_host.borrow_mut();
            to_host
                .reserve_slot()
                .map_err(|_| Box::new(ExaError::Blocking("destination host is full")))?;

            for back_link in to_host.links.values_mut() {
                if back_link.to_host_name == start_host_name {
                    back_link.traversed_this_cycle = true;
                }
            }

            l.traversed_this_cycle = true;
            self.host = l.to_host.clone();
        }

        start_host.borrow_mut().free_slot();

        Ok(())
    }

    fn copy(&mut self, src: &Target, dest: &Target) -> ExaResult {
        let src_value = self.read_target(src)?;

        match dest {
            Target::Literal(_) => Err(ExaError::Fatal("cannot write to literal").into()),
            Target::Register(r) => self.write_register(r, src_value),
        }
    }

    fn addi(&mut self, left: &Target, right: &Target, dest: &Target) -> ExaResult {
        let value = self.read_target(left)? + self.read_target(right)?;

        match dest {
            Target::Literal(_) => Err(ExaError::Fatal("cannot write to literal").into()),
            Target::Register(r) => self.write_register(r, clamp(value, -9999, 9999)),
        }
    }

    fn subi(&mut self, left: &Target, right: &Target, dest: &Target) -> ExaResult {
        let value = self.read_target(left)? - self.read_target(right)?;

        match dest {
            Target::Literal(_) => Err(ExaError::Fatal("cannot write to literal").into()),
            Target::Register(r) => self.write_register(r, clamp(value, -9999, 9999)),
        }
    }

    fn muli(&mut self, left: &Target, right: &Target, dest: &Target) -> ExaResult {
        let value = self.read_target(left)? * self.read_target(right)?;

        match dest {
            Target::Literal(_) => Err(ExaError::Fatal("cannot write to literal").into()),
            Target::Register(r) => self.write_register(r, clamp(value, -9999, 9999)),
        }
    }

    fn divi(&mut self, left: &Target, right: &Target, dest: &Target) -> ExaResult {
        let right = self.read_target(right)?;
        if right == 0 {
            return Err(ExaError::Fatal("divide by zero").into());
        }

        let value = self.read_target(left)? / right;

        match dest {
            Target::Literal(_) => Err(ExaError::Fatal("cannot write to literal").into()),
            Target::Register(r) => self.write_register(r, clamp(value, -9999, 9999)),
        }
    }

    fn modi(&mut self, left: &Target, right: &Target, dest: &Target) -> ExaResult {
        let right = self.read_target(right)?;
        if right == 0 {
            return Err(ExaError::Fatal("divide by zero").into());
        }

        let left = self.read_target(left)?;
        let r = left % right;
        let value = if r < 0 { r + right } else { r };

        match dest {
            Target::Literal(_) => Err(ExaError::Fatal("cannot write to literal").into()),
            Target::Register(r) => self.write_register(r, clamp(value, -9999, 9999)),
        }
    }

    fn swiz(&mut self, input: &Target, mask: &Target, dest: &Target) -> ExaResult {
        let mut value: i32 = 0;
        let input_value = self.read_target(input)?;
        let mask_value = self.read_target(mask)?;

        let mut input_digits = int_to_digits(input_value);
        input_digits.reverse();
        let mask_digits = int_to_digits(mask_value);

        for (idx, m_digit) in mask_digits.into_iter().enumerate() {
            match m_digit {
                1..=4 => {
                    let power = ((idx as i32) - 3).abs() as u32;
                    let incr = u32::pow(10, power) * input_digits[m_digit as usize - 1];
                    value += incr as i32;
                }
                0 | 5..=9 => (),
                _ => panic!("unexpected digit"),
            }
        }

        if (input_value < 0) ^ (mask_value < 0) {
            value *= -1;
        }

        match dest {
            Target::Literal(_) => Err(ExaError::Fatal("cannot write to literal").into()),
            Target::Register(r) => self.write_register(r, value),
        }
    }

    fn jump(&mut self, label: &String) -> ExaResult {
        match self.labels.get(label) {
            Some(position) => {
                // Need to -1 because we will increase the PC
                // when we return back to run_cycle
                self.pc = *position - 1;
                Ok(())
            }
            _ => Err(ExaError::Fatal("unknown label").into()),
        }
    }

    fn tjmp(&mut self, label: &String) -> ExaResult {
        if self.read_register("t")? != 0 {
            return self.jump(label);
        }
        Ok(())
    }
    fn fjmp(&mut self, label: &String) -> ExaResult {
        if self.read_register("t")? == 0 {
            return self.jump(label);
        }
        Ok(())
    }

    fn test(&mut self, left: &Target, comp: &Comparator, right: &Target) -> ExaResult {
        let (l, r) = (self.read_target(left)?, self.read_target(right)?);

        let is_true: bool;
        match comp {
            Comparator::Equal => is_true = l == r,
            Comparator::GreaterThan => is_true = l > r,
            Comparator::LessThan => is_true = l < r,
        }

        self.write_register("t", if is_true { 1 } else { 0 })?;
        Ok(())
    }

    fn repl(&mut self, vm: &mut VM<'a>, label: &String) -> ExaResult {
        let target_pc = match self.labels.get(label) {
            Some(position) => Ok(*position),
            _ => Err(Box::new(ExaError::Fatal("unknown label"))),
        }?;

        self.inner_repl(vm, target_pc)?;
        Ok(())
    }

    /// TEST MRD is a special little snowflake and is run by the VM
    /// after all other processing. It won't ever error, so we don't
    /// return an ExaResult from it.
    pub fn test_mrd(&mut self) {
        let ready = match self.mode {
            Mode::Global => self.global_bus.borrow().has_messages(),
            Mode::Local => self.host.borrow().bus.has_messages(),
        };
        self.write_register("t", if ready { 1 } else { 0 })
            .expect("error writing to T from test mrd");
    }

    fn make_file(&mut self) -> ExaResult {
        if self.file.is_some() {
            return Err(ExaError::Fatal("cannot grab a second file").into());
        }
        let file_id = self.file_counter.fetch_add(1, Ordering::Relaxed);
        self.file = Some(File::new(file_id, vec![]));
        Ok(())
    }

    fn drop_file(&mut self) -> ExaResult {
        if self.file.is_none() {
            return Err(ExaError::Fatal("no file is held").into());
        }
        let mut host_mut = self.host.borrow_mut();
        host_mut.reserve_slot()?;

        let f = self.file.take();
        host_mut.files.push(f.unwrap());
        self.file_pointer = 0;

        Ok(())
    }

    fn wipe_file(&mut self) -> ExaResult {
        if self.file.is_none() {
            return Err(ExaError::Fatal("no file is held").into());
        }

        self.file = None;
        self.file_pointer = 0;

        Ok(())
    }

    fn grab_file(&mut self, file_target: &Target) -> ExaResult {
        let file_id = self.read_target(file_target)?;
        if self.file.is_some() {
            return Err(ExaError::Fatal("cannot grab a second file").into());
        }

        let mut ok = false;
        {
            let files = &mut self.host.borrow_mut().files;
            let mut i = 0;
            while i < files.len() {
                let f = &files[i];
                if f.id == file_id {
                    self.file = Some(files.remove(i));
                    ok = true;
                    break;
                } else {
                    i += 1;
                }
            }
        }

        if ok {
            self.host.borrow_mut().free_slot();
            Ok(())
        } else {
            Err(ExaError::Fatal("file id not found").into())
        }
    }

    fn seek_file(&mut self, target: &Target) -> ExaResult {
        if self.file.is_none() {
            return Err(ExaError::Fatal("no file is held").into());
        }

        let seek_amount = self.read_target(target)?;
        self.file_pointer += seek_amount as isize;

        if self.file_pointer < 0 {
            self.file_pointer = 0;
        } else if self.file_pointer > self.file.as_ref().unwrap().contents.len() as isize {
            self.file_pointer = self.file.as_ref().unwrap().contents.len() as isize;
        }

        Ok(())
    }

    fn void_file(&mut self) -> ExaResult {
        if self.file.is_none() {
            return Err(ExaError::Fatal("no file is held").into());
        }

        let f = self.file.as_mut().unwrap();
        if self.file_pointer >= f.contents.len() as isize {
            return Err(ExaError::Fatal("cannot void from file at append position").into());
        }

        f.contents.remove(self.file_pointer as usize);

        Ok(())
    }

    fn file_command(&mut self, target: &Target) -> ExaResult {
        if self.file.is_none() {
            return Err(ExaError::Fatal("no file is held").into());
        }

        let file_id = self.file.as_ref().unwrap().id;
        match target {
            Target::Literal(_) => Err(ExaError::Fatal("cannot write to literal").into()),
            Target::Register(r) => self.write_register(r, file_id),
        }
    }

    fn test_eof(&mut self) -> ExaResult {
        if self.file.is_none() {
            return Err(ExaError::Fatal("no file is held").into());
        }
        let at_end = self.file_pointer == self.file.as_ref().unwrap().contents.len() as isize;
        let value = if at_end { 1 } else { 0 };

        return self.write_register("t", value);
    }

    fn rand(&mut self, lo: &Target, hi: &Target, dest: &Target) -> ExaResult {
        let (lo_value, hi_value) = (self.read_target(lo)?, self.read_target(hi)?);
        if lo_value > hi_value {
            return Err(ExaError::Fatal("invalid rand range").into());
        }

        let value = fastrand::i32(lo_value..=hi_value);
        match dest {
            Target::Literal(_) => Err(ExaError::Fatal("cannot write to literal").into()),
            Target::Register(r) => self.write_register(r, value),
        }
    }

    fn read_target(&mut self, t: &Target) -> Result<i32, Box<dyn Error>> {
        match t {
            Target::Literal(l) => Ok(*l),
            Target::Register(r) => self.read_register(r),
        }
    }

    pub fn read_register(&mut self, r_specifier: &str) -> Result<i32, Box<dyn Error>> {
        if r_specifier == "m" {
            return self.read_from_bus();
        } else if r_specifier == "f" {
            return self.read_from_file();
        }

        let r = self.resolve_register(r_specifier)?;
        let b = r.borrow();

        match b.permissions {
            Permissions::Denied => {
                return Err(ExaError::Fatal("attempt to read from deactivated register").into());
            }
            Permissions::WriteOnly => {
                return Err(ExaError::Fatal("attempt to read from write-only register").into())
            }
            _ => (),
        }

        Ok(b.value)
    }

    fn write_register(&mut self, r_specifier: &str, value: i32) -> ExaResult {
        if r_specifier == "m" {
            return self.write_to_bus(value);
        } else if r_specifier == "f" {
            return self.write_to_file(value);
        } else if r_specifier == "gp" {
            return self.write_sprite(value);
        }

        let r = self.resolve_register(r_specifier)?;
        let mut b = r.borrow_mut();

        match b.permissions {
            Permissions::Denied => {
                return Err(ExaError::Fatal("attempt to write to deactivated register").into());
            }
            Permissions::ReadOnly => {
                return Err(ExaError::Fatal("attempt to write to read-only register").into())
            }
            _ => (),
        }

        b.value = match r_specifier {
            "gx" => clamp(value, -10, 120),
            "gy" => clamp(value, -10, 100),
            "gz" => clamp(value, -9, 9),
            "#sqr0" => clamp(value, 0, 99),
            "#sqr1" => clamp(value, 0, 99),
            "#tri0" => clamp(value, 0, 99),
            "#nse0" => clamp(value, 0, 99),
            _ => value,
        };
        Ok(())
    }

    fn write_sprite(&mut self, value: i32) -> ExaResult {
        if value < 0 {
            return Ok(());
        }

        let digits = int_to_digits(value);
        match digits[1] {
            0 => self.sprite.disable(digits[2], digits[3]),
            1 => self.sprite.enable(digits[2], digits[3]),
            2 => self.sprite.toggle(digits[2], digits[3]),
            3 => self.sprite = Sprite::from_builtin((digits[2] * 10) + digits[3]),
            _ => (),
        }
        Ok(())
    }

    fn read_from_file(&mut self) -> Result<i32, Box<dyn Error>> {
        if self.file.is_none() {
            return Err(ExaError::Fatal("no file is held").into());
        }

        let f = self.file.as_ref().unwrap();
        if self.file_pointer >= f.contents.len() as isize {
            return Err(ExaError::Fatal("cannot read from file at append position").into());
        }

        let value = f.contents[self.file_pointer as usize];
        self.file_pointer += 1;
        Ok(value)
    }

    fn write_to_file(&mut self, value: i32) -> ExaResult {
        if self.file.is_none() {
            return Err(ExaError::Fatal("no file is held").into());
        }

        let f = self.file.as_mut().unwrap();
        if self.file_pointer == f.contents.len() as isize {
            f.contents.push(value);
        } else {
            f.contents[self.file_pointer as usize] = value;
        }

        self.file_pointer += 1;

        Ok(())
    }

    pub fn read_from_bus(&mut self) -> Result<i32, Box<dyn Error>> {
        let message = match self.mode {
            Mode::Global => self.global_bus.borrow_mut().read(),
            Mode::Local => self.host.borrow_mut().bus.read(),
        }?;

        self.result.unfreeze_exa = Some(message.sender);

        Ok(message.value)
    }

    pub fn write_to_bus(&mut self, value: i32) -> ExaResult {
        match self.mode {
            Mode::Global => self.global_bus.borrow_mut().write(self, value),
            Mode::Local => self.host.borrow_mut().bus.write(self, value),
        }
    }

    fn resolve_register(&self, r_specifier: &str) -> Result<Shared<Register>, Box<dyn Error>> {
        if r_specifier.starts_with('#') {
            return self.resolve_hardware_register(r_specifier);
        } else {
            return self.resolve_exa_register(r_specifier);
        }
    }

    fn resolve_hardware_register(
        &self,
        r_specifier: &str,
    ) -> Result<Shared<Register>, Box<dyn Error>> {
        let h = self.host.borrow();
        let r = h.registers.get(&r_specifier.to_ascii_lowercase());

        match r {
            Some(reg) => Ok(reg.clone()),
            None => Err(ExaError::Fatal("attempt to access unknown hardware register").into()),
        }
    }

    fn resolve_exa_register(&self, r_specifier: &str) -> Result<Shared<Register>, Box<dyn Error>> {
        let r = match r_specifier.to_ascii_lowercase().as_str() {
            "x" => self.registers.x.clone(),
            "t" => self.registers.t.clone(),
            "gx" => self.registers.gx.clone(),
            "gy" => self.registers.gy.clone(),
            "gz" => self.registers.gz.clone(),
            "gp" => self.registers.gp.clone(),
            "ci" => self.registers.ci.clone(),
            "co" => self.registers.co.clone(),
            _ => return Err(ExaError::Fatal("attempted to access unknown exa register").into()),
        };
        Ok(r)
    }

    pub fn coords(&self) -> (i32, i32) {
        (
            self.registers.gx.borrow().value,
            self.registers.gy.borrow().value,
        )
    }

    pub fn reset_collision(&mut self) {
        self.registers.ci.borrow_mut().value = -9999;
    }

    pub fn update_collision(&mut self, other: &Exa) {
        let (self_x, self_y) = (
            self.registers.gx.borrow().value,
            self.registers.gy.borrow().value,
        );
        let (other_x, other_y) = (
            other.registers.gx.borrow().value,
            other.registers.gy.borrow().value,
        );

        // Quick bounds check and bail
        let (x_diff, y_diff) = (self_x - other_x, self_y - other_y);
        if x_diff.abs() >= 10 || y_diff.abs() >= 10 {
            return;
        }

        for (idx, pixel) in self.sprite.pixels.iter().enumerate() {
            if !*pixel {
                continue;
            }
            let (row, col) = (idx as i32 / 10, idx as i32 % 10);
            let (other_row, other_col) = (row + y_diff, col + x_diff);
            if other_row < 0 || other_row > 9 || other_col < 0 || other_col > 9 {
                continue;
            }

            let other_idx = (other_row * 10 + other_col) as usize;
            let other_pixel = other.sprite.pixels[other_idx];
            if other_pixel {
                let mut self_ci = self.registers.ci.borrow_mut();
                let mut other_ci = other.registers.ci.borrow_mut();
                let self_co = self.registers.co.borrow().value;
                let other_co = other.registers.co.borrow().value;

                if self_co > other_ci.value {
                    other_ci.value = self_co;
                }

                if other_co > self_ci.value {
                    self_ci.value = other_co;
                }

                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digits() {
        assert_eq!(int_to_digits(1234), vec![1, 2, 3, 4]);
        assert_eq!(int_to_digits(0), vec![0, 0, 0, 0]);
        assert_eq!(int_to_digits(8), vec![0, 0, 0, 8]);
        assert_eq!(int_to_digits(56), vec![0, 0, 5, 6]);
        assert_eq!(int_to_digits(123), vec![0, 1, 2, 3]);
        assert_eq!(int_to_digits(-9876), vec![9, 8, 7, 6]);
    }
}
