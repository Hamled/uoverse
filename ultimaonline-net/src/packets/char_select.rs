use crate::types::{CharIdentity, FixedStr, Graphic, Hue, List, Name};
use macros::packet;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[packet(standard(id = 0x91))]
pub struct GameLogin {
    pub seed: u32,
    pub username: FixedStr<30>,
    pub password: FixedStr<30>,
}

#[packet(standard(id = 0xB9))]
pub struct Features {
    pub flags: u32,
}

#[packet(standard(id = 0xA9, var_size = true))]
pub struct CharList {
    pub chars: List<CharInfo, 8>,
    pub cities: List<CityInfo, 8>,
    pub flags: u32,
    pub unknown_var1: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CharInfo {
    pub name: Name,
    pub unused: FixedStr<30>,
}

impl From<&str> for CharInfo {
    fn from(val: &str) -> Self {
        Self {
            name: val.into(),
            unused: Default::default(),
        }
    }
}

impl Default for CharInfo {
    fn default() -> Self {
        Self {
            name: Default::default(),
            unused: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MapLocation {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub id: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CityInfo {
    pub index: u8,
    pub city: FixedStr<32>,
    pub building: FixedStr<32>,
    pub location: MapLocation,
    pub description: i32,
    pub unknown_15: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum SkillType {
    Alchemy,
    Anatomy,
    AnimalLore,
    ItemID,
    ArmsLore,
    Parry,
    Begging,
    Blacksmith,
    Fletching,
    Peacemaking,
    Camping,
    Carpentry,
    Cartography,
    Cooking,
    DetectHidden,
    Discordance,
    EvalInt,
    Healing,
    Fishing,
    Forensics,
    Herding,
    Hiding,
    Provocation,
    Inscribe,
    Lockpicking,
    Magery,
    MagicResist,
    Tactics,
    Snooping,
    Musicianship,
    Poisoning,
    Archery,
    SpiritSpeak,
    Stealing,
    Tailoring,
    AnimalTaming,
    TasteID,
    Tinkering,
    Tracking,
    Veterinary,
    Swords,
    Macing,
    Fencing,
    Wrestling,
    Lumberjacking,
    Mining,
    Meditation,
    Stealth,
    RemoveTrap,
    Necromancy,
    Focus,
    Chivalry,
    Bushido,
    Ninjitsu,
    Spellweaving,
    Mysticism,
    Imbuing,
    Throwing,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkillChoice {
    ty: SkillType,
    val: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Profession {
    Warrior = 1,
    Magicians,
    Blacksmith,
    Necromancer,
    Paladin,
    Samurai,
    Ninja,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CharAppearance {
    hue: Hue,
    hair_hue: Hue,
    hair_graphic: Graphic,
    beard_hue: Hue,
    beard_graphic: Graphic,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Character {
    profession: Profession,

    unknown_01: [u8; 15], // Null

    identity: CharIdentity,
    strength: u8,
    dexterity: u8,
    intelligence: u8,
    skills: [SkillChoice; 4],
    appearance: CharAppearance,
}

#[packet(standard(id = 0xF8))]
pub struct CreateCharacter {
    unknown_00: u32, // 0xEDEDEDED
    unknown_04: u16, // 0xFFFF
    unknown_06: u16, // 0xFFFF
    unknown_08: u8,  // 0x00

    pub name: FixedStr<30>,

    unknown_27: u16, // 0x0000

    client_flags: u32,

    unknown_2e: u32, // 0x0001
    unknown_32: u32, // 0x0000

    character: Character,
    city: u16,

    unknown_5e: u16, // 0x0000

    slot: u16,
    client_ip: u32,
    shirt_hue: Hue,
    pants_hue: Hue,
}

#[packet(standard(id = 0xBD))]
pub struct VersionReq {
    pub unknown_00: u16, // 0x0003
}

#[packet(standard(id = 0xBD, var_size = true))]
#[derive(Debug, PartialEq)]
pub struct VersionResp {
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packets::{FromPacketData, Packet};

    mod version_resp {
        use super::*;

        #[test]
        fn round_trip() {
            let version = VersionResp {
                version: "1.2.3.4".to_string(),
            };

            let mut packet = Vec::<u8>::new();
            Packet::<_>::from(&version)
                .to_writer(&mut packet)
                .expect("Failed to write packet");

            let parsed = VersionResp::from_packet_data(&mut packet.as_slice())
                .expect("Failed to parse packet");

            assert_eq!(parsed, version);
        }
    }
}
