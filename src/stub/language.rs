use serde::{Deserialize, Serialize};

mod variable_name_options;
use variable_name_options::VariableNameOptions;

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

#[derive(Deserialize, Debug, Clone)]
pub struct Language {
    pub name: String,
    pub variable_name_options: VariableNameOptions,
    pub source_file_ext: String,
    pub type_tokens: TypeTokens,
}
