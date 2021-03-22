use crate::error::{Error, Result};
use core::mem::size_of;
use serde::{ser, Serialize};
use std::io;

pub struct Serializer<'a, W>
where
    W: io::Write,
{
    size: usize,
    writer: Option<&'a mut W>,
}

#[inline]
pub fn to_size<'a, T>(value: &'a T) -> Result<usize>
where
    T: Serialize,
{
    let mut serializer = Serializer::<Vec<u8>> {
        size: 0,
        writer: None,
    };
    value.serialize(&mut serializer)?;

    Ok(serializer.size)
}

#[inline]
pub fn to_writer<'a, W, T>(writer: &'a mut W, value: &'a T) -> Result<()>
where
    W: io::Write,
    T: Serialize,
{
    let mut serializer = Serializer {
        size: 0,
        writer: Some(writer),
    };
    value.serialize(&mut serializer)?;

    Ok(())
}

impl<'a, 'b, W> ser::Serializer for &'a mut Serializer<'b, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;

    // Unimplemented
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.size += size_of::<bool>();
        if let Some(writer) = &mut self.writer {
            writer.write_all(&[v as u8][..]).map_err(Error::io)
        } else {
            Ok(())
        }
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.size += size_of::<u8>();
        if let Some(writer) = &mut self.writer {
            writer.write_all(&[v][..]).map_err(Error::io)
        } else {
            Ok(())
        }
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_u8(v as u8)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.size += size_of::<u16>();
        if let Some(writer) = &mut self.writer {
            writer.write_all(&v.to_be_bytes()).map_err(Error::io)
        } else {
            Ok(())
        }
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_u16(v as u16)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.size += size_of::<u32>();
        if let Some(writer) = &mut self.writer {
            writer.write_all(&v.to_be_bytes()).map_err(Error::io)
        } else {
            Ok(())
        }
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_u32(v as u32)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.size += size_of::<u64>();
        if let Some(writer) = &mut self.writer {
            writer.write_all(&v.to_be_bytes()).map_err(Error::io)
        } else {
            Ok(())
        }
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.size += size_of::<f32>();
        if let Some(writer) = &mut self.writer {
            writer.write_all(&v.to_be_bytes()).map_err(Error::io)
        } else {
            Ok(())
        }
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.size += size_of::<f64>();
        if let Some(writer) = &mut self.writer {
            writer.write_all(&v.to_be_bytes()).map_err(Error::io)
        } else {
            Ok(())
        }
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.size += v.len();
        if let Some(writer) = &mut self.writer {
            writer.write_all(v).map_err(Error::io)
        } else {
            Ok(())
        }
    }

    fn serialize_char(self, v: char) -> Result<()> {
        // We don't support serializing a single character to multiple bytes
        if v.is_ascii() {
            self.size += size_of::<u8>();

            let mut buf = [0u8; 1];
            v.encode_utf8(&mut buf);
            if let Some(writer) = &mut self.writer {
                writer.write_all(&buf).map_err(Error::io)
            } else {
                Ok(())
            }
        } else {
            Err(Error::Data)
        }
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        // We don't support UTF-8 strings
        if v.is_ascii() {
            if let Some(writer) = &mut self.writer {
                writer.write_all(v.as_bytes()).map_err(Error::io)?;
                writer.write_all(&[0u8][..]).map_err(Error::io)
            } else {
                Ok(())
            }
        } else {
            Err(Error::Data)
        }
    }

    fn serialize_none(self) -> Result<()> {
        Ok(())
    }

    fn serialize_some<T>(self, v: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        v.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        if let Some(len) = len {
            if len > u16::MAX as usize {
                return Err(Error::Data);
            } else {
                self.size += size_of::<u16>();
                if let Some(writer) = &mut self.writer {
                    writer
                        .write_all(&(len as u16).to_be_bytes())
                        .map_err(Error::io)?;
                }
            }
        }
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        // Nothing to be sent
        Ok(())
    }

    // Lots of stuff unimplemented as it's not needed

    fn serialize_unit(self) -> Result<()> {
        unimplemented!()
    }

    fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<()> {
        unimplemented!()
    }

    fn serialize_newtype_struct<T>(self, _: &'static str, _: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn serialize_newtype_variant<T>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unimplemented!()
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap> {
        unimplemented!()
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct> {
        unimplemented!()
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'a, 'b, W> ser::SerializeSeq for &'a mut Serializer<'b, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    // Standard ending without any null terminator
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<W> Serializer<'_, W>
where
    W: io::Write,
{
    fn end_null(&mut self) -> Result<()> {
        self.end_terminator(&[0u8][..])
    }

    fn end_terminator(&mut self, terminator: &[u8]) -> Result<()> {
        self.size += terminator.len();
        if let Some(writer) = &mut self.writer {
            writer.write_all(terminator).map_err(Error::io)
        } else {
            Ok(())
        }
    }
}

impl<'a, 'b, W> ser::SerializeTuple for &'a mut Serializer<'b, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    // Standard ending without any null terminator
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, 'b, W> ser::SerializeTupleStruct for &'a mut Serializer<'b, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    // Standard ending without any null terminator
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, 'b, W> ser::SerializeStruct for &'a mut Serializer<'b, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    // Standard ending without any null terminator
    fn end(self) -> Result<()> {
        Ok(())
    }
}

// Unimplemented serializer types

impl<'a, 'b, W> ser::SerializeTupleVariant for &'a mut Serializer<'b, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    // Standard ending without any null terminator
    fn end(self) -> Result<()> {
        unimplemented!()
    }
}

impl<'a, 'b, W> ser::SerializeMap for &'a mut Serializer<'b, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<()> {
        unimplemented!()
    }
}

impl<'a, 'b, W> ser::SerializeStructVariant for &'a mut Serializer<'b, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    // Standard ending without any null terminator
    fn end(self) -> Result<()> {
        unimplemented!()
    }
}
