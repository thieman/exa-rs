use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ExaError<'a> {
    Blocking(&'a str),
    Fatal(&'a str),
}

impl<'a> fmt::Display for ExaError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ExaError::Blocking(m) => write!(f, "blocking: {}", m),
            ExaError::Fatal(m) => write!(f, "fatal: {}", m),
        }
    }
}

impl<'a> error::Error for ExaError<'a> {}
