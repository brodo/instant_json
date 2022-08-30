use pest::error::Error as PestError;
use pest_meta::parser::Rule;
use thiserror::Error;
use crate::Multiple;

#[derive(Error, Debug, Clone)]
pub enum InstantJsonError<'a> {
    #[error("grammar parsing error")]
    GrammarParse(#[from] PestError<Rule>),
    #[error("json parsing error")]
    JsonParse(PestError<&'a str>),
    #[error("multiple errors")]
    Multiple {
        errors: Vec<PestError<Rule>>
    },
    #[error("not found error")]
    NotFound
}

impl From<Vec<PestError<Rule>>> for InstantJsonError<'_> {
    fn from(errors: Vec<PestError<Rule>>) -> Self {
        Multiple {
            errors
        }
    }
}

impl<'a> From<PestError<&'a str>> for InstantJsonError<'a> {
    fn from(pest_error: PestError<&'a str>) -> Self {
        InstantJsonError::JsonParse(pest_error)
    }
}