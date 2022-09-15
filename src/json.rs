use serde::{Serialize, Deserialize, Serializer};
use std::collections::HashMap;
use serde::ser::{SerializeMap, SerializeSeq};
use crate::{JsonArray, JsonNumber, JsonObject, JsonString};
use crate::JsonValue::JsonBool;


#[derive(Deserialize,Debug)]
pub enum JsonValue {
    JsonString(String),
    JsonNumber(f64),
    JsonBool(bool),
    JsonNull,
    JsonArray(Vec<JsonValue>),
    JsonObject(HashMap<String, JsonValue>),
}

impl Serialize for JsonValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        match self {
            JsonString(str) => serializer.serialize_str(str),
            JsonNumber(num) => serializer.serialize_f64(num.clone()),
            JsonBool(b) => serializer.serialize_bool(b.clone()),
            JsonValue::JsonNull => serializer.serialize_none(),
            JsonObject(m) => {
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            JsonArray(arr) => {
                let mut seq = serializer.serialize_seq(Some(arr.len()))?;
                for e in arr {
                    seq.serialize_element(e)?;
                }
                seq.end()
            }
        }
    }
}
