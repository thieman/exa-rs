use super::exa::Exa;
use super::{Shared, VM};

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

        // Run message buses
        self.bus.borrow_mut().run_cycle();
        for host in self.hosts.values_mut() {
            host.borrow_mut().bus.run_cycle();
        }

        // Run EXAs
        let mut runnable: Vec<Shared<Exa>> = self
            .exas
            .clone()
            .into_iter()
            .filter(|e| !e.borrow().is_frozen())
            .collect();

        while runnable.len() != 0 {
            let exa = runnable.remove(0);
            let mut exa_mut = exa.borrow_mut();
            let result = exa_mut.run_cycle(self);

            if let Some(e) = &result.unfreeze_exa {
                let to_unfreeze = self.get_exa(e);
                to_unfreeze.borrow_mut().unfreeze();
                if !to_unfreeze.borrow_mut().is_fatal() {
                    runnable.push(to_unfreeze);
                }
            }
        }

        self.cycle += 1;
    }
}
