use crate::types::{FixedStr, List};
use macros::packet;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[packet(id = 0x91)]
pub struct GameLogin {
    pub seed: u32,
    pub username: FixedStr<30>,
    pub password: FixedStr<30>,
}

#[packet(id = 0xB9)]
pub struct Features {
    pub flags: u32,
}

#[packet(id = 0xA9, var_size = true)]
pub struct CharList {
    pub chars: List<CharInfo, 8>,
    pub cities: List<CityInfo, 8>,
    pub flags: u32,
    pub unknown_var1: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CharInfo {
    pub name: FixedStr<30>,
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

#[derive(Serialize, Deserialize)]
pub struct MapLocation {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct CityInfo {
    pub index: u8,
    pub city: FixedStr<32>,
    pub building: FixedStr<32>,
    pub location: MapLocation,
    pub description: i32,
    pub unknown_15: i32,
}

#[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum CharIdentity {
    HumanMale = 2,
    HumanFemale,
    ElfMale,
    ElfFemale,
    GargoyleMale,
    GargoyleFemale,
}

#[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr)]
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

#[derive(Serialize, Deserialize)]
pub struct SkillChoice {
    ty: SkillType,
    val: u8,
}

#[derive(Serialize_repr, Deserialize_repr)]
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

// Character appearance types
type Hue = u16;
type Graphic = u16;

#[derive(Serialize, Deserialize)]
pub struct CharAppearance {
    hue: Hue,
    hair_hue: Hue,
    hair_graphic: Graphic,
    beard_hue: Hue,
    beard_graphic: Graphic,
}

#[derive(Serialize, Deserialize)]
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

#[packet(id = 0xF8)]
pub struct CreateCharacter {
    unknown_00: u32, // 0xEDEDEDED
    unknown_04: u16, // 0xFFFF
    unknown_06: u16, // 0xFFFF
    unknown_08: u8,  // 0x00

    name: FixedStr<30>,

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
