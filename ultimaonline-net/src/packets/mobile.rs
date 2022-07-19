use crate::types::{list::ListTerm, Direction, Graphic, Hue, Notoriety, Serial};
use macros::packet;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum EntityFlags {
    None = 0x00,
    Frozen = 0x01,
    Female = 0x02,
    Flying = 0x04,
    YellowBar = 0x08,
    IgnoreMobiles = 0x10,
    Movable = 0x20,
    WarMode = 0x40,
    Hidden = 0x80,
}

#[packet(standard(id = 0x4E))]
pub struct MobLightLevel {
    pub serial: Serial,
    pub level: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub serial: Serial,
    pub type_id: u16,
    pub layer: u8,
    pub hue: Hue,
}

#[packet(standard(id = 0x77))]
#[derive(Debug, PartialEq)]
pub struct State {
    pub serial: Serial,
    pub body: Graphic,
    pub x: u16,
    pub y: u16,
    pub z: i8,
    pub direction: Direction,
    pub hue: Hue,
    pub flags: EntityFlags,
    pub notoriety: Notoriety,
}

#[packet(standard(id = 0x78, var_size = true))]
#[derive(Debug, PartialEq)]
pub struct Appearance {
    pub state: State,
    pub items: ListTerm<Item, u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum QueryKind {
    Status = 0x4,
    Skills = 0x5,
}

#[packet(standard(id = 0x34))]
pub struct Query {
    pub unused: u32, // 0xEDEDEDED
    pub kind: QueryKind,
    pub serial: Serial,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packets::{FromPacketData, Packet};
    use crate::ser::to_writer;
    mod appearance {
        use super::*;

        fn appearance() -> Appearance {
            Appearance {
                state: State {
                    serial: 0x12345678,
                    body: 0xBEEF,
                    x: 0xABCD,
                    y: 0xACAB,
                    z: 0x7F,
                    direction: Direction::North,
                    hue: 0xDEAD,
                    flags: EntityFlags::Hidden,
                    notoriety: Notoriety::Innocent,
                },
                items: vec![
                    Item {
                        serial: 0x40000001,
                        type_id: 0x1A1A,
                        layer: 0x1B,
                        hue: 0x1C1C,
                    },
                    Item {
                        serial: 0x40000002,
                        type_id: 0x2A2A,
                        layer: 0x2B,
                        hue: 0x2C2C,
                    },
                    Item {
                        serial: 0x40000003,
                        type_id: 0x3A3A,
                        layer: 0x3B,
                        hue: 0x3C3C,
                    },
                    Item {
                        serial: 0x40000004,
                        type_id: 0x4A4A,
                        layer: 0x4B,
                        hue: 0x4C4C,
                    },
                    Item {
                        serial: 0x40000005,
                        type_id: 0x5A5A,
                        layer: 0x5B,
                        hue: 0x5C5C,
                    },
                    Item {
                        serial: 0x40000006,
                        type_id: 0x6A6A,
                        layer: 0x6B,
                        hue: 0x6C6C,
                    },
                    Item {
                        serial: 0x40000007,
                        type_id: 0x7A7A,
                        layer: 0x7B,
                        hue: 0x7C7C,
                    },
                    Item {
                        serial: 0x40000008,
                        type_id: 0x8A8A,
                        layer: 0x8B,
                        hue: 0x8C8C,
                    },
                ]
                .into(),
            }
        }

        #[test]
        fn serialize() {
            let expected_bytes = [
                0x78u8, 0x00, 0x5F, 0x12, 0x34, 0x56, 0x78, 0xBE, 0xEF, 0xAB, 0xCD, 0xAC, 0xAB,
                0x7F, 0x00, 0xDE, 0xAD, 0x80, 0x01, 0x40, 0x00, 0x00, 0x01, 0x1A, 0x1A, 0x1B, 0x1C,
                0x1C, 0x40, 0x00, 0x00, 0x02, 0x2A, 0x2A, 0x2B, 0x2C, 0x2C, 0x40, 0x00, 0x00, 0x03,
                0x3A, 0x3A, 0x3B, 0x3C, 0x3C, 0x40, 0x00, 0x00, 0x04, 0x4A, 0x4A, 0x4B, 0x4C, 0x4C,
                0x40, 0x00, 0x00, 0x05, 0x5A, 0x5A, 0x5B, 0x5C, 0x5C, 0x40, 0x00, 0x00, 0x06, 0x6A,
                0x6A, 0x6B, 0x6C, 0x6C, 0x40, 0x00, 0x00, 0x07, 0x7A, 0x7A, 0x7B, 0x7C, 0x7C, 0x40,
                0x00, 0x00, 0x08, 0x8A, 0x8A, 0x8B, 0x8C, 0x8C, 0x00, 0x00, 0x00, 0x00,
            ];

            let mut packet = Vec::<u8>::new();
            to_writer(&mut packet, &Packet::<_>::from(&appearance()))
                .expect("Failed to write packet");

            assert_eq!(packet.as_slice(), expected_bytes);
        }

        #[test]
        fn deserialize() {
            let appearance = appearance();

            let mut input: &[u8] = &[
                0x78u8, 0x00, 0x5F, 0x12, 0x34, 0x56, 0x78, 0xBE, 0xEF, 0xAB, 0xCD, 0xAC, 0xAB,
                0x7F, 0x00, 0xDE, 0xAD, 0x80, 0x01, 0x40, 0x00, 0x00, 0x01, 0x1A, 0x1A, 0x1B, 0x1C,
                0x1C, 0x40, 0x00, 0x00, 0x02, 0x2A, 0x2A, 0x2B, 0x2C, 0x2C, 0x40, 0x00, 0x00, 0x03,
                0x3A, 0x3A, 0x3B, 0x3C, 0x3C, 0x40, 0x00, 0x00, 0x04, 0x4A, 0x4A, 0x4B, 0x4C, 0x4C,
                0x40, 0x00, 0x00, 0x05, 0x5A, 0x5A, 0x5B, 0x5C, 0x5C, 0x40, 0x00, 0x00, 0x06, 0x6A,
                0x6A, 0x6B, 0x6C, 0x6C, 0x40, 0x00, 0x00, 0x07, 0x7A, 0x7A, 0x7B, 0x7C, 0x7C, 0x40,
                0x00, 0x00, 0x08, 0x8A, 0x8A, 0x8B, 0x8C, 0x8C, 0x00, 0x00, 0x00, 0x00,
            ];

            println!("Packet len: {:x}", input.len());

            let parsed = Appearance::from_packet_data(&mut input).expect("Failed to parse packet");

            assert_eq!(parsed, appearance);
        }
    }
}
