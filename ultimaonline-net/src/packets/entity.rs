use crate::types::{list::ListNonTerm, Serial};
use macros::packet;

#[packet(var(id = 0xD6))]
pub struct EntityBatchQuery {
    pub serials: ListNonTerm<Serial>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packets::{FromPacketData, Packet};
    use crate::ser::to_writer;
    mod entity_batch_query {
        use super::*;

        #[test]
        fn serialize() {
            let expected_bytes = [0xD6u8, 0x00, 0x07, 0x40, 0x00, 0x00, 0x32];

            let mut packet = Vec::<u8>::new();
            to_writer(
                &mut packet,
                &Packet::<_>::from(&EntityBatchQuery {
                    serials: vec![0x40000032].into(),
                }),
            )
            .expect("Failed to write packet");

            assert_eq!(packet.as_slice(), expected_bytes);
        }

        #[test]
        fn deserialize() {
            let batch_query = EntityBatchQuery {
                serials: vec![0x40000096, 0x40000544].into(),
            };

            let mut input: &[u8] = &[
                0xD6u8, 0x00, 0x0B, 0x40, 0x00, 0x00, 0x96, 0x40, 0x00, 0x05, 0x44,
            ];

            let parsed =
                EntityBatchQuery::from_packet_data(&mut input).expect("Failed to parse packet");

            assert_eq!(parsed, batch_query);
        }
    }
}
