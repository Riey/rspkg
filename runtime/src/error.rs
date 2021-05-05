use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::process::ExitStatus;
use wasmer_wasi::WasiStateCreationError;

pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    CommandError(String, ExitStatus),
    WasiError(WasiStateCreationError),
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
            Error::CommandError(command_name, status) => {
                write!(f, "Command {} run error: {}", command_name, status)
            }
            Error::WasiError(error) => write!(f, "WASI creation error: {}", error),
            Error::IoError(error) => write!(f, "IO error: {}", error),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<WasiStateCreationError> for Error {
    fn from(error: WasiStateCreationError) -> Self {
        Error::WasiError(error)
    }
}

impl StdError for Error {}
