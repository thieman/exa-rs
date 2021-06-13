use std::error::Error;

use super::error::ExaError;
use super::exa::Exa;

#[derive(Debug, PartialEq, Eq)]
pub struct Message {
    pub sender: String,
    pub value: i32,
}

/// MessageBus, aka the M register.
/// Bus works roughly like this: a write to the bus cannot be read
/// the same cycle it is written. visible helps us track this.
/// additionally, only one message can be read out of the bus
/// per cycle. both readers and writers are released the cycle
/// that the corresponding message is read.
#[derive(Debug, PartialEq, Eq)]
pub struct MessageBus {
    messages: Vec<Message>,
    visible: usize,
    read_available: bool,
}

impl MessageBus {
    pub fn new() -> MessageBus {
        MessageBus {
            messages: vec![],
            visible: 0,
            read_available: true,
        }
    }

    pub fn read(&mut self) -> Result<Message, Box<dyn Error>> {
        if !self.read_available {
            return Err(ExaError::Blocking("no available read bandwidth on bus").into());
        }

        if self.visible == 0 || self.messages.len() == 0 {
            return Err(ExaError::Blocking("no messages available to read").into());
        }

        let read = self.messages.remove(0);
        self.visible -= 1;
        self.read_available = false;
        Ok(read)
    }

    pub fn write(&mut self, sender: &Exa, value: i32) -> Result<(), Box<dyn Error>> {
        self.messages.push(Message {
            sender: sender.name.to_string(),
            value: value,
        });

        Err(ExaError::Freezing("bus write successful, freezing until it is read").into())
    }

    /// Reset read_available and make all written messages visible. Needs to be
    /// run before Exa cycles each VM cycle.
    pub fn run_cycle(&mut self) {
        self.read_available = true;
        self.visible = self.messages.len();
    }
}
