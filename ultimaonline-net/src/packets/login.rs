use serde::ser::{Serialize, SerializeTuple, Serializer};
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

struct LoginRejection {
    reason: LoginRejectionReason,
}

impl Serialize for LoginRejection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_tuple(2)?;
        s.serialize_element(&[0x82u8][..])?;
        s.serialize_element(&self.reason)?;
        s.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ser::to_writer;
    #[test]
    fn serialize_login_rejection() {
        let rej_invalid = [0x82u8, 0];

        let mut packet: Vec<u8> = Vec::new();

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
