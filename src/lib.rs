use std::collections::HashMap;

use pest_meta::optimizer::{optimize, OptimizedRule};
use pest_meta::parser::{self, Rule};
use pest_meta::validator::validate_pairs;
use pest_vm::Vm;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::console_log;
use crate::error::InstantJsonError;
use crate::InstantJsonError::JsonParse;
use crate::json::JsonValue;
use crate::JsonValue::{JsonNumber, JsonObject, JsonString};

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
                            if let JsonObject(hm) = current_obj {
                                hm.insert(current_key.to_owned(), new_obj);
                                *current_obj = hm.get_mut(current_key).unwrap();
                                stack.push(child.into_inner());
                            }
                        }
                        "pair" => {
                            stack.push(child.clone().into_inner());
                            is_key = true;
                        }
                        "string" => {
                            let child_str = child.as_str();
                            if is_key {
                                current_key = &child_str[1..child_str.len() - 1];
                            } else {
                                if let JsonObject(hm) = current_obj {
                                    hm.insert(current_key.to_owned(), JsonString(child_str.to_string()));
                                }
                            }
                        }
                        "number" => {
                            if let JsonObject(hm) = current_obj {
                                let child_str = child.as_str();
                                let child_num: f64 = child_str.parse().unwrap();
                                hm.insert(current_key.to_owned(), JsonNumber(child_num));
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
        console_log!("parse result: {:?}", &root);
        serde_wasm_bindgen::to_value(&root).map_err(|_| { JsonParse { message: "invalid root".to_owned() }.into() })
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
    use wasm_bindgen_test::{console_log, wasm_bindgen_test};
    use web_sys::console;
    use js_sys::{JSON};
    use wasm_bindgen::{JsError, JsValue};

    static JSON_GRAMMAR: &str = include_str!("../examples/json.pest");

    #[wasm_bindgen_test]
    fn test_parse_pest() {
        let mut ij = InstantJson::new();
        let res = ij.compile("simple_schema", &JSON_GRAMMAR.replace("\n", ""));
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

    fn simple_test_validate(res: Result<JsValue, JsError>) {
        match res {
            Err(e) => {
                console::log_1(&e.into());
                assert!(false, "got error");
            }
            Ok(obj) => {
                console_log!("{}",JSON::stringify(&obj).unwrap());
            }
        }
    }

    #[wasm_bindgen_test]
    fn test_parse_simple_json_1() {
        let ij = simple_test_init();
        let p_res = ij.parse("simple_schema", r#"{"hello": 1}"#);
        simple_test_validate(p_res);
        assert_eq!(ij.vms.len(), 1);
    }

    #[wasm_bindgen_test]
    fn test_parse_simple_json_2() {
        let ij = simple_test_init();
        let p_res = ij.parse("simple_schema", r#"{"hello": {"world": 1}}"#);
        simple_test_validate(p_res);
        assert_eq!(ij.vms.len(), 1);
    }
}