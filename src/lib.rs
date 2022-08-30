mod error;

use std::collections::HashMap;
use pest::iterators::Pairs;
use pest_meta::optimizer::{optimize, OptimizedRule};
// use wasm_bindgen::prelude::*;
use pest_meta::parser::{self, Rule};
use pest_vm::Vm;


use crate::error::InstantJsonError;
use crate::InstantJsonError::Multiple;


pub struct InstantJson {
    vms: HashMap<String, Vm>
}

impl InstantJson {
    fn new() -> Self {
        InstantJson {
            vms: HashMap::new()
        }
    }

    fn compile<'a>(&'a mut self, schema_name: &str, schema: &'a str) -> Result<(), InstantJsonError>{
        let rules = parse_pest(schema)?;
        self.vms.insert(schema_name.to_string(), Vm::new(rules));
        Ok(())
    }

    fn parse<'a>(&'a self, schema_name: &str, json_str: &'a str) -> Result<Pairs<&str>, InstantJsonError>  {
        let vm = self.vms.get(schema_name).ok_or(InstantJsonError::NotFound)?;
        Ok(vm.parse("root", json_str)?)
    }
}



fn parse_pest(input: &str) -> Result<Vec<OptimizedRule>, InstantJsonError> {
    let pairs = parser::parse(Rule::grammar_rule, input)?;
    let ast = parser::consume_rules(pairs)?;
    Ok(optimize(ast.clone()))
}

#[cfg(test)]
mod tests {
    use crate::{InstantJson};

    #[test]
    fn test_parse_pest() {
        let input_grammar = r#"root = { "abc" }"#;
        let mut ij = InstantJson::new();
        ij.compile("simple_schema", input_grammar).expect("should compile");
        assert_eq!(ij.vms.len(), 1);
    }

    #[test]
    fn test_parse_json() {
        let input_grammar = r#"root = { "abc" }"#;
        let mut ij = InstantJson::new();
        ij.compile("simple_schema", input_grammar).expect("should compile");
        ij.parse("simple_schema", "abc").expect("should parse");
        assert_eq!(ij.vms.len(), 1);
    }
}