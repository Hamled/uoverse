use crate::types::{list::ListTerm, Direction, Graphic, Hue, Notoriety, Serial};
use macros::packet;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum EntityFlags {
    None = 0x00,
    Frozen = 0x01,
    Female = 0x02,
    Flying = 0x04,
    YellowBar = 0x08,
    IgnoreMobiles = 0x10,
    Movable = 0x20,
    WarMode = 0x40,
    Hidden = 0x80,
}

#[packet(standard(id = 0x4E))]
pub struct MobLightLevel {
    pub serial: Serial,
    pub level: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub serial: Serial,
    pub type_id: u16,
    pub layer: u8,
    pub hue: Hue,
}

#[packet(standard(id = 0x77))]
pub struct State {
    pub serial: Serial,
    pub body: Graphic,
    pub x: u16,
    pub y: u16,
    pub z: i8,
    pub direction: Direction,
    pub hue: Hue,
    pub flags: EntityFlags,
    pub notoriety: Notoriety,
}

#[packet(standard(id = 0x78, var_size = true))]
pub struct Appearance {
    pub state: State,
    pub items: ListTerm<Item, u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum QueryKind {
    Status = 0x4,
    Skills = 0x5,
}

#[packet(standard(id = 0x34))]
pub struct Query {
    pub unused: u32, // 0xEDEDEDED
    pub kind: QueryKind,
    pub serial: Serial,
}
