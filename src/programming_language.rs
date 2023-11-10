use std::fs;

use serde::{Deserialize, Serialize};
use toml;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum VariableNameFormat {
    SnakeCase,
    CamelCase,
    PascalCase,
}

#[derive(Deserialize, Debug)]
pub struct ProgrammingLanguage {
    pub name: String,
    pub variable_format: VariableNameFormat,
    pub source_file_ext: String,
    pub type_tokens: TypeTokens,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TypeTokens {
    pub int: Option<String>,
    pub float: Option<String>,
    pub long: Option<String>,
    pub bool: Option<String>,
    pub word: Option<String>,
    pub string: Option<String>,
}

impl From<String> for ProgrammingLanguage {
    fn from(value: String) -> Self {
        let language_config_filepath = format!("config/stub_templates/{}/stub_config.toml", value);
        let config_file_content = fs::read_to_string(language_config_filepath)
            .expect(&format!("No stub configuration exists for {}", value));

        toml::from_str(&config_file_content).expect("There was an error with the stub configuration")
    }
}

impl ProgrammingLanguage {
    pub fn template_glob(&self) -> String {
        format!("config/stub_templates/{}/*.{}.jinja", self.name, self.source_file_ext)
    }
}
