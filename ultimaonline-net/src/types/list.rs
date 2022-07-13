use serde::de::{self, Deserialize, Deserializer, SeqAccess, Visitor};
use serde::ser::{self, Serialize, SerializeStruct, Serializer};
use std::fmt;
use std::marker::PhantomData;

#[derive(Clone, Debug, PartialEq)]
pub struct List<T, const LEN_BITS: usize>(Vec<T>);

impl<T: Serialize, const LEN_BITS: usize> Serialize for List<T, LEN_BITS> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut struct_ser = serializer.serialize_struct("List", 2)?;

        let len = self.0.len();
        match LEN_BITS {
            8 => struct_ser.serialize_field("length", &(len as u8))?,
            16 => struct_ser.serialize_field("length", &(len as u16))?,
            32 => struct_ser.serialize_field("length", &(len as u32))?,
            64 => struct_ser.serialize_field("length", &(len as u64))?,
            _ => {
                return Err(ser::Error::custom(
                    "List LEN_BITS must be one of: 8, 16, 32, 64",
                ))
            }
        };

        struct_ser.serialize_field("elements", &self.0)?;
        struct_ser.end()
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

        if let Some(elements) = seq.next_element::<Vec<T>>()? {
            val.0 = elements;
            Ok(val)
        } else {
            Err(de::Error::custom("Missing 1 or more elements from List"))
        }
    }
}

impl<'de, T: 'de + Deserialize<'de>, const LEN_BITS: usize> Deserialize<'de> for List<T, LEN_BITS> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] = &["length", "elements"];
        deserializer.deserialize_struct(
            "List",
            FIELDS,
            ListVisitor {
                element_type: PhantomData,
            },
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ListTerm<T, const TERM_BITS: usize>(Vec<T>);

impl<T: Serialize, const TERM_BITS: usize> Serialize for ListTerm<T, TERM_BITS> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut struct_ser = serializer.serialize_struct("ListTerm", 2)?;

        struct_ser.serialize_field("elements", &self.0)?;

        // Serialize a null terminator with a sized based on TERM_BITS
        match TERM_BITS {
            8 => struct_ser.serialize_field("terminator", &(0 as u8))?,
            16 => struct_ser.serialize_field("terminator", &(0 as u16))?,
            32 => struct_ser.serialize_field("terminator", &(0 as u32))?,
            64 => struct_ser.serialize_field("terminator", &(0 as u64))?,
            _ => {
                return Err(ser::Error::custom(
                    "ListTerm TERM_BITS must be one of: 8, 16, 32, 64",
                ))
            }
        };

        struct_ser.end()
    }
}

impl<T, const TERM_BITS: usize> Default for ListTerm<T, TERM_BITS> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T, const TERM_BITS: usize> From<Vec<T>> for ListTerm<T, TERM_BITS> {
    fn from(val: Vec<T>) -> Self {
        Self(val)
    }
}

impl<T, const TERM_BITS: usize> From<ListTerm<T, TERM_BITS>> for Vec<T> {
    fn from(val: ListTerm<T, TERM_BITS>) -> Self {
        val.0
    }
}

struct ListTermVisitor<'de, T: Deserialize<'de>, const TERM_BITS: usize> {
    element_type: PhantomData<&'de T>,
}

impl<'de, T: Deserialize<'de>, const TERM_BITS: usize> Visitor<'de>
    for ListTermVisitor<'de, T, TERM_BITS>
{
    type Value = ListTerm<T, TERM_BITS>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_fmt(format_args!(
            "a list terminated by a {}-bit null value",
            TERM_BITS
        ))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut val: ListTerm<T, TERM_BITS> = Default::default();
        loop {
            let term = match TERM_BITS {
                8 => seq.next_element::<u8>()?.and_then(|t| Some(t as usize)),
                16 => seq.next_element::<u16>()?.and_then(|t| Some(t as usize)),
                32 => seq.next_element::<u32>()?.and_then(|t| Some(t as usize)),
                64 => seq.next_element::<u64>()?.and_then(|t| Some(t as usize)),
                _ => {
                    return Err(de::Error::custom(
                        "ListTerm TERM_BITS must be one of: 8, 16, 32, 64",
                    ))
                }
            };

            match term {
                Some(0) => break,
                _ => {
                    if let Some(e) = seq.next_element::<T>()? {
                        val.0.push(e);
                    } else {
                        return Err(de::Error::custom(
                            "Unable to deserialize element from ListTerm",
                        ));
                    }
                }
            }
        }

        Ok(val)
    }
}

impl<'de, T: 'de + Deserialize<'de>, const TERM_BITS: usize> Deserialize<'de>
    for ListTerm<T, TERM_BITS>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ListTermVisitor {
            element_type: PhantomData,
        })
    }
}
