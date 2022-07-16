use crate::error::Result;
use serde::Serialize;
use std::io::{Read, Write};

pub mod char_login;
pub mod char_select;
pub mod login;
pub mod mobile;
pub mod world;

#[derive(Serialize)]
pub struct Packet<T> {
    id: u8,
    size: Option<u16>,
    contents: T,
}

impl<T> Packet<T>
where
    T: Serialize,
{
    pub fn to_writer<W: Write>(&self, writer: &mut W) -> Result<()> {
        crate::ser::to_writer(writer, self)
    }
}

pub trait FromPacketData
where
    Self: Sized,
{
    fn from_packet_data<R: Read>(reader: &mut R) -> Result<Self>;
}
