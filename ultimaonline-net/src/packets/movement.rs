use macros::packet;

use crate::types::{MovementRaw, Notoriety};

#[packet(fixed(id = 0x02, size = 6))]
pub struct Request {
    pub movement: MovementRaw,
    pub sequence: u8,
    pub auth_token: u32, // Unused
}

#[packet(fixed(id = 0x22, size = 2))]
pub struct Success {
    pub sequence: u8,
    pub notoriety: Notoriety,
}

#[packet(fixed(id = 0x21, size = 7))]
pub struct Reject {
    pub sequence: u8,
    pub x: u16,
    pub y: u16,
    pub movement: MovementRaw,
    pub z: u8,
}
