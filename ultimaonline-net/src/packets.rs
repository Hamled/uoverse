use crate::error::Result;
use serde::Serialize;
pub mod login;

#[derive(Serialize)]
pub struct Packet<'a, T> {
    id: u8,
    size: Option<u16>,
    contents: &'a T,
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
