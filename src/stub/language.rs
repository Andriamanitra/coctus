use serde::de::Error;
use serde::{Deserialize, Serialize};

mod variable_name_options;
use variable_name_options::VariableNameOptions;

use super::preprocessor::{self, Preprocessor};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub(super) struct TypeTokens {
    int: Option<String>,
    float: Option<String>,
    long: Option<String>,
    bool: Option<String>,
    word: Option<String>,
    string: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub(super) struct Language {
    pub variable_name_options: VariableNameOptions,
    pub source_file_ext: String,
    // NOTE: These comments are for a future PR
    //
    // Generic, used for either type keywords...:
    // - float    in:   float a_float;   (C)
    // - i32      in:   let an_int: i32; (Rust)
    // ... or parsing functions:
    // - int      in:   x = int(input()) i      (Python)
    // - StrToInt in:   x = StrToInt(Inputs[0]) (Pascal)
    pub type_tokens: TypeTokens,
    // But sometimes you need two tokens per type for a language.
    // - Int32 and StrToInt for Pascal.
    pub type_parsers: Option<TypeTokens>,
    #[serde(deserialize_with = "deser_preprocessor", default)]
    pub preprocessor: Option<Preprocessor>,
}

fn deser_preprocessor<'de, D>(deserializer: D) -> Result<Option<Preprocessor>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let preprocessor: String = Deserialize::deserialize(deserializer)?;
    match preprocessor.as_str() {
        "lisp-like" => Ok(Some(preprocessor::lisp_like::transform)),
        "forward-declarations" => Ok(Some(preprocessor::forward_declarations::transform)),
        _ => Err(D::Error::custom(format!("preprocessor {preprocessor} not found."))),
    }
}
