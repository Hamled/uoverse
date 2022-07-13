use crate::types::Serial;
use macros::packet;

#[packet(id = 0x4E)]
pub struct MobLightLevel {
    pub serial: Serial,
    pub level: u8,
}
