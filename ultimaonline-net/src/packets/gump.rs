use crate::types::Serial;
use macros::packet;

// TODO: Figure out if we should do something with this.
// It appears to signal that the HP status tracking UI was
// closed for a particular entity.
//
// Tracking such UI state on the server is less than ideal,
// but could be a way to reduce the volume of update packets.
#[packet(extended(id = 0x0C))]
pub struct CloseStatus {
    serial: Serial,
}
