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
