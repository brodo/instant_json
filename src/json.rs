use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum JsonValue {
    JsonString(String),
    JsonNumber(f64),
    JsonBool(bool),
    JsonNull,
    JsonObject(HashMap<String, JsonValue>)
}
