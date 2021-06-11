extern crate simple_error;

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use itertools::Itertools;

use self::exa::Exa;
use error::ExaError;
use register::Register;

pub mod cycle;
pub mod error;
pub mod exa;
pub mod file;
pub mod instruction;
pub mod register;

pub type Shared<T> = Rc<RefCell<T>>;

#[derive(PartialEq, Eq, Debug)]
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
}

impl<'a> Host<'_> {
    pub fn new(name: String, capacity: usize) -> Host<'a> {
        Host {
            name,
            capacity,
            occupied: 0,
            links: HashMap::new(),
            registers: HashMap::new(),
        }
    }
    pub fn new_shared(name: String, capacity: usize) -> Shared<Host<'a>> {
        Rc::new(RefCell::new(Host::new(name, capacity)))
    }

    /// Increments occupied by 1, if there is remaining capacity. Successful
    /// calls mean you need to free_slot later when you leave the Host.
    pub fn reserve_slot(&mut self) -> Result<(), Box<ExaError<'a>>> {
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
}

impl<'a> VM<'a> {
    pub fn new() -> VM<'a> {
        VM {
            cycle: 0,
            hosts: HashMap::new(),
            exas: Vec::new(),
        }
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
