use serde::de::{
    self, Deserialize, DeserializeSeed, Deserializer, SeqAccess, VariantAccess, Visitor,
};
use serde::ser::{self, Serialize, SerializeSeq, SerializeStruct, Serializer};
use std::{
    convert::{TryFrom, TryInto},
    fmt,
    marker::PhantomData,
};

pub trait ListLen: TryFrom<u64> + Into<u64> {
    const BITS: u32;
}
impl ListLen for u8 {
    const BITS: u32 = u8::BITS;
}
impl ListLen for u16 {
    const BITS: u32 = u16::BITS;
}
impl ListLen for u32 {
    const BITS: u32 = u32::BITS;
}
impl ListLen for u64 {
    const BITS: u32 = u64::BITS;
}

#[derive(Clone, Debug, PartialEq)]
pub struct List<T, L: ListLen>(Vec<T>, PhantomData<L>);

impl<T: Serialize, L: ListLen + Serialize> Serialize for List<T, L> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut struct_ser = serializer.serialize_struct("List", 2)?;

        struct_ser.serialize_field::<L>(
            "length",
            &(self.0.len() as u64)
                .try_into()
                .or(Err(ser::Error::custom(format!(
                    "List length cannot fit into {} bits",
                    L::BITS
                ))))?,
        )?;
        struct_ser.serialize_field("elements", &self.0)?;

        struct_ser.end()
    }
}

impl<T, L: ListLen> Default for List<T, L> {
    fn default() -> Self {
        Self(Default::default(), PhantomData)
    }
}

impl<T, L: ListLen> From<Vec<T>> for List<T, L> {
    fn from(val: Vec<T>) -> Self {
        Self(val, PhantomData)
    }
}

impl<T, L: ListLen> From<List<T, L>> for Vec<T> {
    fn from(val: List<T, L>) -> Self {
        val.0
    }
}

struct ListVisitor<T, L> {
    element_type: PhantomData<T>,
    length_type: PhantomData<L>,
}

impl<'de, T, L> Visitor<'de> for ListVisitor<T, L>
where
    T: Deserialize<'de>,
    L: ListLen + Deserialize<'de>,
{
    type Value = List<T, L>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_fmt(format_args!(
            "a list prefixed with a {}-bit length value",
            L::BITS
        ))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let len = seq
            .next_element::<L>()?
            .map(|len| len.into())
            .ok_or(de::Error::invalid_length(0, &self))?;

        Ok(seq
            .next_element_seed(ListElements {
                len: len as usize,
                inner: Default::default(),
            })?
            .ok_or(de::Error::invalid_length(1, &self))?
            .into())
    }
}

impl<'de, T, L> Deserialize<'de> for List<T, L>
where
    T: 'de + Deserialize<'de>,
    L: 'de + ListLen + Deserialize<'de>,
{
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
                length_type: PhantomData,
            },
        )
    }
}

struct ListElements<T> {
    len: usize,
    inner: Vec<T>,
}

impl<'de, T> DeserializeSeed<'de> for ListElements<T>
where
    T: Deserialize<'de>,
{
    type Value = Vec<T>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ListElementsVisitor<T>(ListElements<T>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for ListElementsVisitor<T> {
            type Value = Vec<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_fmt(format_args!("a list of length {}", self.0.len))
            }

            fn visit_seq<A: SeqAccess<'de>>(mut self, mut seq: A) -> Result<Self::Value, A::Error> {
                let size = seq.size_hint().unwrap_or(self.0.len);

                self.0.inner.reserve(size);
                while let Some(element) = seq.next_element::<T>()? {
                    self.0.inner.push(element);
                }

                if self.0.inner.len() != self.0.len {
                    Err(de::Error::invalid_length(self.0.inner.len(), &self))
                } else {
                    Ok(self.0.inner)
                }
            }
        }

        deserializer.deserialize_tuple(self.len, ListElementsVisitor(self))
    }
}

pub trait ListTerminator: TryFrom<u64> + Into<u64> {
    const BITS: u32;
}
impl ListTerminator for u8 {
    const BITS: u32 = u8::BITS;
}
impl ListTerminator for u16 {
    const BITS: u32 = u16::BITS;
}
impl ListTerminator for u32 {
    const BITS: u32 = u32::BITS;
}
impl ListTerminator for u64 {
    const BITS: u32 = u64::BITS;
}

#[derive(Clone, Debug, PartialEq)]
pub struct ListTerm<T, Term: ListTerminator>(Vec<T>, PhantomData<Term>);

impl<T: Serialize, Term: ListTerminator + Serialize> Serialize for ListTerm<T, Term> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Clone, Debug, PartialEq, serde::Serialize)]
        enum Element<'a, T, Term: ListTerminator> {
            Terminator(Term),
            Value(&'a T),
        }

        let mut seq_ser = serializer.serialize_seq(Some(self.0.len()))?;
        for element in &self.0 {
            seq_ser.serialize_element(&Element::<T, Term>::Value(element))?;
        }
        seq_ser.serialize_element(&Element::<T, Term>::Terminator(unsafe {
            0u64.try_into().unwrap_unchecked()
        }))?;

        seq_ser.end()
    }
}

impl<T, Term: ListTerminator> Default for ListTerm<T, Term> {
    fn default() -> Self {
        Self(Default::default(), PhantomData)
    }
}

impl<T, Term: ListTerminator> From<Vec<T>> for ListTerm<T, Term> {
    fn from(val: Vec<T>) -> Self {
        Self(val, PhantomData)
    }
}

impl<T, Term: ListTerminator> From<ListTerm<T, Term>> for Vec<T> {
    fn from(val: ListTerm<T, Term>) -> Self {
        val.0
    }
}

struct ListTermVisitor<T, Term> {
    element_type: PhantomData<T>,
    terminator_type: PhantomData<Term>,
}

impl<'de, T, Term> Visitor<'de> for ListTermVisitor<T, Term>
where
    T: Deserialize<'de>,
    Term: ListTerminator + Deserialize<'de>,
{
    type Value = ListTerm<T, Term>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_fmt(format_args!(
            "a list terminated by a {}-bit null value",
            Term::BITS
        ))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut elements = Vec::<T>::new();
        while let Some(element) = seq.next_element::<ListTermElement<T, Term>>()? {
            match element {
                ListTermElement::Value(val) => elements.push(val),
                ListTermElement::Terminator(_) => break,
            }
        }

        Ok(elements.into())
    }
}

impl<'de, T, Term> Deserialize<'de> for ListTerm<T, Term>
where
    T: 'de + Deserialize<'de>,
    Term: 'de + ListTerminator + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ListTermVisitor {
            element_type: PhantomData,
            terminator_type: PhantomData,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
enum ListTermElement<T, Term: ListTerminator> {
    Terminator(PhantomData<Term>),
    Value(T),
}

impl<'de, T, Term> Deserialize<'de> for ListTermElement<T, Term>
where
    T: Deserialize<'de>,
    Term: ListTerminator + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const VARIANTS: &'static [&'static str] = &["Terminator", "Value"];
        deserializer.deserialize_enum("ListTermElement", VARIANTS, ListTermElementVisitor::new())
    }
}

struct ListTermElementVisitor<T, Term: ListTerminator> {
    element_type: PhantomData<T>,
    terminator_type: PhantomData<Term>,
}

impl<T, Term: ListTerminator> ListTermElementVisitor<T, Term> {
    fn new() -> Self {
        Self {
            element_type: PhantomData,
            terminator_type: PhantomData,
        }
    }
}

impl<'de, T, Term> Visitor<'de> for ListTermElementVisitor<T, Term>
where
    T: Deserialize<'de>,
    Term: ListTerminator + Deserialize<'de>,
{
    type Value = ListTermElement<T, Term>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_fmt(format_args!(
            "an untagged enum variant, either Value({}) or Terminator(0{})",
            std::any::type_name::<T>(),
            std::any::type_name::<Term>()
        ))
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        // Figure out if this is a terminator
        let (val, variant) = data.variant::<Term>()?;

        match val.into() {
            0u64 => {
                <A::Variant as VariantAccess>::unit_variant(variant)?;
                Ok(ListTermElement::Terminator(PhantomData))
            }
            _ => Ok(ListTermElement::Value(
                <A::Variant as VariantAccess>::newtype_variant(variant)?,
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ListNonTerm<T: Serialize>(Vec<T>);

impl<T: Serialize> Default for ListNonTerm<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Serialize> From<Vec<T>> for ListNonTerm<T> {
    fn from(val: Vec<T>) -> Self {
        Self(val)
    }
}

impl<T: Serialize> From<ListNonTerm<T>> for Vec<T> {
    fn from(val: ListNonTerm<T>) -> Self {
        val.0
    }
}
