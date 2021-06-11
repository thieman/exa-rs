use std::error::Error;

use super::super::error::{BlockingError, FatalError};
use super::super::instruction::{Instruction, Target};
use super::super::register::Register;
use super::super::{Permissions, Shared};
use super::Exa;

impl<'a> Exa<'a> {
    pub fn run_cycle(&mut self) {
        let result = match &self.instructions[self.pc].clone() {
            Instruction::Link(ref dest) => self.link(dest),
            _ => Ok(()),
        };

        self.error = match result {
            Ok(_) => None,
            Err(e) => Some(e),
        };

        if self.pc == (self.instructions.len() - 1) {
            self.pc = 0;
        } else {
            self.pc += 1;
        }
    }

    fn link(&mut self, dest: &Target) -> Result<(), Box<dyn Error>> {
        let link_id = match dest {
            Target::Literal(l) => *l,
            Target::Register(r) => self.read_register(r)?,
        };

        let h = self.host.clone();
        let links = &mut h.borrow_mut().links;
        let link = links.get_mut(&link_id);

        if link.is_none() {
            return Err(Box::new(FatalError::new("invalid link id")));
        }

        let l = link.unwrap();
        if l.traversed_this_cycle {
            return Err(Box::new(BlockingError::new("link bandwidth exceeded")));
        }

        l.traversed_this_cycle = true;

        self.host = l.to_host.clone();
        Ok(())
    }

    fn read_register(&self, r_specifier: &str) -> Result<i32, Box<dyn Error>> {
        let r = self.resolve_register(r_specifier)?;
        let b = r.borrow();

        match b.permissions {
            Permissions::Denied => {
                return Err(Box::new(FatalError::new(
                    "attempt to read from deactivated register",
                )))
            }
            Permissions::WriteOnly => {
                return Err(Box::new(FatalError::new(
                    "attempt to read from write-only register",
                )))
            }
            _ => Ok(b.value),
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
            None => Err(Box::new(FatalError::new(
                "attempt to access unknown hardware register",
            ))),
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
            _ => {
                return Err(Box::new(FatalError::new(
                    "attempted to access unknown exa register",
                )))
            }
        };
        Ok(r)
    }
}
