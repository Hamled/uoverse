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

#[packet(fixed(id = 0x1B, size = 36))]
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

#[packet(fixed(id = 0x55, size = 0))]
pub struct LoginComplete;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Attribute {
    pub current: u16,
    pub maximum: u16,
}

pub type Stat = u16;
pub type Resistance = u16;

#[packet(var(id = 0x11))]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packets::{FromPacketData, Packet};
    use crate::ser::to_writer;
    mod login_complete {
        use super::*;

        #[test]
        fn serialize() {
            let expected_bytes = [0x55u8];

            let mut packet = Vec::<u8>::new();
            to_writer(&mut packet, &Packet::<_>::from(&LoginComplete {}))
                .expect("Failed to write packet");

            assert_eq!(packet.as_slice(), expected_bytes);
        }

        #[test]
        fn deserialize() {
            let login_complete = LoginComplete {};

            let mut input: &[u8] = &[0x55u8];

            let parsed =
                LoginComplete::from_packet_data(&mut input).expect("Failed to parse packet");

            assert_eq!(parsed, login_complete);
        }
    }
}
