use serde::de::{self, Deserialize, Deserializer, SeqAccess, Visitor};
use serde::ser::{self, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::convert::TryFrom;
use std::fmt;

pub mod list;
pub use list::List;

pub mod movement;
pub use movement::{Movement, MovementRaw};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FixedStr<const LEN: usize> {
    str: [u8; LEN],
}

impl<const LEN: usize> Serialize for FixedStr<LEN> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if LEN > u16::MAX as usize {
            Err(ser::Error::custom(
                "FixedStr must have a length <= u16::MAX",
            ))
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

impl<'a, const LEN: usize> TryFrom<&'a FixedStr<LEN>> for &'a str {
    type Error = std::str::Utf8Error;

    fn try_from(fixed: &'a FixedStr<LEN>) -> Result<Self, Self::Error> {
        std::str::from_utf8(&fixed.str)
    }
}

struct FixedStrVisitor<const LEN: usize>;

impl<'de, const LEN: usize> Visitor<'de> for FixedStrVisitor<LEN> {
    type Value = FixedStr<LEN>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_fmt(format_args!("a fixed-length string of {} bytes", LEN))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut val: FixedStr<LEN> = Default::default();

        for i in 0..seq.size_hint().unwrap_or(LEN) {
            let by = seq.next_element::<u8>()?;
            match by {
                Some(by) => val.str[i] = by,
                None => {
                    return Err(de::Error::custom(
                        "Missing 1 or more elements from FixedStr",
                    ))
                }
            }
        }

        Ok(val)
    }
}

impl<'de, const LEN: usize> Deserialize<'de> for FixedStr<LEN> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(LEN, FixedStrVisitor)
    }
}

pub type Serial = u32;

pub type Name = FixedStr<30>;

// Mobile appearance types
pub type Hue = u16;
pub type Graphic = u16;

#[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Direction {
    North = 0,
    Right,
    East,
    Down,
    South,
    Left,
    West,
    Up,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Notoriety {
    Innocent = 1,
    Ally,
    CanBeAttacked,
    Criminal,
    Enemy,
    Murderer,
    Invulnerable,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum CharIdentity {
    HumanMale = 2,
    HumanFemale,
    ElfMale,
    ElfFemale,
    GargoyleMale,
    GargoyleFemale,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Race {
    Human = 1,
    Elf,
    Gargoyle,
}
