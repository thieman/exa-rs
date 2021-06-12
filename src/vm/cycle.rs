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

        // Clean up EXAs with fatal errors last cycle
        let mut i = 0;
        while i != self.exas.len() {
            let exa = &self.exas[i];
            if exa.borrow().is_fatal() {
                // TODO: Drop file
                exa.borrow_mut().host.borrow_mut().free_slot();
                self.exas.remove(i);
            } else {
                i += 1;
            }
        }

        for exa in self.exas.iter() {
            exa.borrow_mut().run_cycle();
        }

        self.cycle += 1;
    }
}
