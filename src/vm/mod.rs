extern crate simple_error;

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

use itertools::Itertools;

use exa::Exa;

pub mod cycle;
pub mod exa;
mod file;
pub mod instruction;

type Shared<T> = Rc<RefCell<T>>;

#[derive(PartialEq, Eq, Debug)]
pub struct Host<'a> {
    pub name: String,

    // capacity is total squares that can be occupied by EXAs or files.
    // Do NOT include squares that are occupied by registers,
    // level art, or anything else.
    pub capacity: u16,

    // occupied is how much of the capacity is currently filled
    pub occupied: u16,

    // key is the number of the link that needs to be passed to the LINK op
    pub links: HashMap<u16, HostLink<'a>>,
}

impl<'a> Host<'_> {
    pub fn new(name: String, capacity: u16) -> Host<'a> {
        Host {
            name,
            capacity,
            occupied: 0,
            links: HashMap::new(),
        }
    }
    pub fn new_shared(name: String, capacity: u16) -> Shared<Host<'a>> {
        Rc::new(RefCell::new(Host::new(name, capacity)))
    }

    /// Increments occupied by 1, if there is remaining capacity. Successful
    /// calls mean you need to free_slot later when you leave the Host.
    pub fn reserve_slot(&mut self) -> Result<(), Box<dyn Error>> {
        if self.capacity <= self.occupied {
            simple_error::bail!("host has no remaining capacity");
        }
        self.occupied += 1;
        Ok(())
    }

    /// Decrements occupied by 1. Call this when you move out of a Host.__rust_force_expr!
    /// Calling this before reserving a slot from the same object would...be bad, don't do that.
    pub fn free_slot(&mut self) {
        self.occupied -= 1;
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

#[derive(Debug)]
pub enum Permissions {
    Denied,
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

#[derive(Debug)]
pub struct Register {
    pub permissions: Permissions,
    pub value: i16,
}

#[derive(Debug)]
pub struct VM<'a> {
    cycle: u64,

    hosts: HashMap<String, Shared<Host<'a>>>,

    exas: Vec<Shared<Exa<'a>>>,
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
        link_id: u16,
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
        write!(f, "VM (cycle:{})", self.cycle);
        for h in self.hosts.values().sorted() {
            write!(f, "\n\t{}", h.borrow());
            for e in self.exas.iter() {
                if e.borrow().host == *h {
                    write!(f, "\n\t\t{}", e.borrow());
                }
            }
        }
        Ok(())
    }
}
