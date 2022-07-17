use macros::packet;

// TODO: Figure out if this will have actual content
// ModernUO implementation says it doesn't.
#[packet(standard(id = 0xB5))]
pub struct OpenWindow {
    pub unused_00: [u8; 0x20], // All zeros?
    pub unused_20: [u8; 0x1F], // All zeros?
}
