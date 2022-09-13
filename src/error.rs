use std::fmt::{Debug};

use pest::error::Error as PestError;
use pest_meta::parser::Rule;
use thiserror::Error;
use wasm_bindgen::JsError;
use crate::error::InstantJsonError::Multiple;


#[derive(Error, Debug, Clone)]
pub enum InstantJsonError {
    #[error("grammar parsing error: {0}")]
    GrammarParse(#[from] PestError<Rule>),
    #[error("json parsing error")]
    JsonParse {
        message: String
    },
    #[error("multiple errors: {0:#?}")]
    Multiple(Vec<PestError<Rule>>),
    #[error("not found error")]
    NotFound
}

impl From<Vec<PestError<Rule>>> for InstantJsonError {
    fn from(errors: Vec<PestError<Rule>>) -> Self {
        Multiple(errors)
    }
}

impl<'a> From<PestError<&'a str>> for InstantJsonError {
    fn from(pest_error: PestError<&'a str>) -> Self {
        InstantJsonError::JsonParse{
            message: pest_error.to_string()
        }
    }
}


