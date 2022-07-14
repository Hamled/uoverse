use crate::types::{Direction, Graphic, Name, Race, Serial};
use macros::packet;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
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

#[packet(id = 0x55)]
pub struct LoginComplete;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Attribute {
    pub current: u16,
    pub maximum: u16,
}

pub type Stat = u16;
pub type Resistance = u16;

#[packet(id = 0x11, var_size = true)]
pub struct CharStatus {
    pub serial: Serial,
    pub name: Name,
    pub hitpoints: Attribute,
    pub renamable: bool,
    pub version: u8, // 0x06
    pub gender: bool,
    pub strength: Stat,
    pub dexterity: Stat,
    pub intelligence: Stat,
    pub stamina: Attribute,
    pub mana: Attribute,
    pub gold: u32,
    pub phys_resist: Resistance,
    pub weight: Attribute,
    pub race: Race,
    pub stat_cap: u16,
    pub follower_count: u8,
    pub follower_max: u8,
    pub fire_resist: Resistance,
    pub cold_resist: Resistance,
    pub poison_resist: Resistance,
    pub energy_resist: Resistance,
    pub luck: Stat,
    pub damage_min: u16,
    pub damage_max: u16,
    pub tithing_points: u32,

    // Age of Shadows stats
    pub aos_stats: [Stat; 15],
}
