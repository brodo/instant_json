use std::collections::HashMap;

use js_sys::{Object};
use pest_meta::optimizer::{optimize, OptimizedRule};
use pest_meta::parser::{self, Rule};
use pest_meta::validator::validate_pairs;
use pest_vm::Vm;
use wasm_bindgen::prelude::*;
use crate::error::InstantJsonError;
// use web_sys::console;
mod error;

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
    pub fn compile(&mut self, schema_name: &str, schema: &str) -> Result<(), JsError>{
        let rules = parse_pest(schema)?;
        self.vms.insert(schema_name.to_string(), Vm::new(rules));
        Ok(())
    }

    #[wasm_bindgen]
    pub fn parse(&self, schema_name: &str, json_str: &str) -> Result<Object, JsError>  {
        let vm = self.vms.get(schema_name).ok_or(InstantJsonError::NotFound)?;
        let _pairs = vm.parse("object", json_str)?;

        Ok(Object::new())
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
    use wasm_bindgen::JsValue;
    use crate::InstantJson;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::console;

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

    #[wasm_bindgen_test]
    fn test_parse_json() {
        let mut ij = InstantJson::new();
        let com_res = ij.compile("simple_schema", JSON_GRAMMAR);
        if let Err(e) = com_res {
            console::log_1(&e.into());
            assert!(false, "Error while compiling rules");
        }

        let p_res = ij.parse("simple_schema", r#"{"hello": 1}"#);
        assert!(p_res.is_ok());
        assert_eq!(ij.vms.len(), 1);
    }
}