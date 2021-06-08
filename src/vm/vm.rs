use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

type Shared<T> = Rc<RefCell<T>>;

#[derive(Debug)]
pub struct Host<'a> {
    pub name: String,

    // capacity is total squares that can be occupied by EXAs or files.
    // Do NOT include squares that are occupied by registers,
    // level art, or anything else.
    pub capacity: u16,

    // key is the number of the link that needs to be passed to the LINK op
    pub links: HashMap<u16, HostLink<'a>>,
}

impl<'a> Host<'_> {
    pub fn new(name: String, capacity: u16) -> Host<'a> {
        Host {
            name,
            capacity,
            links: HashMap::new(),
        }
    }
    pub fn new_shared(name: String, capacity: u16) -> Shared<Host<'a>> {
        Rc::new(RefCell::new(Host::new(name, capacity)))
    }
}

#[derive(Debug)]
pub struct HostLink<'a> {
    pub to_host: Shared<Host<'a>>,
    // links can only support one traversal per cycle
    pub traversed_this_cycle: bool,
}

#[derive(Debug)]
pub enum Permissions {
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
    hosts: HashMap<String, Shared<Host<'a>>>,
}

impl<'a> VM<'a> {
    pub fn new() -> VM<'a> {
        VM {
            hosts: HashMap::new(),
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
}
