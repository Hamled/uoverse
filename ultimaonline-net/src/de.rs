use crate::error::{Error, Result};
use byteorder::{BigEndian, ReadBytesExt};
use serde::{
    de::{self, Visitor},
    Deserialize,
};
use std::{convert::TryInto, io, str};

pub struct Deserializer<'a, R>
where
    R: io::BufRead,
{
    reader: &'a mut R,
    peek: bool,
    remaining: usize,
}

pub fn from_reader<'a, R, T>(reader: &'a mut R, size: usize) -> Result<T>
where
    R: io::BufRead,
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer {
        reader,
        peek: false,
        remaining: size,
    };

    let t = T::deserialize(&mut deserializer)?;

    match deserializer.remaining {
        0 => Ok(t),
        _ => Err(Error::de("data remains after deserializing value")),
    }
}

macro_rules! impl_read_literal {
    ($name:ident : $ty:ty = $read_func:ident()) => {
        #[inline]
        fn $name(&mut self) -> Result<$ty> {
            if self.peek {
                let buf = self.reader.fill_buf()?;
                if buf.len() < ::core::mem::size_of::<$ty>() {
                    Err(Self::insufficient_buffer::<$ty>())
                } else {
                    Ok(unsafe {
                        <$ty>::from_be_bytes(
                            buf[..::core::mem::size_of::<$ty>()]
                                .try_into()
                                .unwrap_unchecked(),
                        )
                    })
                }
            } else {
                let val = self.reader.$read_func::<BigEndian>()?;
                self.track_read(::core::mem::size_of::<$ty>())?;

                Ok(val)
            }
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

    fn insufficient_buffer<T>() -> Error {
        io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("insufficient buffer for {}", std::any::type_name::<T>()),
        )
        .into()
    }

    fn track_read(&mut self, amount: usize) -> Result<()> {
        self.remaining = self
            .remaining
            .checked_sub(amount)
            .ok_or(Error::de("read past end of serialized value"))?;
        Ok(())
    }
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
        let val = if self.peek {
            let buf = self.reader.fill_buf()?;
            if buf.is_empty() {
                return Err(Deserializer::<'de, R>::insufficient_buffer::<bool>());
            }
            buf[0]
        } else {
            let val = self.reader.read_u8()?;
            self.track_read(core::mem::size_of::<bool>())?;
            val
        };

        visitor.visit_bool(if val == 0 { false } else { true })
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let val = if self.peek {
            let buf = self.reader.fill_buf()?;
            if buf.is_empty() {
                return Err(Deserializer::<'de, R>::insufficient_buffer::<u8>());
            }
            buf[0]
        } else {
            let val = self.reader.read_u8()?;
            self.track_read(core::mem::size_of::<u8>())?;
            val
        };

        visitor.visit_u8(val)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let val = if self.peek {
            let buf = self.reader.fill_buf()?;
            if buf.is_empty() {
                return Err(Deserializer::<'de, R>::insufficient_buffer::<i8>());
            }
            buf[0] as i8
        } else {
            let val = self.reader.read_i8()?;
            self.track_read(core::mem::size_of::<i8>())?;
            val
        };

        visitor.visit_i8(val)
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
        if self.peek {
            unimplemented!();
        }

        // TODO: Make a zero-copy version of this if possible
        let mut buffer = vec![];
        loop {
            let byte = self.reader.read_u8()?;
            match byte {
                0 => break,
                n => buffer.push(n),
            }
        }

        self.track_read(buffer.len() + 1)?;

        let s =
            str::from_utf8(&buffer).map_err(|_| Error::data("string data could not be parsed"))?;
        // We don't support UTF-8
        if !s.is_ascii() {
            return Err(Error::data("non-ASCII string encoding is unsupported"));
        }

        visitor.visit_str(s)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.peek {
            unimplemented!();
        }

        let mut buffer = vec![];
        loop {
            let byte = self.reader.read_u8()?;
            match byte {
                0 => break,
                n => buffer.push(n),
            }
        }

        self.track_read(buffer.len() + 1)?;

        let s = String::from_utf8(buffer)
            .map_err(|_| Error::data("string data could not be parsed"))?;
        // We don't support UTF-8
        if !s.is_ascii() {
            return Err(Error::data("non-ASCII string encoding is unsupported"));
        }

        visitor.visit_string(s)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
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
        struct Access<'de, 'a, R: io::BufRead> {
            deserializer: &'a mut Deserializer<'de, R>,
        }

        impl<'de, 'a, R: io::BufRead> de::SeqAccess<'de> for Access<'de, 'a, R> {
            type Error = Error;

            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
            where
                T: de::DeserializeSeed<'de>,
            {
                match self.deserializer.remaining {
                    0 => Ok(None),
                    _ => Ok(Some(de::DeserializeSeed::deserialize(
                        seed,
                        &mut *self.deserializer,
                    )?)),
                }
            }

            fn size_hint(&self) -> Option<usize> {
                None
            }
        }

        visitor.visit_seq(Access { deserializer: self })
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // HACK: We only support enums for TermList elements
        visitor.visit_enum(TerminatorEnum { deserializer: self })
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

struct TerminatorEnum<'de, 'a, R: io::BufRead> {
    deserializer: &'a mut Deserializer<'de, R>,
}

impl<'de, 'a, R: io::BufRead> de::EnumAccess<'de> for TerminatorEnum<'de, 'a, R> {
    type Error = Error;
    type Variant = TerminatorVariant<'de, 'a, R>;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant)>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.deserializer.peek = true;
        let val = seed.deserialize(&mut *self.deserializer)?;
        self.deserializer.peek = false;

        Ok((
            val,
            TerminatorVariant {
                deserializer: self.deserializer,
                terminator_size: core::mem::size_of::<T::Value>(),
            },
        ))
    }
}

struct TerminatorVariant<'de, 'a, R: io::BufRead> {
    deserializer: &'a mut Deserializer<'de, R>,
    terminator_size: usize,
}

impl<'de, 'a, R: io::BufRead> de::VariantAccess<'de> for TerminatorVariant<'de, 'a, R> {
    type Error = Error;

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.deserializer)
    }

    fn unit_variant(self) -> Result<()> {
        // This was a terminator variant, consume the bytes
        self.deserializer.reader.consume(self.terminator_size);
        self.deserializer.track_read(self.terminator_size)?;

        Ok(())
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }
}
