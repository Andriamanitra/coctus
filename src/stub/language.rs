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
    #[serde(deserialize_with = "deser_preprocessors", default)]
    pub preprocessors: Vec<Preprocessor>,
}

fn deser_preprocessors<'de, D>(deserializer: D) -> Result<Vec<Preprocessor>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let preprocessors: Vec<String> = Deserialize::deserialize(deserializer)?;
    let mut output: Vec<Preprocessor> = Vec::new();

    for preprocessor_name in preprocessors {
        match preprocessor_name.as_str() {
            "s-expression" => output.push(preprocessor::s_expressions::transform),
            _ => return Err(D::Error::custom(format!("preprocessor {preprocessor_name} not found."))),
        }
    }
    Ok(output)
}
