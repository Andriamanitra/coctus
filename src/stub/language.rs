use serde::de::Error;
use serde::{Deserialize, Serialize};

mod variable_name_options;
use variable_name_options::VariableNameOptions;

use super::preprocessor::{self, Preprocessor};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct TypeTokens {
    pub int: Option<String>,
    pub float: Option<String>,
    pub long: Option<String>,
    pub bool: Option<String>,
    pub word: Option<String>,
    pub string: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Language {
    pub name: String,
    pub variable_name_options: VariableNameOptions,
    pub source_file_ext: String,
    pub type_tokens: TypeTokens,
    #[serde(deserialize_with = "deser_preprocessor", default)]
    pub preprocessor: Option<Preprocessor>,
}

fn deser_preprocessor<'de, D>(deserializer: D) -> Result<Option<Preprocessor>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let preprocessor: String = Deserialize::deserialize(deserializer)?;
    match preprocessor.as_str() {
        "s-expression" => Ok(Some(preprocessor::s_expressions::transform)),
        _ => Err(D::Error::custom(format!("preprocessor {preprocessor} not found."))),
    }
}
