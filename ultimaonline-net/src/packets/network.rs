use macros::packet;

#[packet(standard(id = 0x73))]
pub struct PingReq {
    pub val: u8,
}

#[packet(standard(id = 0x73))]
pub struct PingAck {
    pub val: u8,
}
