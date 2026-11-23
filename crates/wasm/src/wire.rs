//! NTE wire v1: a self-describing, interned binary encoding of Serde's JSON model.
//! Containers end with END; strings use a per-message dictionary. Safe integers
//! use f64 (exact through 2^53-1); larger integers retain their decimal spelling.
use serde::ser::{self, Serialize, Serializer};
use std::collections::HashMap;

const NULL: u8 = 0;
const FALSE: u8 = 1;
const TRUE: u8 = 2;
const NUMBER: u8 = 3;
const STRING: u8 = 4;
const REFERENCE: u8 = 5;
const ARRAY: u8 = 6;
const OBJECT: u8 = 7;
const END: u8 = 8;
const EXACT: u8 = 9;
const RAW_JSON: u8 = 10;

pub fn encode(value: &(impl Serialize + ?Sized)) -> Result<Vec<u8>, serde_json::Error> {
    let mut wire = Wire {
        bytes: b"NTE\x01".to_vec(),
        strings: HashMap::new(),
    };
    value.serialize(&mut wire)?;
    Ok(wire.bytes)
}

pub struct Wire {
    bytes: Vec<u8>,
    strings: HashMap<String, u32>,
}

impl Wire {
    fn text(&mut self, text: &str) -> Result<(), serde_json::Error> {
        if let Some(index) = self.strings.get(text) {
            self.bytes.push(REFERENCE);
            self.bytes.extend_from_slice(&index.to_le_bytes());
        } else {
            let index = u32::try_from(self.strings.len()).map_err(ser::Error::custom)?;
            let len = u32::try_from(text.len()).map_err(ser::Error::custom)?;
            self.bytes.push(STRING);
            self.bytes.extend_from_slice(&len.to_le_bytes());
            self.bytes.extend_from_slice(text.as_bytes());
            self.strings.insert(text.to_owned(), index);
        }
        Ok(())
    }

    fn number(&mut self, value: f64) {
        if value.is_finite() {
            self.bytes.push(NUMBER);
            self.bytes.extend_from_slice(&value.to_le_bytes());
        } else {
            self.bytes.push(NULL);
        }
    }

    fn integer(&mut self, value: i128) -> Result<(), serde_json::Error> {
        if (-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&value) {
            self.number(value as f64);
            Ok(())
        } else {
            self.exact(&value.to_string())
        }
    }

    fn exact(&mut self, text: &str) -> Result<(), serde_json::Error> {
        self.bytes.push(EXACT);
        self.text(text)
    }

    fn container(&mut self, tag: u8, ends: u8) -> Compound<'_> {
        self.bytes.push(tag);
        Compound {
            wire: self,
            ends,
            raw_number: false,
        }
    }
}

pub struct Compound<'a> {
    wire: &'a mut Wire,
    ends: u8,
    raw_number: bool,
}

impl Compound<'_> {
    fn finish(self) -> Result<(), serde_json::Error> {
        for _ in 0..self.ends {
            self.wire.bytes.push(END);
        }
        Ok(())
    }
}

macro_rules! signed {
    ($($method:ident($ty:ty)),* $(,)?) => {$(
        fn $method(self, value: $ty) -> Result<(), Self::Error> { self.integer(i128::from(value)) }
    )*};
}

impl<'a> Serializer for &'a mut Wire {
    type Ok = ();
    type Error = serde_json::Error;
    type SerializeSeq = Compound<'a>;
    type SerializeTuple = Compound<'a>;
    type SerializeTupleStruct = Compound<'a>;
    type SerializeTupleVariant = Compound<'a>;
    type SerializeMap = Compound<'a>;
    type SerializeStruct = Compound<'a>;
    type SerializeStructVariant = Compound<'a>;

    // Follow the same representation choices as JSON-facing domain serializers.
    fn is_human_readable(&self) -> bool {
        true
    }
    fn serialize_bool(self, value: bool) -> Result<(), Self::Error> {
        self.bytes.push(if value { TRUE } else { FALSE });
        Ok(())
    }
    signed!(
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_i128(i128),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64)
    );
    fn serialize_u128(self, value: u128) -> Result<(), Self::Error> {
        match i128::try_from(value) {
            Ok(value) => self.integer(value),
            Err(_) => self.exact(&value.to_string()),
        }
    }
    fn serialize_f32(self, value: f32) -> Result<(), Self::Error> {
        if value.is_finite() {
            self.exact(&serde_json::to_string(&value)?)
        } else {
            self.serialize_unit()
        }
    }
    fn serialize_f64(self, value: f64) -> Result<(), Self::Error> {
        self.number(value);
        Ok(())
    }
    fn serialize_char(self, value: char) -> Result<(), Self::Error> {
        self.text(value.encode_utf8(&mut [0; 4]))
    }
    fn serialize_str(self, value: &str) -> Result<(), Self::Error> {
        self.text(value)
    }
    fn serialize_bytes(self, value: &[u8]) -> Result<(), Self::Error> {
        self.bytes.push(ARRAY);
        for byte in value {
            self.integer(i128::from(*byte))?;
        }
        self.bytes.push(END);
        Ok(())
    }
    fn serialize_none(self) -> Result<(), Self::Error> {
        self.serialize_unit()
    }
    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<(), Self::Error> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), Self::Error> {
        self.bytes.push(NULL);
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), Self::Error> {
        self.serialize_unit()
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<(), Self::Error> {
        self.text(variant)
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.bytes.push(OBJECT);
        self.text(variant)?;
        value.serialize(&mut *self)?;
        self.bytes.push(END);
        Ok(())
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Compound<'a>, Self::Error> {
        Ok(self.container(ARRAY, 1))
    }
    fn serialize_tuple(self, _: usize) -> Result<Compound<'a>, Self::Error> {
        self.serialize_seq(None)
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Compound<'a>, Self::Error> {
        self.serialize_seq(None)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Compound<'a>, Self::Error> {
        self.bytes.push(OBJECT);
        self.text(variant)?;
        Ok(self.container(ARRAY, 2))
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Compound<'a>, Self::Error> {
        Ok(self.container(OBJECT, 1))
    }
    fn serialize_struct(self, name: &'static str, _: usize) -> Result<Compound<'a>, Self::Error> {
        // serde_json arbitrary_precision numbers are serialized through this
        // private struct token. Preserve the original number, not its wrapper.
        if name == "$serde_json::private::Number" {
            self.bytes.push(EXACT);
            Ok(Compound {
                wire: self,
                ends: 0,
                raw_number: true,
            })
        } else if name == "$serde_json::private::RawValue" {
            // Borrowed static provenance is already stored as JSON by content.
            // Intern it once per batch instead of reparsing/cloning it in Rust.
            self.bytes.push(RAW_JSON);
            Ok(Compound {
                wire: self,
                ends: 0,
                raw_number: true,
            })
        } else {
            self.serialize_map(None)
        }
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Compound<'a>, Self::Error> {
        self.bytes.push(OBJECT);
        self.text(variant)?;
        Ok(self.container(OBJECT, 2))
    }
}

macro_rules! sequence {
    ($trait:ident, $method:ident) => {
        impl ser::$trait for Compound<'_> {
            type Ok = ();
            type Error = serde_json::Error;
            fn $method<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
                value.serialize(&mut *self.wire)
            }
            fn end(self) -> Result<(), Self::Error> {
                self.finish()
            }
        }
    };
}
sequence!(SerializeSeq, serialize_element);
sequence!(SerializeTuple, serialize_element);
sequence!(SerializeTupleStruct, serialize_field);
sequence!(SerializeTupleVariant, serialize_field);

impl ser::SerializeMap for Compound<'_> {
    type Ok = ();
    type Error = serde_json::Error;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Self::Error> {
        // JSON map keys can be strings, integer IDs or booleans. Coerce these
        // scalar forms without accepting composite keys that JSON rejects.
        let key = serde_json::to_value(key)?;
        match key {
            serde_json::Value::String(text) => self.wire.text(&text),
            serde_json::Value::Number(number) => self.wire.text(&number.to_string()),
            serde_json::Value::Bool(value) => self.wire.text(if value { "true" } else { "false" }),
            _ => Err(ser::Error::custom(
                "wire object key must be a string or scalar ID",
            )),
        }
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut *self.wire)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}

macro_rules! structure {
    ($trait:ident) => {
        impl ser::$trait for Compound<'_> {
            type Ok = ();
            type Error = serde_json::Error;
            fn serialize_field<T: Serialize + ?Sized>(
                &mut self,
                key: &'static str,
                value: &T,
            ) -> Result<(), Self::Error> {
                if !self.raw_number {
                    self.wire.text(key)?;
                }
                value.serialize(&mut *self.wire)
            }
            fn end(self) -> Result<(), Self::Error> {
                self.finish()
            }
        }
    };
}
structure!(SerializeStruct);
structure!(SerializeStructVariant);
