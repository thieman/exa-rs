use std::cell::RefCell;
use std::rc::Rc;

use super::{Permissions, Shared};

#[derive(Debug, PartialEq, Eq)]
pub struct Register {
    pub permissions: Permissions,
    pub value: i32,
}

impl Register {
    pub fn new(permissions: Permissions, value: i32) -> Register {
        Register { permissions, value }
    }
    pub fn new_shared(permissions: Permissions, value: i32) -> Shared<Register> {
        Rc::new(RefCell::new(Register::new(permissions, value)))
    }
}
