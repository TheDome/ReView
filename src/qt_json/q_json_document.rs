use std::borrow::Borrow;
use std::collections::HashMap;
use std::io::{Cursor, Error, ErrorKind, Read, Seek, Write};
use std::path::Component::CurDir;

use byteorder::{LittleEndian, ReadBytesExt};
use gtk::License;
use log::{debug, trace, warn};
use num_traits::FromPrimitive;

use crate::qt_json::elements::{JSONBaseValue, JsonValue, Object};
use crate::qt_json::q_json_document::QTValueType::Array;

pub struct QJSONDocument {
    pub tag: u32,
    pub version: u32,
    // 1
    pub base: JSONBaseValue,
}

#[derive(Debug, Eq, PartialEq, FromPrimitive)]
#[repr(u32)]
enum QTValueType {
    Null = 0x0,
    Bool = 0x1,
    Double = 0x2,
    String = 0x3,
    Array = 0x4,
    Object = 0x5,
    Undefined = 0x80,
}

#[cfg(not(feature = "QJsonBigEndian"))]
const QT_JSON_TAG: u32 =
    ((('s' as u32) << 24) | (('j' as u32) << 16) | (('b' as u32) << 8) | ('q' as u32));
#[cfg(feature = "QJsonBigEndian")]
const QT_JSON_TAG: u32 =
    ((('q' as u32) << 24) | (('b' as u32) << 16) | (('j' as u32) << 8) | ('s' as u32));

#[cfg(not(feature = "QJsonBigEndian"))]
pub type Endianess = LittleEndian;
#[cfg(feature = "QJsonBigEndian")]
pub type Endianess = BigEndian;

impl QJSONDocument {
    pub fn from_binary(data: Vec<u8>) -> Result<Self, Error> {
        debug!("[QBJS] Loading data");

        let mut reader = Cursor::new(&data);

        let tag = reader.read_u32::<Endianess>()?;
        let version = reader.read_u32::<Endianess>()?;

        assert_eq!(tag, QT_JSON_TAG);

        assert_eq!(version, 1);

        debug!("QBJS Version: {}", version);

        let elem = Self::load_element(data[8..].to_vec())?;

        let base = match elem {
            JsonValue::Object(o) => JSONBaseValue::Object(o),
            JsonValue::Array(a) => JSONBaseValue::Array(a),
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "The Base must be either an Array or object",
                ));
            }
        };

        let doc = QJSONDocument { tag, version, base };

        debug!("[QBJS] Parsing finished!");

        Ok(doc)
    }

    fn load_element(data: Vec<u8>) -> Result<JsonValue, Error> {
        let mut reader = Cursor::new(&data);

        let size = reader.read_u32::<Endianess>()?;
        let header = reader.read_u32::<Endianess>()?;
        let offset = reader.read_u32::<Endianess>()?;

        let is_object = (header & 0x1) == 1;
        let len = header >> 1;

        trace!("Element Size is: {:#0X}", size);
        trace!("Element Offset is: {:#0X}", offset);
        trace!("Element is an object: {}", is_object);
        trace!("Element elements: {}", len);

        let table = data.split_at(offset as usize).1;

        // u32 is 4 bytes
        trace!("Table len is {}", table.len() / 4);

        let base = match is_object {
            true => Self::load_object(&data, table, len, size),
            false => Self::load_array(&data, table, len, size),
        };

        trace!("{:?}", base);

        Ok(base?)
    }

    /**
     * loads an object from the stream
     */
    fn load_object(
        data: &Vec<u8>,
        offsets: &[u8],
        len: u32,
        size: u32,
    ) -> Result<JsonValue, Error> {
        debug!("Loading object ..");
        trace!("Expected len: {}", len);
        trace!("Actual len: {}", offsets.len() / 4);

        assert!(offsets.len() / 4 >= len as usize);

        let mut offsets = Cursor::new(offsets);
        let mut values = HashMap::new();

        for i in 0..len {
            trace!("Iterating over entry {}", i);

            let offset = offsets.read_u32::<Endianess>()?;
            trace!("Entry at offset: {:0X?}", offset);

            let element = data.split_at(offset as usize).1;
            let mut reader = Cursor::new(element);

            let value_header = reader.read_u32::<Endianess>()?;
            trace!(" > Value header {:032b}", value_header);

            let value_type_number: u32 = value_header & 0b111;
            let latin_or_int = ((value_header & 0b1000) >> 3) == 1;
            let latin_key = ((value_header & 0b10000) >> 4) == 1;
            let orig_value: u32 = (value_header & 0xFFFFFFE0) >> 5;

            let value_type: Option<QTValueType> = FromPrimitive::from_u32(value_type_number);

            if value_type.is_none() {
                warn!("Could not parse value at json entry {}\nContinuing. But this might have unacceptable impact", i);
                debug!("Value type: {:#0X}", value_type_number);
                debug!("Value value: {:#04X}", orig_value);
            }

            trace!(" > Value of type: {:?}", value_type);
            trace!(" > Key is latin: {}", latin_key);
            let key = Self::read_string(&mut reader, latin_key)?;

            trace!(" > Key is: '{}'", key);
            trace!(" > Reading value of type: {:?}", value_type);

            let value = Self::decode_value(
                value_type,
                orig_value,
                latin_or_int,
                latin_key,
                size as usize,
                data,
            )?;

            trace!(" > Value is: {:?}", value);

            values.insert(key, value);
        }

        let object = Object { size: len, values };

        trace!("Using object {:?}", object);

        Ok(JsonValue::Object(object))
    }

    fn load_array(data: &Vec<u8>, offsets: &[u8], len: u32, size: u32) -> Result<JsonValue, Error> {
        debug!("Loading array ..");
        trace!("Expected len: {}", len);
        trace!("Actual len: {}", offsets.len() / 4);

        assert!(offsets.len() / 4 >= len as usize);

        let mut offsets = Cursor::new(offsets);
        let mut values = Vec::new();

        for i in 0..len {
            trace!("Iterating over entry {}", i);

            let offset = offsets.read_u32::<Endianess>()?;
            trace!("Entry at offset: 0x{:0X}", offset);

            let value_header = offset;
            trace!(" > Value header {:032b}b", value_header);

            let value_type_number: u16 = (value_header & 0b111) as u16;
            let latin_or_int = ((value_header & 0b1000) >> 3) == 1;
            let latin_key = ((value_header & 0b10000) >> 4) == 1;
            let orig_value: u32 = (value_header & 0xFFFFFFE0) >> 5;

            let value_type: Option<QTValueType> = FromPrimitive::from_u16(value_type_number);

            if value_type.is_none() {
                warn!("Could not parse value at json entry {}\nContinuing. But this might have unacceptable impact", i);
                debug!("Value type: {:#0X}", value_type_number);
                debug!("Value value: {:#04X}", orig_value);
            }

            trace!(" > Reading value of type: {:?}", value_type);

            let value = Self::decode_value(
                value_type,
                orig_value,
                latin_or_int,
                latin_key,
                size as usize,
                data,
            )?;

            trace!(" > Value is: {:?}", value);

            values.push(value);
        }

        Ok(JsonValue::Array(values))
    }

    fn decode_value(
        value_type: Option<QTValueType>,
        orig_value: u32,
        latin_or_int: bool,
        latin_key: bool,
        size: usize,
        data: &[u8],
    ) -> Result<JsonValue, std::io::Error> {
        let value = match value_type {
            Some(QTValueType::Double) => {
                if latin_or_int {
                    JsonValue::Number(orig_value.into())
                } else {
                    trace!(" > > Value is of type f64");
                    trace!(" > > Value located at offset: {:0X?}", orig_value);

                    let value_data = data.split_at(orig_value as usize).1;
                    let mut reader = Cursor::new(value_data);
                    JsonValue::Number(reader.read_f64::<Endianess>()?)
                }
            }
            Some(QTValueType::String) => {
                trace!(" > > Value located at offset: {:0X?}", orig_value);

                let value_data = data.split_at(orig_value as usize).1;
                let mut reader = Cursor::new(value_data);
                JsonValue::String(Self::read_string(&mut reader, latin_key)?)
            }
            Some(QTValueType::Object) | Some(QTValueType::Array) => {
                trace!(" > > Value located at offset: {:0X?}", orig_value);

                trace!(
                    " > > Trimming {} bytes from object",
                    data.len() - size as usize
                );
                let value_data = data.split_at(size as usize).0;

                trace!(" > > Trimming {} bytes from object top", orig_value);
                let encapsulated = value_data.split_at(orig_value as usize).1;
                Self::load_element(Vec::from(encapsulated))?
            }
            Some(QTValueType::Null) => JsonValue::Null,
            _ => JsonValue::Undefined,
        };

        Ok(value)
    }

    /**
     * reads a string.
     * This class is capable of reading a string in UTF16 and UTF8
     */
    fn read_string(reader: &mut dyn Read, latin: bool) -> Result<String, Error> {
        let key_len = match latin {
            true => reader.read_u16::<Endianess>()? as u32,
            false => reader.read_u32::<Endianess>()?,
        };

        trace!(" --> Reading string, latin:{}, len:{}", latin, key_len);
        if latin {
            let mut buffer = Vec::new();
            for _ in 0..key_len {
                buffer.push(reader.read_u8()?);
            }

            Ok(String::from_utf8_lossy(buffer.as_slice()).parse().unwrap())
        } else {
            let mut buffer = Vec::new();
            for _ in 0..key_len {
                buffer.push((reader.read_u8()? as u16) << 8 | reader.read_u8()? as u16);
            }
            Ok(String::from_utf16_lossy(buffer.as_slice()))
        }
    }
}
