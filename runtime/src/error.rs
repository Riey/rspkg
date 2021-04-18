use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::process::ExitStatus;

pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    CrateNotFound(String),
    ArtifactNotFound(u32),
    CommandError(String, ExitStatus),
    IoError(io::Error),
    SerdeError(serde_json::Error),
    #[cfg(feature = "fetch")]
    FetchError(reqwest::Error),
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
            Error::ArtifactNotFound(index) => {
                write!(f, "Can't find artifact index {}", index)
            }
            Error::CommandError(command_name, status) => {
                write!(f, "Command {} run error: {}", command_name, status)
            }
            Error::SerdeError(error) => write!(f, "Serde error: {}", error),
            Error::IoError(error) => write!(f, "IO error: {}", error),
            #[cfg(feature = "fetch")]
            Error::FetchError(error) => write!(f, "crates.io error: {}", error),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::SerdeError(error)
    }
}

#[cfg(feature = "fetch")]
impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::FetchError(error)
    }
}

impl StdError for Error {}
