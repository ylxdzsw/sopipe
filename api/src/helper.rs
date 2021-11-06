use std::collections::BTreeSet;

use super::{Argument, ArgumentValue};

use thiserror::Error;

use serde::Deserialize;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};

#[derive(Error, Debug)]
pub enum ArgParseError {
    #[error("the type `{0}` is not supported")]
    UnsupportedType(&'static str),
    #[error("the input number does not fit `{0}`")]
    Overflow(&'static str),
    #[error("expecting {expected}, received {supplied}")]
    TypeError {
        supplied: &'static str,
        expected: &'static str
    },
    #[error("deserialize error from serde. msg: {0}")]
    DeserializeError(String),
    #[error("expecting {expected} arguments, received {supplied}")]
    TooManyArguments {
        supplied: usize,
        expected: usize
    },
    #[error("too many positional arguments")]
    TooManyPositionalArguments,
    #[error("unknown data store error")]
    Unknown,
}

impl serde::de::Error for ArgParseError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::DeserializeError(format!("{}", msg))
    }
}

pub struct Deserializer<'de> {
    args: &'de [Argument],
    ident: bool, // should reading ident next
    pos_fields: Option<Vec<&'static str>>, // the name of positional arguments in *reverse order*, which are the fields that not appear in the arguments
    list_pos: Option<usize> // the position of the next value in a parsing list. TODO: serde has a method `into_deserializer` that maybe used here
}

impl<'de> Deserializer<'de> {
    fn from_args(args: &'de [Argument]) -> Self {
        Deserializer { args, ident: true, pos_fields: None, list_pos: None }
    }

    fn next_key(&mut self) -> &'de str {
        assert!(self.ident);
        assert!(self.list_pos.is_none());

        self.ident = false;
        &self.args[0].0
    }

    fn next_value(&mut self) -> &'de ArgumentValue {
        assert!(!self.ident);

        if let Some(list_pos) = &mut self.list_pos {
            let value = &self.args[0].1.as_vec().unwrap()[*list_pos];
            *list_pos += 1;
            return value
        }

        let value = &self.args[0].1;
        self.args = &self.args[1..];
        self.ident = true;

        value
    }

    fn peek_next_value(&self) -> &'de ArgumentValue {
        assert!(!self.ident);

        if let Some(list_pos) = &self.list_pos {
            let value = &self.args[0].1.as_vec().unwrap()[*list_pos];
            return value
        }

        &self.args[0].1
    }
}

pub fn parse_args<'a, T: Deserialize<'a>>(args: &'a [Argument]) -> Result<T, ArgParseError> {
    let mut deserializer = Deserializer::from_args(args);
    let t = T::deserialize(&mut deserializer)?;
    assert!(deserializer.args.is_empty());
    Ok(t)
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = ArgParseError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, ArgParseError> {
        match self.peek_next_value() {
            ArgumentValue::String(_) => self.deserialize_str(visitor),
            ArgumentValue::Int(_) => self.deserialize_u64(visitor),
            ArgumentValue::Vec(_) => self.deserialize_seq(visitor),
            ArgumentValue::None => self.deserialize_unit(visitor),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, ArgParseError> {
        assert!(self.next_value().is_none());
        visitor.visit_bool(true) // appearence is true
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, ArgParseError> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, ArgParseError> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, ArgParseError> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, ArgParseError> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, ArgParseError> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, ArgParseError> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, ArgParseError> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, ArgParseError> {
        let value = self.next_value();
        let value = value.as_int().ok_or_else(|| ArgParseError::TypeError { supplied: value.type_name(), expected: "int" })?;
        visitor.visit_u64(*value)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, ArgParseError> {
        unimplemented!()
    }

    fn deserialize_f64<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, ArgParseError> {
        unimplemented!()
    }

    fn deserialize_char<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, ArgParseError> {
        unimplemented!()
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, ArgParseError> {
        let value = self.next_value();
        let value = value.as_string().ok_or_else(|| ArgParseError::TypeError { supplied: value.type_name(), expected: "string" })?;
        visitor.visit_borrowed_str(value)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, ArgParseError> {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, ArgParseError> {
        unimplemented!()
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, ArgParseError> {
        unimplemented!()
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, ArgParseError> {
        if let ArgumentValue::None = self.peek_next_value() {
            return visitor.visit_none()
        }

        visitor.visit_some(self)
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, ArgParseError> {
        let value = self.next_value();
        if value.is_none() {
            visitor.visit_unit()
        } else {
            Err(ArgParseError::TypeError { expected: "none", supplied: value.type_name() })
        }
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(self, _name: &'static str, _visitor: V) -> Result<V::Value, ArgParseError> {
        unimplemented!()
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(self, _name: &'static str, visitor: V) -> Result<V::Value, ArgParseError> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, ArgParseError> {
        let value = self.peek_next_value();
        value.as_vec().ok_or_else(|| ArgParseError::TypeError { supplied: value.type_name(), expected: "vec" })?;

        self.list_pos = Some(0);
        visitor.visit_seq(SeqAccessor { de: self })
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _len: usize, _visitor: V) -> Result<V::Value, ArgParseError> {
        unimplemented!()
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(self, _name: &'static str, _len: usize, _visitor: V) -> Result<V::Value, ArgParseError> {
        unimplemented!()
    }

    fn deserialize_map<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, ArgParseError> {
        unimplemented!()
    }

    fn deserialize_struct<V: Visitor<'de>>(self, _name: &'static str, fields: &'static [&'static str], visitor: V) -> Result<V::Value, ArgParseError> {
        assert!(self.ident);
        assert!(self.pos_fields.is_none());

        if self.args.len() > fields.len() {
            return Err(ArgParseError::TooManyArguments { expected: fields.len(), supplied: self.args.len() })
        }

        let appeared: BTreeSet<_> = self.args.iter().map(|x| &x.0[..]).collect();

        self.pos_fields = Some(fields.iter().copied().filter(|x| !appeared.contains(x)).rev().collect());

        visitor.visit_map(MapAccessor { de: self })
    }

    fn deserialize_enum<V: Visitor<'de>>(self, _name: &'static str, _variants: &'static [&'static str], _visitor: V) -> Result<V::Value, ArgParseError> {
        todo!()
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, ArgParseError> {
        let mut value = self.next_key();
        if value.is_empty() {
            if let Some(name) = self.pos_fields.as_mut().unwrap().pop() {
                value = name;
            } else {
                return Err(ArgParseError::TooManyPositionalArguments)
            }
        }
        visitor.visit_borrowed_str(value)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, ArgParseError> {
        self.next_value();
        visitor.visit_none()
    }
}

struct SeqAccessor<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> SeqAccess<'de> for SeqAccessor<'a, 'de> {
    type Error = ArgParseError;

    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>, ArgParseError> {
        if self.de.list_pos.unwrap() >= self.de.args[0].1.as_vec().unwrap().len() {
            self.de.list_pos = None;
            self.de.next_value();
            return Ok(None)
        }

        seed.deserialize(&mut *self.de).map(Some)
    }
}

struct MapAccessor<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> MapAccess<'de> for MapAccessor<'a, 'de> {
    type Error = ArgParseError;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>, ArgParseError> {
        if self.de.args.is_empty() {
            return Ok(None)
        }

        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, ArgParseError> {
        seed.deserialize(&mut *self.de)
    }
}
