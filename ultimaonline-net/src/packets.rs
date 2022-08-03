use crate::error::Result;
use serde::Serialize;
use std::io::{BufRead, Write};

pub mod action;
pub mod char_login;
pub mod char_select;
pub mod chat;
pub mod client_info;
pub mod entity;
pub mod gump;
pub mod housing;
pub mod login;
pub mod mobile;
pub mod movement;
pub mod network;
pub mod world;

pub const EXTENDED_PACKET_ID: u8 = 0xBF;

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
    fn from_packet_data<R: BufRead>(reader: &mut R) -> Result<Self>;
}

pub fn write_packet<T, U, W: Write>(content: T, dst: &mut W) -> Result<()>
where
    T: Serialize,
    U: Serialize,
    Packet<U>: From<T>,
{
    Packet::<U>::from(content).to_writer(dst)?;
    Ok(())
}
