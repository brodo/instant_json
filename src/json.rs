use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::rc::Rc;
use crate::JsonValue::JsonObject;

#[derive(Serialize, Deserialize, Debug)]
pub enum JsonValue {
    JsonString(String),
    JsonNumber(f64),
    JsonBool(bool),
    JsonNull,
    JsonObject(HashMap<String, JsonValue>)
}
