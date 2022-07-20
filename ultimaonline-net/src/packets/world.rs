use macros::packet;

#[packet(fixed(id = 0x4F, size = 1))]
pub struct WorldLightLevel {
    pub level: u8,
}
