use super::VM;

impl<'a> VM<'a> {
    pub fn run_cycle(&mut self) {
        self.cycle += 1;
    }
}
