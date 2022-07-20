use macros::packet;

#[packet(fixed(id = 0xFB, size = 1))]
pub struct ShowPublicContent {
    show: bool,
}
