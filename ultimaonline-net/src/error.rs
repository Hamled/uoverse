use serde::{de, ser};
use std::fmt;
use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Io(io::Error),
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

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::Io(err) => fmt::Display::fmt(err, formatter),
            Error::Data(msg) => formatter.write_str(format!("Invalid data: {}", msg).as_str()),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl Error {
    pub fn io(err: std::io::Error) -> Self {
        Error::Io(err)
    }

    pub fn data<T: Into<String>>(msg: T) -> Self {
        Error::Data(msg.into())
    }
}

impl std::error::Error for Error {}
