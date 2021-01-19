use crate::types::FixedStr;
use macros::packet;
use serde::Serialize;
use serde_repr::Serialize_repr;

#[allow(dead_code)]
#[derive(Serialize_repr)]
#[repr(u8)]
enum LoginRejectionReason {
    Invalid = 0,
    InUse = 1,
    Blocked = 2,
    BadPass = 3,
    Idle = 254,
    BadComm = 255,
}

#[packet(id = 0x82)]
struct LoginRejection {
    reason: LoginRejectionReason,
}

#[derive(Serialize)]
#[repr(C, packed(1))]
struct ServerInfo {
    index: u16,
    name: FixedStr<32>,
    fullness: u8,
    timezone: u8,
    ip_address: u32,
}

#[packet(id = 0xA8, var_size = true)]
struct ServerList {
    flags: u8,
    list: Vec<ServerInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;
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
                },
            )
            .expect("Failed to write packet");

            assert_eq!(packet.as_slice(), rej_invalid);
        }
    }

    mod server_list {
        use super::*;

        #[test]
        fn serialize() {
            let servers = vec![
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
            ];

            let server_list = include_bytes!("../../test/resources/ServerList.pkt");

            let mut packet = Vec::<u8>::new();
            to_writer(
                &mut packet,
                &ServerList {
                    flags: 0x5D,
                    list: servers,
                },
            )
            .expect("Failed to write packet");

            assert_eq!(packet.as_slice(), server_list);
        }
    }
}
