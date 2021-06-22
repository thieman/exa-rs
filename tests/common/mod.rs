extern crate exa;

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use exa::vm::error::ExaError;
use exa::vm::exa::{Exa, Mode};
use exa::vm::register::Register;
use exa::vm::{Host, Permissions, Shared, VM};

pub struct TestBench<'a> {
    vm: Shared<VM<'a>>,
    spawned: usize,
    redshift: bool,
}

impl fmt::Display for TestBench<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.vm.borrow())
    }
}

#[allow(dead_code)]
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
            redshift: false,
        }
    }

    pub fn redshift_vm() -> TestBench<'a> {
        let vm = VM::new_redshift();

        TestBench {
            vm: Rc::new(RefCell::new(vm)),
            spawned: 0,
            redshift: true,
        }
    }

    /// Spawn an Exa in the first host.
    pub fn exa(&mut self, script: &str) -> Shared<Exa<'a>> {
        let name = if self.redshift { "core" } else { "start" };
        let host = self.vm.borrow().hosts.get(name).unwrap().clone();
        let mut name = String::from("x");
        name.push_str(&self.spawned.to_string());
        self.spawned += 1;
        Exa::spawn(
            &mut self.vm.clone().borrow_mut(),
            host,
            name,
            self.redshift,
            script,
        )
        .unwrap()
    }

    /// Spawn an Exa, with all available options.
    pub fn exa_custom(&mut self, script: &str, host: &str, mode: Mode) -> Shared<Exa<'a>> {
        let host = self.vm.borrow().hosts.get(host).unwrap().clone();
        let mut name = String::from("x");
        name.push_str(&self.spawned.to_string());
        self.spawned += 1;
        let e = Exa::spawn(
            &mut self.vm.clone().borrow_mut(),
            host,
            name,
            self.redshift,
            script,
        )
        .unwrap();
        e.borrow_mut().mode = mode;
        e
    }

    /// Get an exa by its name. Useful for grabbing new EXAs that have been
    /// spawned via REPL commands.
    pub fn get_exa(&mut self, name: &str) -> Shared<Exa<'a>> {
        self.vm.borrow_mut().get_exa(name)
    }

    pub fn run_cycle(&mut self) {
        self.vm.borrow_mut().run_cycle();
        println!("{}", self);
    }

    pub fn assert_position(&self, exa: &Shared<Exa<'a>>, hostname: &str) {
        assert_eq!(exa.borrow().host.borrow().name, hostname);
    }

    pub fn assert_exa_register(&self, exa: &Shared<Exa<'a>>, specifier: &str, value: i32) {
        let v = exa.borrow_mut().read_register(specifier).unwrap();
        assert_eq!(v, value, "wanted {} got {}", value, v);
    }

    pub fn assert_exa_no_file(&self, exa: &Shared<Exa<'a>>) {
        assert!(exa.borrow().file.is_none());
    }

    pub fn assert_exa_file(&self, exa: &Shared<Exa<'a>>, file_id: i32) {
        assert_eq!(
            exa.borrow().file.as_ref().expect("no file held").id,
            file_id
        );
    }

    pub fn assert_exa_file_contents(&self, exa: &Shared<Exa<'a>>, contents: Vec<i32>) {
        let e = exa.borrow();
        let f = e.file.as_ref().expect("no file held");
        assert_eq!(f.contents, contents);
    }

    pub fn assert_host_file(&self, hostname: &str, file_id: i32) {
        let vm = self.vm.borrow();
        let host = vm.hosts.get(hostname).expect("unknown host");
        for f in host.borrow().files.iter() {
            if f.id == file_id {
                return;
            }
        }
        panic!("file not found");
    }

    pub fn assert_host_no_file(&self, hostname: &str, file_id: i32) {
        let vm = self.vm.borrow();
        let host = vm.hosts.get(hostname).expect("unknown host");
        for f in host.borrow().files.iter() {
            if f.id == file_id {
                panic!("file found");
            }
        }
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

    pub fn assert_freezing_error(&self, exa: &Shared<Exa<'a>>) {
        let e = exa.borrow();
        let error = e.error.as_ref().unwrap();
        match error.downcast_ref::<ExaError>() {
            Some(e) => match *e {
                ExaError::Freezing(_) => (),
                _ => panic!("expected freezing error, got {}", e),
            },
            _ => panic!("expected freezing error, got {}", e),
        }
    }

    pub fn assert_no_error(&self, exa: &Shared<Exa<'a>>) {
        let e = exa.borrow();
        assert!(
            e.error.is_none(),
            "expected no error, got {}",
            e.error.as_ref().unwrap()
        );
    }

    pub fn assert_alive(&self, exa: &Shared<Exa<'a>>) {
        for test_exa in self.vm.borrow().exas.iter() {
            if test_exa == exa {
                return;
            }
        }
        panic!("exa is not alive")
    }
    pub fn assert_dead(&self, exa: &Shared<Exa<'a>>) {
        for test_exa in self.vm.borrow().exas.iter() {
            assert_ne!(test_exa, exa);
        }
    }
}
