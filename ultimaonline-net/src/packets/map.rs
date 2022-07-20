use macros::packet;

#[packet(var(id = 0xBF))]
pub struct MapChange {
    unknown_00: u16, // 0x0008
    map_id: u8,
}
