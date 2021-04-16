use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::process::ExitStatus;

pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    CrateNotFound(String),
    CommandError(&'static str, ExitStatus),
    IoError(io::Error),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::CrateNotFound(name) => {
                write!(f, "Crate {} has not found", name)
            }
            Error::CommandError(command_name, status) => {
                write!(f, "Command {} run error: {}", command_name, status)
            }
            Error::IoError(error) => write!(f, "IO error: {}", error),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

impl StdError for Error {}
