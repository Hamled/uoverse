use crate::types::{FixedStr, List};
use macros::packet;
use serde::{Deserialize, Serialize};

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
