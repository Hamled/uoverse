use crate::types::{Direction, Graphic, Serial};
use macros::packet;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum BodyType {
    Empty,
    Monster,
    Sea,
    Animal,
    Human,
    Equipment,
}

#[packet(id = 0x1B)]
pub struct LoginConfirmation {
    pub serial: Serial,

    pub unknown_04: u32, // 0x00000000

    pub body: Graphic,
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub direction: Direction,

    pub unknown_10: u8,       // 0x00
    pub unknown_11: u32,      // 0xFFFFFFFF
    pub unknown_15: [u8; 14], // All zero
}

#[packet(id = 0xBF, var_size = true)]
pub struct MapChange {
    unknown_00: u16, // 0x0008
    map_id: u8,
}

#[packet(id = 0x55)]
pub struct LoginComplete;
