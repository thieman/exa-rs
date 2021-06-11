use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct BlockingError {
    message: String,
}

impl fmt::Display for BlockingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl Error for BlockingError {
    fn description(&self) -> &str {
        &self.message
    }
}
impl BlockingError {
    pub fn new(m: &str) -> BlockingError {
        BlockingError {
            message: m.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FatalError {
    message: String,
}

impl fmt::Display for FatalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl Error for FatalError {
    fn description(&self) -> &str {
        &self.message
    }
}
impl FatalError {
    pub fn new(m: &str) -> FatalError {
        FatalError {
            message: m.to_string(),
        }
    }
}
