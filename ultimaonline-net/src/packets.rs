use crate::error::Result;
use serde::Serialize;

pub mod char_login;
pub mod char_select;
pub mod login;

#[derive(Serialize)]
pub struct Packet<'a, T> {
    id: u8,
    size: Option<u16>,
    contents: &'a T,
}

impl<'a, T> Packet<'a, T>
where
    T: Serialize,
{
    pub fn to_writer<W: std::io::Write>(&'a self, writer: &mut W) -> Result<()> {
        crate::ser::to_writer(writer, self)
    }
}

pub trait ToPacket<'a>
where
    Self: Sized,
{
    fn to_packet(&'a self) -> Packet<'a, Self>;
}

pub trait FromPacketData
where
    Self: Sized,
{
    fn from_packet_data<R: std::io::Read>(reader: &mut R) -> Result<Self>;
}
