use std::collections::HashMap;

pub struct Host<'a> {
    pub name: String,

    // capacity is total squares that can be occupied by EXAs or files.
    // Do NOT include squares that are occupied by registers,
    // level art, or anything else.
    pub capacity: u16,

    // key is the number of the link that needs to be passed to the LINK
    pub links: HashMap<u16, HostLink<'a>>,
}

impl<'a> Host<'a> {
    pub fn new(name: String, capacity: u16) -> Host<'a> {
        Host {
            name,
            capacity,
            links: HashMap::new(),
        }
    }
}

pub struct HostLink<'a> {
    pub to_host: &'a Host<'a>,
    // links can only support one traversal per cycle
    pub traversed_this_cycle: bool,
}

pub enum Permissions {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

pub struct Register {
    pub permissions: Permissions,
    pub value: i16,
}

pub struct Level<'a> {
    hosts: HashMap<String, Host<'a>>,
}

impl<'a> Level<'a> {
    pub fn new() -> Level<'a> {
        Level {
            hosts: HashMap::new(),
        }
    }
    pub fn add_host(&mut self, h: Host<'a>) -> &Host<'a> {
        let name = h.name.clone();
        self.hosts.insert(name.clone(), h);
        self.hosts.get(&name).unwrap()
    }
    pub fn add_link<'b>(&mut self, link_id: u16, from_host: &Host<'a>, to_host: &'a Host<'a>) {
        let link = HostLink {
            to_host: to_host,
            traversed_this_cycle: false,
        };
        let f = self.hosts.get_mut(&from_host.name).unwrap();
        f.links.insert(link_id, link);
    }
}
