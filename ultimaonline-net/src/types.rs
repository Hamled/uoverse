use serde::de::{self, Deserialize, Deserializer, SeqAccess, Visitor};
use serde::ser::{self, Serialize, SerializeTuple, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::convert::TryFrom;
use std::fmt;
use std::marker::PhantomData;

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

#[derive(Clone, Debug, PartialEq)]
pub struct List<T, const LEN_BITS: usize>(Vec<T>);

impl<T: Serialize, const LEN_BITS: usize> Serialize for List<T, LEN_BITS> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // HACK: This is incredibly scuffed, but since serde doesn't really
        // want to support serializing sequences with different "metadata"
        // structures, like a length prefix that's of a different size depending
        // on the list type... we fake like this is a tuple, with the length
        // as the first element.
        let len = self.0.len();
        let mut tuple_ser = serializer.serialize_tuple(len)?; // Len gets ignored here

        match LEN_BITS {
            8 => tuple_ser.serialize_element(&(len as u8))?,
            16 => tuple_ser.serialize_element(&(len as u16))?,
            32 => tuple_ser.serialize_element(&(len as u32))?,
            64 => tuple_ser.serialize_element(&(len as u64))?,
            _ => {
                return Err(ser::Error::custom(
                    "List LEN_BITS must be one of: 8, 16, 32, 64",
                ))
            }
        };

        for e in &self.0 {
            tuple_ser.serialize_element(&e)?;
        }
        tuple_ser.end()
    }
}

impl<T, const LEN_BITS: usize> Default for List<T, LEN_BITS> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T, const LEN_BITS: usize> From<Vec<T>> for List<T, LEN_BITS> {
    fn from(val: Vec<T>) -> Self {
        Self(val)
    }
}

impl<T, const LEN_BITS: usize> From<List<T, LEN_BITS>> for Vec<T> {
    fn from(val: List<T, LEN_BITS>) -> Self {
        val.0
    }
}

struct ListVisitor<'de, T: Deserialize<'de>, const LEN: usize> {
    element_type: PhantomData<&'de T>,
}

impl<'de, T: Deserialize<'de>, const LEN_BITS: usize> Visitor<'de>
    for ListVisitor<'de, T, LEN_BITS>
{
    type Value = List<T, LEN_BITS>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_fmt(format_args!(
            "a list prefixed with {}-bit length value",
            LEN_BITS
        ))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut val: List<T, LEN_BITS> = Default::default();

        let len = match LEN_BITS {
            8 => seq.next_element::<u8>()?.map(|len| len as usize),
            16 => seq.next_element::<u16>()?.map(|len| len as usize),
            32 => seq.next_element::<u32>()?.map(|len| len as usize),
            64 => seq.next_element::<u64>()?.map(|len| len as usize),
            _ => {
                return Err(de::Error::custom(
                    "List LEN_BITS must be one of: 8, 16, 32, 64",
                ))
            }
        };

        if len.is_none() {
            return Err(de::Error::custom(format!(
                "List<T, {}> was not prefixed with length value",
                LEN_BITS
            )));
        }

        let len = len.unwrap();
        for _ in 0..len {
            let e = seq.next_element::<T>()?;
            match e {
                Some(e) => val.0.push(e),
                None => return Err(de::Error::custom("Missing 1 or more elements from List")),
            }
        }

        Ok(val)
    }
}

impl<'de, T: 'de + Deserialize<'de>, const LEN_BITS: usize> Deserialize<'de> for List<T, LEN_BITS> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(
            0, // This will be ignored
            ListVisitor {
                element_type: PhantomData,
            },
        )
    }
}

pub type Serial = u32;

pub type Name = FixedStr<30>;

// Mobile appearance types
pub type Hue = u16;
pub type Graphic = u16;

#[derive(Serialize_repr, Deserialize_repr)]
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

    Running = 0x80,
}

#[derive(Serialize_repr, Deserialize_repr)]
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

#[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum CharIdentity {
    HumanMale = 2,
    HumanFemale,
    ElfMale,
    ElfFemale,
    GargoyleMale,
    GargoyleFemale,
}

#[derive(Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Race {
    Human = 1,
    Elf,
    Gargoyle,
}
