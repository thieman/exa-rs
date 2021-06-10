use super::super::instruction::{Instruction, Target};
use super::Exa;

impl<'a> Exa<'a> {
    pub fn run_cycle(&mut self) {
        if self.pc > (self.instructions.len() - 1) {
            self.pc = 0;
        }

        match &self.instructions[self.pc].clone() {
            Instruction::Link(ref dest) => self.link(dest),
            _ => println!("something else!"),
        }

        self.pc += 1;
    }

    fn link(&mut self, dest: &Target) {
        println!("{:?}", dest);
    }
}
