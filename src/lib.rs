use std::collections::HashMap;

use pest_meta::optimizer::{optimize, OptimizedRule};
use pest_meta::parser::{self, Rule};
use pest_meta::validator::validate_pairs;
use pest_vm::Vm;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::console_log;
use crate::error::InstantJsonError;
use crate::InstantJsonError::{FloatParse, JsonParse};
use crate::json::JsonValue;
use crate::JsonValue::{JsonArray, JsonNull, JsonNumber, JsonObject, JsonString};
use serde::Serialize;
use serde_wasm_bindgen::Serializer;

mod error;
mod json;

#[wasm_bindgen]
pub struct InstantJson {
    vms: HashMap<String, Vm>,
}

#[wasm_bindgen]
impl InstantJson {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        InstantJson {
            vms: HashMap::new()
        }
    }
    #[wasm_bindgen]
    pub fn compile(&mut self, schema_name: &str, schema: &str) -> Result<(), JsError> {
        let rules = parse_pest(schema)?;
        self.vms.insert(schema_name.to_string(), Vm::new(rules));
        Ok(())
    }

    #[wasm_bindgen]
    pub fn parse(&self, schema_name: &str, json_str: &str) -> Result<JsValue, JsError> {
        let vm = self.vms.get(schema_name).ok_or(InstantJsonError::NotFound)?;
        let mut pairs = vm.parse("root", json_str)?;
        let root_pair = pairs.next().ok_or(JsonParse { message: "invalid root".to_owned() })?;
        let root_obj_pair = root_pair.into_inner().next().ok_or(JsonParse { message: "invalid root object".to_owned() })?;
        let mut root = JsonObject(HashMap::new());
        if root_obj_pair.as_rule() == "object" {
            let current_obj = &mut &mut root;
            let mut stack = vec![];
            stack.push(root_obj_pair.into_inner());
            let mut is_key = false;
            let mut current_key = "";
            while let Some(current) = stack.pop() {
                for child in current {
                    match child.as_rule() {
                        "object" => {
                            let new_obj = JsonObject(HashMap::new());
                            match current_obj {
                                JsonObject(hm) => {
                                    hm.insert(current_key.to_owned(), new_obj);
                                    *current_obj = hm.get_mut(current_key).unwrap();
                                    stack.push(child.into_inner());
                                }
                                JsonArray(arr) => {
                                    arr.push(new_obj);
                                    *current_obj = arr.last_mut().unwrap();
                                    stack.push(child.into_inner());
                                }
                                _ => {}
                            }
                        }
                        "pair" => {
                            stack.push(child.clone().into_inner());
                            is_key = true;
                        }
                        "string" => {
                            let child_str = child.as_str();
                            let child_str_unquoted = &child_str[1..child_str.len() - 1];
                            if is_key {
                                current_key = child_str_unquoted;
                                is_key = false;
                            } else {
                                match current_obj {
                                    JsonObject(hm) => {
                                        hm.insert(current_key.to_owned(), JsonString(child_str_unquoted.to_string()));
                                    }
                                    JsonArray(arr) => {
                                        arr.push(JsonString(child_str_unquoted.to_string()))
                                    }
                                    _ => {}
                                }
                            }
                        }
                        "number" => {
                            let child_str = child.as_str();
                            match child_str.parse::<f64>() {
                                Ok(child_num) => {
                                    match current_obj {
                                        JsonObject(hm) => {
                                            hm.insert(current_key.to_owned(), JsonNumber(child_num));
                                        }
                                        JsonArray(arr) => {
                                            arr.push(JsonNumber(child_num))
                                        }
                                        _ => {}
                                    }
                                }
                                Err(e) =>{
                                    return Err(FloatParse(e).into());
                                }
                            }


                        }
                        "null" => {
                            match current_obj {
                                JsonObject(hm) => {
                                    hm.insert(current_key.to_owned(), JsonNull);
                                }
                                JsonArray(arr) => {
                                    arr.push(JsonNull)
                                }
                                _ => {}
                            }
                        }
                        "array" => {
                            let new_arr = JsonArray(vec![]);
                            match current_obj {
                                JsonObject(hm) => {
                                    hm.insert(current_key.to_owned(), new_arr);
                                    *current_obj = hm.get_mut(current_key).unwrap();
                                    stack.push(child.into_inner());
                                }
                                JsonArray(arr) => {
                                    arr.push(new_arr);
                                    *current_obj = arr.last_mut().unwrap();
                                    stack.push(child.into_inner());
                                }
                                _ => {}
                            }
                        }
                        "EOI" => {}
                        _ => {
                            console_log!("parsed unknown thing: {}", child.as_rule());
                        }
                    }
                }
            }
        } else {
            return Err(JsonParse { message: "Root needs to be object!".to_string() }.into());
        }
        let serializer = Serializer::json_compatible();
        root.serialize(&serializer).map_err(|_| { JsonParse { message: "invalid root".to_owned() }.into() })
    }
}


fn parse_pest(input: &str) -> Result<Vec<OptimizedRule>, InstantJsonError> {
    let pairs = parser::parse(Rule::grammar_rules, input)?;
    validate_pairs(pairs.clone())?;
    let ast = parser::consume_rules(pairs)?;
    Ok(optimize(ast))
}


#[cfg(test)]
pub mod tests {
    use crate::{InstantJson};
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::console;
    use js_sys::{JSON};
    use wasm_bindgen::{JsError, JsValue};

    static JSON_GRAMMAR: &str = include_str!("../examples/json.pest");

    #[wasm_bindgen_test]
    fn test_parse_pest() {
        let mut ij = InstantJson::new();
        let res = ij.compile("simple_schema", &JSON_GRAMMAR);
        if let Err(e) = res {
            console::log_1(&e.into());
            assert!(false, "Error while compiling rules");
        }
        assert_eq!(ij.vms.len(), 1);
    }


    fn simple_test_init() -> InstantJson {
        let mut ij = InstantJson::new();
        let com_res = ij.compile("simple_schema", JSON_GRAMMAR);
        if let Err(e) = com_res {
            console::log_1(&e.into());
            assert!(false, "Error while compiling rules");
        }
        ij
    }

    fn is_invertible(res: Result<JsValue, JsError>, input: &str) {
        encodes_to(res, input, input);
    }
    fn encodes_to(res: Result<JsValue, JsError>, input: &str, expected: &str) {
        match res {
            Err(e) => {
                console::log_1(&e.into());
                assert!(false, "got error");
            }
            Ok(obj) => {
                assert!(obj.is_object());
                let res_json = JSON::stringify(&obj).unwrap();
                assert_eq!(res_json, expected)
            }
        }
    }
    

    #[wasm_bindgen_test]
    fn test_parse_number() {
        let ij = simple_test_init();
        let input = r#"{"hello":1}"#;
        let p_res = ij.parse("simple_schema", input);
        is_invertible(p_res, input);
        assert_eq!(ij.vms.len(), 1);
    }

    #[wasm_bindgen_test]
    fn test_parse_string() {
        let ij = simple_test_init();
        let input = r#"{"hello":"world"}"#;
        let p_res = ij.parse("simple_schema", input);
        is_invertible(p_res, input);
        assert_eq!(ij.vms.len(), 1);
    }


    #[wasm_bindgen_test]
    fn test_parse_nested_once() {
        let ij = simple_test_init();
        let input = r#"{"hello":{"world":1}}"#;
        let p_res = ij.parse("simple_schema", input);
        is_invertible(p_res, input);
        assert_eq!(ij.vms.len(), 1);
    }

    #[wasm_bindgen_test]
    fn test_parse_nested_twice() {
        let ij = simple_test_init();
        let input = r#"{"hello":{"world":{"test":1}}}"#;
        let p_res = ij.parse("simple_schema", input);
        is_invertible(p_res, input);
        assert_eq!(ij.vms.len(), 1);
    }

    #[wasm_bindgen_test]
    fn test_parse_null() {
        let ij = simple_test_init();
        let input = r#"{"hello":null}"#;
        let p_res = ij.parse("simple_schema", input);
        is_invertible(p_res, input);
        assert_eq!(ij.vms.len(), 1);
    }

    #[wasm_bindgen_test]
    fn test_parse_array() {
        let ij = simple_test_init();
        let input = r#"{"hello":[1,2,3]}"#;
        let p_res = ij.parse("simple_schema", input);
        is_invertible(p_res, input);
        assert_eq!(ij.vms.len(), 1);
    }

    #[wasm_bindgen_test]
    fn test_parse_array_with_obj() {
        let ij = simple_test_init();
        let input = r#"{"hello":[1,2,{"foo":"bar"}]}"#;
        let p_res = ij.parse("simple_schema", input);
        is_invertible(p_res, input);
        assert_eq!(ij.vms.len(), 1);
    }

    #[wasm_bindgen_test]
    fn test_parse_array_empty() {
        let ij = simple_test_init();
        let input = r#"{"items":[]}"#;
        let p_res = ij.parse("simple_schema", input);
        is_invertible(p_res, input);
        assert_eq!(ij.vms.len(), 1);
    }

    #[wasm_bindgen_test]
    fn test_parse_array_in_obj() {
        let ij = simple_test_init();
        let input = r#"{"obj":{"items":[]}}"#;
        let p_res = ij.parse("simple_schema", input);
        is_invertible(p_res, input);
        assert_eq!(ij.vms.len(), 1);
    }

    #[wasm_bindgen_test]
    fn test_parse_i_number_double_huge_neg_exp() {
        let ij = simple_test_init();
        let input = r#"{"val":123.456e-789}"#;
        let p_res = ij.parse("simple_schema", input);
        encodes_to(p_res, input, r#"{"val":0}"#);
        assert_eq!(ij.vms.len(), 1);
    }


}