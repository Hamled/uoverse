use macros::packet;

#[packet(standard(id = 0xBF, var_size = true))]
pub struct MapChange {
    unknown_00: u16, // 0x0008
    map_id: u8,
}
