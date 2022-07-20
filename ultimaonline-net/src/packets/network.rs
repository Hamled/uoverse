use macros::packet;

#[packet(fixed(id = 0x73, size = 1))]
pub struct PingReq {
    pub val: u8,
}

#[packet(fixed(id = 0x73, size = 1))]
pub struct PingAck {
    pub val: u8,
}
