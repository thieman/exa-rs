use super::VM;

impl<'a> VM<'a> {
    pub fn run_cycle(&mut self) {
        // Reset traversal status on all host links. These can only
        // support one EXA per cycle, others need to block.
        for h in self.hosts.values() {
            for link in h.borrow_mut().links.values_mut() {
                link.traversed_this_cycle = false;
            }
        }

        for exa in self.exas.iter() {
            exa.borrow_mut().run_cycle();
        }

        self.cycle += 1;
    }
}
