use crate::types::Serial;
use macros::packet;

#[packet(fixed(id = 0x06, size = 4))]
pub struct ClickUse {
    serial: Serial,
}

#[packet(fixed(id = 0x09, size = 4))]
pub struct ClickLook {
    serial: Serial,
}
