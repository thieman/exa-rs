extern crate exa;

use std::cell::RefCell;
use std::rc::Rc;

use exa::vm::error::ExaError;
use exa::vm::exa::Exa;
use exa::vm::register::Register;
use exa::vm::{Host, Permissions, Shared, VM};

pub struct TestBench<'a> {
    vm: Shared<VM<'a>>,
    spawned: usize,
}

impl<'a> TestBench<'a> {
    /// basic_vm provides a VM with two hosts, each with capacity 4.
    /// Host "start" is linked to host "end" via a 800<->-1 link. A
    /// ReadWrite register #REG exists in "one" and is initialized to 100.
    pub fn basic_vm() -> TestBench<'a> {
        let h1 = Host::new_shared(String::from("start"), 4);
        let r = Register::new(Permissions::ReadWrite, 100);
        h1.borrow_mut()
            .add_register(String::from("#REG"), Rc::new(RefCell::new(r)));

        let h2 = Host::new_shared(String::from("end"), 4);

        let mut vm = VM::new();

        vm.add_host(h1.clone());
        vm.add_host(h2.clone());

        vm.add_link(800, h1.clone(), h2.clone());
        vm.add_link(-1, h2.clone(), h1.clone());

        TestBench {
            vm: Rc::new(RefCell::new(vm)),
            spawned: 0,
        }
    }

    /// Spawn an Exa in the first host.
    pub fn exa(&mut self, script: &str) -> Shared<Exa<'a>> {
        let host = self.vm.borrow().hosts.get("start").unwrap().clone();
        let mut name = String::from("x");
        name.push_str(&self.spawned.to_string());
        self.spawned += 1;
        Exa::spawn(&mut self.vm.clone().borrow_mut(), host, name, script).unwrap()
    }

    pub fn run_cycle(&mut self) {
        self.vm.borrow_mut().run_cycle()
    }

    pub fn assert_position(&self, exa: &Shared<Exa<'a>>, hostname: &str) {
        assert_eq!(exa.borrow().host.borrow().name, hostname);
    }

    pub fn assert_fatal_error(&self, exa: &Shared<Exa<'a>>) {
        let e = exa.borrow();
        let error = e.error.as_ref().unwrap();
        match error.downcast_ref::<ExaError>() {
            Some(e) => match *e {
                ExaError::Fatal(_) => (),
                _ => panic!("expected fatal error, got {}", e),
            },
            _ => panic!("expected fatal error, got {}", e),
        }
    }

    pub fn assert_blocking_error(&self, exa: &Shared<Exa<'a>>) {
        let e = exa.borrow();
        let error = e.error.as_ref().unwrap();
        match error.downcast_ref::<ExaError>() {
            Some(e) => match *e {
                ExaError::Blocking(_) => (),
                _ => panic!("expected blocking error, got {}", e),
            },
            _ => panic!("expected blocking error, got {}", e),
        }
    }

    pub fn assert_no_error(&self, exa: &Shared<Exa<'a>>) {
        let e = exa.borrow();
        assert!(e.error.is_none());
    }
}
