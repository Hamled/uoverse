use serde::{de, ser};
use std::{fmt, io};

pub type Result<T> = std::result::Result<T, Error>;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),
    #[error("serialization failed because {0}")]
    Serialization(String),
    #[error("deserialization failed because {0}")]
    Deserialization(String),
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("packet data is invalid because {0}")]
    Data(String),
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Serialization(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Deserialization(msg.to_string())
    }
}

impl Error {
    pub fn data(msg: impl Into<String>) -> Self {
        Error::Data(msg.into())
    }

    pub fn ser(msg: impl Into<String>) -> Self {
        Error::Serialization(msg.into())
    }

    pub fn de(msg: impl Into<String>) -> Self {
        Error::Deserialization(msg.into())
    }
}
