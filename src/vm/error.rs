use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ExaError<'a> {
    Blocking(&'a str),
    Fatal(&'a str),
    /// Freezing errors cause an Exa to skip processing cycles until
    /// the freeze is released by an outside process. One use case here
    /// is freezing an Exa after an M write until it is read by
    /// another Exa.
    Freezing(&'a str),
}

impl<'a> fmt::Display for ExaError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ExaError::Blocking(m) => write!(f, "blocking: {}", m),
            ExaError::Fatal(m) => write!(f, "fatal: {}", m),
            ExaError::Freezing(m) => write!(f, "freezing: {}", m),
        }
    }
}

impl<'a> error::Error for ExaError<'a> {}
