use macros::packet;

#[packet(id = 0x4F)]
pub struct WorldLightLevel {
    pub level: u8,
}
