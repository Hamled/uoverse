use serde::ser::{Error, Serialize, Serializer};

#[derive(Clone, Copy)]
pub struct FixedStr<const LEN: usize> {
    str: [u8; LEN],
}

impl<const LEN: usize> Serialize for FixedStr<LEN> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if LEN > u16::MAX as usize {
            Err(Error::custom("FixedStr must have a length <= u16::MAX"))
        } else {
            serializer.serialize_bytes(&self.str)
        }
    }
}

impl<const LEN: usize> Default for FixedStr<LEN> {
    fn default() -> Self {
        Self { str: [0u8; LEN] }
    }
}

impl<const LEN: usize> From<&str> for FixedStr<LEN> {
    fn from(string: &str) -> Self {
        let mut fixed: Self = Default::default();
        let len = std::cmp::min(LEN, string.len());
        fixed.str[..len].copy_from_slice(&string.as_bytes()[..len]);

        fixed
    }
}
