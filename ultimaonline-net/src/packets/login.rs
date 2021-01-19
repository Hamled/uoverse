use macros::packet;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ser::to_writer;
    #[test]
    fn serialize_login_rejection() {
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
