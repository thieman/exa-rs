use std::error::Error;

use super::super::error::ExaError;
use super::super::instruction::{Instruction, Target};
use super::super::register::Register;
use super::super::{Permissions, Shared};
use super::{Exa, Mode};

fn clamp(value: i32) -> i32 {
    if value > 9999 {
        9999
    } else if value < -9999 {
        -9999
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
    pub fn run_cycle(&mut self) -> &CycleResult {
        // Reset result struct to pass up to VM
        self.result = CycleResult::new();

        let result = match &self.instructions[self.pc].clone() {
            Instruction::Link(ref dest) => self.link(dest),
            Instruction::Copy(ref src, ref dest) => self.copy(src, dest),
            Instruction::Addi(ref left, ref right, ref dest) => self.addi(left, right, dest),
            Instruction::Subi(ref left, ref right, ref dest) => self.subi(left, right, dest),
            Instruction::Muli(ref left, ref right, ref dest) => self.muli(left, right, dest),
            Instruction::Divi(ref left, ref right, ref dest) => self.divi(left, right, dest),
            Instruction::Modi(ref left, ref right, ref dest) => self.modi(left, right, dest),
            Instruction::Swiz(ref input, ref mask, ref dest) => self.swiz(input, mask, dest),
            Instruction::Halt => Err(ExaError::Fatal("explicit halt").into()),
            Instruction::Noop => Ok(()),
            _ => Ok(()),
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
            panic!("cannot call unfreeze, exa is not frozen");
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
        {
            let links = &mut start_host.borrow_mut().links;
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
                if back_link.to_host == start_host {
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
            Target::Register(r) => self.write_register(r, clamp(value)),
        }
    }

    fn subi(&mut self, left: &Target, right: &Target, dest: &Target) -> ExaResult {
        let value = self.read_target(left)? - self.read_target(right)?;

        match dest {
            Target::Literal(_) => Err(ExaError::Fatal("cannot write to literal").into()),
            Target::Register(r) => self.write_register(r, clamp(value)),
        }
    }

    fn muli(&mut self, left: &Target, right: &Target, dest: &Target) -> ExaResult {
        let value = self.read_target(left)? * self.read_target(right)?;

        match dest {
            Target::Literal(_) => Err(ExaError::Fatal("cannot write to literal").into()),
            Target::Register(r) => self.write_register(r, clamp(value)),
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
            Target::Register(r) => self.write_register(r, clamp(value)),
        }
    }

    fn modi(&mut self, left: &Target, right: &Target, dest: &Target) -> ExaResult {
        let right = self.read_target(right)?;
        if right == 0 {
            return Err(ExaError::Fatal("divide by zero").into());
        }

        let value = self.read_target(left)? % right;

        match dest {
            Target::Literal(_) => Err(ExaError::Fatal("cannot write to literal").into()),
            Target::Register(r) => self.write_register(r, clamp(value)),
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

    fn read_target(&mut self, t: &Target) -> Result<i32, Box<dyn Error>> {
        match t {
            Target::Literal(l) => Ok(*l),
            Target::Register(r) => self.read_register(r),
        }
    }

    pub fn read_register(&mut self, r_specifier: &str) -> Result<i32, Box<dyn Error>> {
        if r_specifier == "m" {
            return self.read_from_bus();
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
            _ => Ok(b.value),
        }
    }

    fn write_register(&mut self, r_specifier: &str, value: i32) -> ExaResult {
        if r_specifier == "m" {
            return self.write_to_bus(value);
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

        b.value = value;
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
