use serde::{de, ser};
use std::{fmt, io};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("packet data is invalid because {0}")]
    Data(String),
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Error {
    pub fn data(msg: impl Into<String>) -> Self {
        Error::Data(msg.into())
    }
}
