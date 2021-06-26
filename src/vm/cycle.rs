use fastrand;

use super::error::ExaError;
use super::exa::Exa;
use super::{Shared, VM};

impl<'a> VM<'a> {
    pub fn run_cycles(&mut self, num_cycles: usize) {
        for _ in 0..num_cycles {
            self.run_cycle();
        }
    }

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

        // TODO: Clear out killed EXAs from message bus send queues

        // Run message buses
        self.bus.borrow_mut().run_cycle();
        for host in self.hosts.values_mut() {
            host.borrow_mut().bus.run_cycle();
        }

        // Run KILLs. These seem to have a special execution order
        // so they need to go before other EXA commands. KILLs are based
        // on positioning at the start of the cycle, and if you get killed,
        // you don't get to run anything else this cycle.
        let killers = self
            .exas
            .iter()
            .filter(|e| e.borrow().will_kill_this_cycle());

        for killer in killers {
            let kill_target = self.kill_target(&killer.borrow());
            if kill_target.is_some() {
                kill_target.unwrap().borrow_mut().error = Some(ExaError::Fatal("killed").into());
            }
        }

        // Gather TEST MRDs, before they increment their pcs. TEST MRD seems
        // like it needs to happen after everything
        // else, since it'll return True even if, on that cycle, reading
        // would have actually blocked. But the *next* cycle it won't block.
        let test_mrds: Vec<Shared<Exa>> = self
            .exas
            .clone()
            .into_iter()
            .filter(|e| e.borrow().will_test_mrd_this_cycle())
            .collect();

        // Run EXA cycles
        let mut runnable: Vec<Shared<Exa>> = self
            .exas
            .clone()
            .into_iter()
            // Do not run frozen EXAs until something else thaws them
            .filter(|e| !e.borrow().is_frozen())
            // Do not run EXAs that were just killed
            .filter(|e| !e.borrow().is_fatal())
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

        // Run the TEST MRDs from earlier.
        for exa in test_mrds.iter() {
            exa.borrow_mut().test_mrd();
        }

        self.cycle += 1;
    }

    /// Kill targeting seems pretty complex, and I haven't been able to
    /// fully reverse engineer it. We're going to try this for now and hope
    /// we don't run into any programs that are actually relying on the
    /// exact kill behavior in the retail game. Kill targets are prioritized
    /// based on:
    /// - whether they are also performing a KILL this turn
    /// - whether they are in our EXA chain, and newer than us
    /// - whether they are in our EXA chain, and older than us
    /// - everyone else
    /// We take the first group that has any members, and pick a random
    /// as-of-yet unkilled member from it, then kill it.
    fn kill_target(&self, killer: &Exa<'a>) -> Option<Shared<Exa<'a>>> {
        let k = killer;
        let host_exas: Vec<Shared<Exa<'a>>> = self
            .exas
            .clone()
            .into_iter()
            .filter(|e| *e.borrow() != *k && *e.borrow().host == *k.host)
            .collect();

        if host_exas.len() == 0 {
            return None;
        }

        let other_killers: Vec<Shared<Exa<'a>>> = host_exas
            .clone()
            .into_iter()
            .filter(|e| e.borrow().will_kill_this_cycle())
            .collect();

        if other_killers.len() != 0 {
            let choice = &other_killers[fastrand::usize(..other_killers.len())];
            return Some(choice.clone());
        }

        let descendants: Vec<Shared<Exa<'a>>> = host_exas
            .clone()
            .into_iter()
            .filter(|e| e.borrow().descendant_of(killer))
            .collect();

        if descendants.len() != 0 {
            let choice = &descendants[fastrand::usize(..descendants.len())];
            return Some(choice.clone());
        }

        let ancestors: Vec<Shared<Exa<'a>>> = host_exas
            .clone()
            .into_iter()
            .filter(|e| e.borrow().ancestor_of(killer))
            .collect();

        if ancestors.len() != 0 {
            let choice = &ancestors[fastrand::usize(..ancestors.len())];
            return Some(choice.clone());
        }

        let choice = &host_exas[fastrand::usize(..host_exas.len())];
        Some(choice.clone())
    }
}
