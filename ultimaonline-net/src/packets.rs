use serde::Serialize;
pub mod login;

#[derive(Serialize)]
struct Packet<'a, T> {
    id: u8,
    size: Option<u16>,
    contents: &'a T,
}

trait ToPacket<'a>
where
    Self: Sized,
{
    fn to_packet(&'a self) -> Packet<'a, Self>;
}
