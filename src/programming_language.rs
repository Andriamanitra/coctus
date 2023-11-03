use std::fs;

use serde::Deserialize;
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
    name: String,
    variable_format: VariableNameFormat,
    source_file_ext: String,
}

impl From<String> for ProgrammingLanguage {
    fn from(value: String) -> Self {
        let language_config_filepath = format!("config/{}.toml", value);
        let config_file_content = fs::read_to_string(language_config_filepath)
            .expect(&format!("No stub configuration exists for {}", value));

        toml::from_str(&config_file_content).expect("There was an error with the stub configuration")
    }
}
