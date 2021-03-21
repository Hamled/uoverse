use crate::types::{FixedStr, List};
use macros::packet;
use serde::{Deserialize, Serialize};

#[packet(id = 0x91)]
pub struct GameLogin {
    pub seed: u32,
    pub username: FixedStr<30>,
    pub password: FixedStr<30>,
}
