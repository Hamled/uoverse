use macros::packet;

use crate::types::FixedStr;

#[packet(extended(id = 0x05))]
#[derive(Debug, PartialEq)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

#[packet(extended(id = 0x0B))]
#[derive(Debug, PartialEq)]
pub struct Language {
    pub lang: FixedStr<4>,
}

// TODO: Investigate whether ClassicUO is
// calculating the flags incorrectly.
// ModernUO ignores this entirely.
#[packet(extended(id = 0x0F))]
pub struct Flags {
    pub unknown_00: u8, // Always 0x0A
    pub flags: u32,     // Always 0xFFFFFFFF
}

#[packet(standard(id = 0xC8))]
pub struct ViewRange {
    pub range: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packets::{FromPacketData, Packet};
    use crate::ser::to_writer;
    mod window_size {
        use super::*;

        #[test]
        fn serialize() {
            let expected_bytes = [
                0xBFu8, 0x00, 0x0D, 0x00, 0x05, 0x00, 0x32, 0x47, 0xD5, 0x34, 0x93, 0x47, 0xDF,
            ];

            let mut packet = Vec::<u8>::new();
            to_writer(
                &mut packet,
                &Packet::<_>::from(&WindowSize {
                    width: 3295189,
                    height: 882067423,
                }),
            )
            .expect("Failed to write packet");

            assert_eq!(packet.as_slice(), expected_bytes);
        }

        #[test]
        fn deserialize() {
            let window_size = WindowSize {
                width: 345_729_057,
                height: 3_820_817_358,
            };

            let mut input: &[u8] = &[
                0xBFu8, 0x00, 0x0D, 0x00, 0x05, 0x14, 0x9b, 0x68, 0x21, 0xE3, 0xBD, 0x0B, 0xCE,
            ];

            let parsed = WindowSize::from_packet_data(&mut input).expect("Failed to parse packet");

            assert_eq!(parsed, window_size);
        }
    }

    mod language {
        use super::*;

        #[test]
        fn serialize() {
            let expected_bytes = [0xBFu8, 0x00, 0x09, 0x00, 0x0B, 0x45, 0x4E, 0x55, 0x00];

            let mut packet = Vec::<u8>::new();
            to_writer(
                &mut packet,
                &Packet::<_>::from(&Language { lang: "ENU".into() }),
            )
            .expect("Failed to write packet");

            assert_eq!(packet.as_slice(), expected_bytes);
        }

        #[test]
        fn deserialize() {
            let language = Language { lang: "RUS".into() };

            let mut input: &[u8] = &[0xBFu8, 0x00, 0x09, 0x00, 0x0B, 0x52, 0x55, 0x53, 0x00];

            let parsed = Language::from_packet_data(&mut input).expect("Failed to parse packet");

            assert_eq!(parsed, language);
        }
    }
}
