use crate::error::{Error, Result};
use byteorder::{BigEndian, ReadBytesExt};
use serde::{
    de::{self, Visitor},
    Deserialize,
};
use std::{io, str};

pub struct Deserializer<'a, R>
where
    R: io::BufRead,
{
    reader: &'a mut R,
}

pub fn from_reader<'a, R, T>(reader: &'a mut R) -> Result<T>
where
    R: io::BufRead,
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer { reader };
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

macro_rules! impl_read_literal {
    ($name:ident : $ty:ty = $read_func:ident()) => {
        #[inline]
        fn $name(&mut self) -> Result<$ty> {
            self.reader.$read_func::<BigEndian>().map_err(Error::io)
        }
    };
}

impl<R> Deserializer<'_, R>
where
    R: io::BufRead,
{
    impl_read_literal!(read_u16: u16 = read_u16());
    impl_read_literal!(read_i16: i16 = read_i16());
    impl_read_literal!(read_u32: u32 = read_u32());
    impl_read_literal!(read_i32: i32 = read_i32());
    impl_read_literal!(read_u64: u64 = read_u64());
    impl_read_literal!(read_i64: i64 = read_i64());
    impl_read_literal!(read_f32: f32 = read_f32());
    impl_read_literal!(read_f64: f64 = read_f64());
}

// TODO: Make the deserialization process perform less copying

impl<'de, 'a, R> de::Deserializer<'de> for &'a mut Deserializer<'de, R>
where
    R: io::BufRead,
{
    type Error = Error;

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0u8; 1];
        self.reader.read(&mut buf).map_err(Error::io)?;

        let res = if buf[0] == 0 { false } else { true };

        visitor.visit_bool(res)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.reader.read_u8().map_err(Error::io)?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.reader.read_i8().map_err(Error::io)?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.read_u16()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.read_i16()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.read_u32()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.read_i32()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.read_u64()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.read_i64()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.read_f32()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.read_f64()?)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // TODO: Make a zero-copy version of this if possible
        let mut buffer = vec![];
        loop {
            let byte = self.reader.read_u8().map_err(Error::io)?;
            match byte {
                0 => break,
                n => buffer.push(n),
            }
        }

        let s = str::from_utf8(&buffer).map_err(|_| Error::data("Could not parse string"))?;
        // We don't support UTF-8
        if !s.is_ascii() {
            return Err(Error::data("Unsupported string encoding"));
        }

        visitor.visit_str(s)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut buffer = vec![];
        loop {
            let byte = self.reader.read_u8().map_err(Error::io)?;
            match byte {
                0 => break,
                n => buffer.push(n),
            }
        }

        let s = String::from_utf8(buffer).map_err(|_| Error::data("Could not parse string"))?;
        // We don't support UTF-8
        if !s.is_ascii() {
            return Err(Error::data("Unsupported string encoding"));
        }

        visitor.visit_string(s)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Adapted from serde_bincode
        struct Access<'de, 'a, R: io::BufRead> {
            deserializer: &'a mut Deserializer<'de, R>,
            len: usize,
        }

        impl<'de, 'a, R: io::BufRead> de::SeqAccess<'de> for Access<'de, 'a, R> {
            type Error = Error;

            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
            where
                T: de::DeserializeSeed<'de>,
            {
                if self.len > 0 {
                    self.len -= 1;
                    let value = de::DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }

            fn size_hint(&self) -> Option<usize> {
                Some(self.len)
            }
        }

        visitor.visit_seq(Access {
            deserializer: self,
            len,
        })
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.read_u16()?;
        self.deserialize_tuple(len as usize, visitor)
    }

    // Unimplemented parts of the Serde data model

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}
