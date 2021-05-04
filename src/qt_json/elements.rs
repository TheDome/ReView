use std::collections::HashMap;
use std::io::{Cursor, Error};


use byteorder::ReadBytesExt;


use crate::qt_json::q_json_document::Endianess;

#[derive(Debug)]
pub enum JsonValue {
    String(String),
    Number(f64),
    // Since JS uses 64Bit floats, we can use them also
    Object(Object),
    Array(Vec<JsonValue>),
    Undefined,
    Bool(bool),
    Null,
}

#[derive(Debug)]
pub struct Object {
    pub size: u32,
    pub values: HashMap<String, JsonValue>,
}

impl Object {
    pub fn from_binary(data: &Vec<u8>) -> Result<Self, Error> {
        let mut cursor = Cursor::new(&data);

        let size = cursor.read_u32::<Endianess>()?;

        assert_eq!(data.len(), (size + 4) as usize);

        Ok(Object {
            size,
            values: HashMap::new(),
        })
    }
}

#[derive(Debug)]
pub enum JsonBaseValue {
    Object(Object),
    Array(Vec<JsonValue>),
}

/**
* This is the base element of a JSOn Document.
*
* A JSON Document can have either an Array or An Object as a Base
*/
#[derive(Debug)]
pub struct JsonBase {
    /**
     * The size of the overall Object (not needed in Rust)
     */
    pub size: u32,
    /**
     * The number of Elements, this base has.
     * (Self-explainatory for Object and Array)
     */
    pub elements: u32,
    /**
     * The value of this json
     */
    pub value: JsonBaseValue,
}

#[derive(Debug)]
pub struct QJsonDocument {
    pub tag: u32,
    // qbjs
    pub version: u32,
    // 1
    pub start: JsonBase,
}
