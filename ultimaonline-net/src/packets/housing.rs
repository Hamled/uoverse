use macros::packet;

#[packet(standard(id = 0xFB))]
pub struct ShowPublicContent {
    show: bool,
}
