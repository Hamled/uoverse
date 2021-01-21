use crate::types::FixedStr;
use macros::packet;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize, Deserialize)]
pub struct ClientVersion {
    major: u32,
    minor: u32,
    revision: u32,
    patch: u32,
}

impl std::fmt::Display for ClientVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.major, self.minor, self.revision, self.patch
        )
    }
}

#[packet(id = 0xEF)]
pub struct ClientHello {
    pub seed: u32,
    pub version: ClientVersion,
}

#[packet(id = 0x80)]
pub struct AccountLogin {
    pub username: FixedStr<30>,
    pub password: FixedStr<30>,
    unknown_3c: u8,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum LoginRejectionReason {
    Invalid = 0,
    InUse = 1,
    Blocked = 2,
    BadPass = 3,
    Idle = 254,
    BadComm = 255,
}

#[packet(id = 0x82)]
#[derive(Debug, PartialEq)]
pub struct LoginRejection {
    pub reason: LoginRejectionReason,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ServerInfo {
    pub index: u16,
    pub name: FixedStr<32>,
    pub fullness: u8,
    pub timezone: u8,
    pub ip_address: u32,
}

#[packet(id = 0xA8, var_size = true)]
#[derive(Debug, PartialEq)]
pub struct ServerList {
    pub flags: u8,
    pub list: Vec<ServerInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packets::{FromPacketData, ToPacket};
    use crate::ser::to_writer;
    mod login_rejection {
        use super::*;

        #[test]
        fn serialize() {
            let rej_invalid = [0x82u8, 0];

            let mut packet = Vec::<u8>::new();
            to_writer(
                &mut packet,
                &LoginRejection {
                    reason: LoginRejectionReason::Invalid,
                }
                .to_packet(),
            )
            .expect("Failed to write packet");

            assert_eq!(packet.as_slice(), rej_invalid);
        }

        #[test]
        fn deserialize() {
            let rej_blocked = LoginRejection {
                reason: LoginRejectionReason::Blocked,
            };

            let mut input: &[u8] = &[0x82u8, 2];

            let parsed =
                LoginRejection::from_packet_data(&mut input).expect("Failed to parse packet");

            assert_eq!(parsed, rej_blocked);
        }
    }

    mod server_list {
        use super::*;

        fn servers() -> Vec<ServerInfo> {
            vec![
                ServerInfo {
                    index: 0,
                    name: "Server 1".into(),
                    fullness: 10,
                    timezone: 3,
                    ip_address: 0x12345678,
                },
                ServerInfo {
                    index: 1,
                    name: "Another Server".into(),
                    fullness: 39,
                    timezone: 9,
                    ip_address: 0x09080706,
                },
            ]
        }

        #[test]
        fn serialize() {
            let server_list = include_bytes!("../../test/resources/ServerList.pkt");

            let mut packet = Vec::<u8>::new();
            to_writer(
                &mut packet,
                &ServerList {
                    flags: 0x5D,
                    list: servers(),
                }
                .to_packet(),
            )
            .expect("Failed to write packet");

            assert_eq!(packet.as_slice(), server_list);
        }

        #[test]
        fn deserialize() {
            let server_list = ServerList {
                flags: 0x5D,
                list: servers(),
            };

            let mut input: &[u8] = include_bytes!("../../test/resources/ServerList.pkt");

            let parsed = ServerList::from_packet_data(&mut input).expect("Failed to parse packet");

            assert_eq!(parsed, server_list);
        }
    }
}
