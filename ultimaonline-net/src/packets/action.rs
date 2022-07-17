use crate::types::Serial;
use macros::packet;

#[packet(standard(id = 0x06))]
pub struct ClickUse {
    serial: Serial,
}

#[packet(standard(id = 0x09))]
pub struct ClickLook {
    serial: Serial,
}
