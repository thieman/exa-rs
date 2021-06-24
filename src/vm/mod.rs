extern crate simple_error;

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::rc::Rc;
use std::sync::atomic::AtomicI32;

use itertools::Itertools;

use self::exa::Exa;
use bus::MessageBus;
use error::ExaError;
use file::File;
use register::Register;

pub mod bus;
pub mod cycle;
pub mod error;
pub mod exa;
pub mod file;
pub mod instruction;
pub mod register;

pub type Shared<T> = Rc<RefCell<T>>;

#[derive(Debug)]
pub struct Host<'a> {
    pub name: String,

    // capacity is total squares that can be occupied by EXAs or files.
    // Do NOT include squares that are occupied by registers,
    // level art, or anything else.
    pub capacity: usize,

    // occupied is how much of the capacity is currently filled
    pub occupied: usize,

    // key is the number of the link that needs to be passed to the LINK op
    pub links: HashMap<i32, HostLink<'a>>,

    pub registers: HashMap<String, Shared<Register>>,

    pub bus: MessageBus,

    pub files: Vec<File>,
}

impl PartialEq for Host<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Host<'_> {}

impl<'a> Host<'_> {
    pub fn new(name: String, capacity: usize) -> Host<'a> {
        Host {
            name,
            capacity,
            occupied: 0,
            links: HashMap::new(),
            registers: HashMap::new(),
            bus: MessageBus::new(),
            files: vec![],
        }
    }
    pub fn new_shared(name: String, capacity: usize) -> Shared<Host<'a>> {
        Rc::new(RefCell::new(Host::new(name, capacity)))
    }

    /// Increments occupied by 1, if there is remaining capacity. Successful
    /// calls mean you need to free_slot later when you leave the Host.
    pub fn reserve_slot(&mut self) -> Result<(), Box<dyn Error>> {
        if self.capacity <= self.occupied {
            return Err(ExaError::Blocking("host has no remaining capacity").into());
        }
        self.occupied += 1;
        Ok(())
    }

    /// Decrements occupied by 1. Call this when you move out of a Host.__rust_force_expr!
    /// Calling this before reserving a slot from the same object would...be bad, don't do that.
    pub fn free_slot(&mut self) {
        self.occupied -= 1;
    }

    pub fn add_register(&mut self, name: String, register: Shared<Register>) {
        self.registers.insert(name.to_ascii_lowercase(), register);
    }
}

impl Ord for Host<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Host<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl fmt::Display for Host<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Host {} (free capacity: {} / {})",
            self.name,
            (self.capacity - self.occupied),
            self.capacity
        )
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct HostLink<'a> {
    pub to_host: Shared<Host<'a>>,
    // links can only support one traversal per cycle
    pub traversed_this_cycle: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Permissions {
    Denied,
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

#[derive(Debug)]
pub struct VM<'a> {
    pub cycle: u64,

    pub hosts: HashMap<String, Shared<Host<'a>>>,

    pub exas: Vec<Shared<Exa<'a>>>,

    pub bus: Shared<MessageBus>,

    pub file_counter: Rc<AtomicI32>,

    framebuffer: [bool; 120 * 100],
}

impl<'a> VM<'a> {
    pub fn new() -> VM<'a> {
        VM {
            cycle: 0,
            hosts: HashMap::new(),
            exas: Vec::new(),
            bus: Rc::new(RefCell::new(MessageBus::new())),
            file_counter: Rc::new(AtomicI32::new(400)),
            framebuffer: [false; 120 * 100],
        }
    }

    // Instantiate a VM matching the Redshift spec
    pub fn new_redshift() -> VM<'a> {
        let mut vm = VM::new();

        let core = Host::new_shared("core".to_string(), 18);
        let input = Host::new_shared("input".to_string(), 24);
        let sound = Host::new_shared("sound".to_string(), 24);
        let aux1 = Host::new_shared("aux1".to_string(), 3);
        let aux2 = Host::new_shared("aux2".to_string(), 3);

        let padx = Register::new(Permissions::ReadOnly, 0);
        let pady = Register::new(Permissions::ReadOnly, 0);
        let padb = Register::new(Permissions::ReadOnly, 0);
        let en3d = Register::new(Permissions::ReadOnly, 0);
        let sqr0 = Register::new(Permissions::ReadWrite, 0);
        let sqr1 = Register::new(Permissions::ReadWrite, 0);
        let tri0 = Register::new(Permissions::ReadWrite, 0);
        let nse0 = Register::new(Permissions::ReadWrite, 0);

        input
            .borrow_mut()
            .add_register(String::from("#PADX"), Rc::new(RefCell::new(padx)));
        input
            .borrow_mut()
            .add_register(String::from("#PADY"), Rc::new(RefCell::new(pady)));
        input
            .borrow_mut()
            .add_register(String::from("#PADB"), Rc::new(RefCell::new(padb)));
        input
            .borrow_mut()
            .add_register(String::from("#EN3D"), Rc::new(RefCell::new(en3d)));

        sound
            .borrow_mut()
            .add_register(String::from("#SQR0"), Rc::new(RefCell::new(sqr0)));
        sound
            .borrow_mut()
            .add_register(String::from("#SQR1"), Rc::new(RefCell::new(sqr1)));
        sound
            .borrow_mut()
            .add_register(String::from("#TRI0"), Rc::new(RefCell::new(tri0)));
        sound
            .borrow_mut()
            .add_register(String::from("#NSE0"), Rc::new(RefCell::new(nse0)));

        vm.add_host(core.clone());
        vm.add_host(input.clone());
        vm.add_host(sound.clone());
        vm.add_host(aux1.clone());
        vm.add_host(aux2.clone());

        vm.add_link(800, core.clone(), input.clone());
        vm.add_link(-1, input.clone(), core.clone());
        vm.add_link(801, core.clone(), sound.clone());
        vm.add_link(-1, sound.clone(), core.clone());
        vm.add_link(802, core.clone(), aux1.clone());
        vm.add_link(-1, aux1.clone(), core.clone());
        vm.add_link(803, core.clone(), aux2.clone());
        vm.add_link(-1, aux2.clone(), core.clone());

        vm
    }

    pub fn add_host(&mut self, host: Shared<Host<'a>>) {
        self.hosts
            .insert(String::from(&host.borrow().name), host.clone());
    }
    pub fn add_link<'b>(
        &mut self,
        link_id: i32,
        from_host: Shared<Host<'b>>,
        to_host: Shared<Host<'b>>,
    ) {
        let link = HostLink {
            to_host: to_host,
            traversed_this_cycle: false,
        };
        from_host.borrow_mut().links.insert(link_id, link);
    }
    pub fn register_exa(&mut self, exa: Shared<Exa<'a>>) {
        self.exas.push(exa);
    }
    pub fn get_exa(&self, name: &str) -> Shared<Exa<'a>> {
        for e in self.exas.iter() {
            if e.borrow().name == name {
                return e.clone();
            }
        }
        panic!("unknown exa {}", name)
    }

    // Update framebuffer based on current sprite info
    // of running EXAs, then return ref to framebuffer.
    pub fn render(&mut self) -> &[bool; 120 * 100] {
        self.framebuffer.iter_mut().for_each(|m| *m = false);

        for exa in self.exas.iter() {
            for (x, y) in exa.borrow().pixels() {
                self.framebuffer[(x + (y * 120)) as usize] = true;
            }
        }
        &self.framebuffer
    }
}

impl fmt::Display for VM<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VM (cycle:{})", self.cycle)?;
        for h in self.hosts.values().sorted() {
            write!(f, "\n\t{}", h.borrow())?;
            for e in self.exas.iter() {
                if e.borrow().host == *h {
                    write!(f, "\n\t\t{}", e.borrow())?;
                }
            }
        }
        Ok(())
    }
}
