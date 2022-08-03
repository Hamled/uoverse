use macros::packet;

#[packet(extended(id = 0x08))]
pub struct MapChange {
    pub map_id: u8,
}
