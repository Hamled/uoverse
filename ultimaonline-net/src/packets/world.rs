use macros::packet;

#[packet(standard(id = 0x4F))]
pub struct WorldLightLevel {
    pub level: u8,
}
