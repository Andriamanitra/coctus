use std::fs;

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub enum VariableNameFormat {
    SnakeCase,
    CamelCase,
    PascalCase,
}

impl VariableNameFormat {
    pub fn convert(&self, variable_name: &str) -> String {
        match self {
            Self::SnakeCase => Self::convert_to_snake_case(variable_name),
            Self::PascalCase => Self::covert_to_pascal_case(variable_name),
            Self::CamelCase => Self::covert_to_camel_case(variable_name),
        }
    }

    fn convert_to_snake_case(variable_name: &str) -> String {
        let word_break = Regex::new(r"([a-z])([A-Z])").unwrap();
        word_break
            .replace_all(variable_name, |caps: &regex::Captures| {
                format!("{}_{}", &caps[1], &caps[2].to_lowercase())
            })
            .to_lowercase()
            .to_string()
    }

    fn covert_to_pascal_case(variable_name: &str) -> String {
        variable_name[0..1].to_uppercase() + &variable_name[1..]
    }

    fn covert_to_camel_case(_variable_name: &str) -> String {
        todo!()
    }
}

#[derive(Deserialize, Debug)]
pub struct Language {
    pub name: String,
    pub variable_format: VariableNameFormat,
    pub source_file_ext: String,
    pub type_tokens: TypeTokens,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct TypeTokens {
    pub int: Option<String>,
    pub float: Option<String>,
    pub long: Option<String>,
    pub bool: Option<String>,
    pub word: Option<String>,
    pub string: Option<String>,
}

impl From<&str> for Language {
    fn from(value: &str) -> Self {
        let language_config_filepath = format!("config/stub_templates/{}/stub_config.toml", value);
        let config_file_content = fs::read_to_string(language_config_filepath)
            .unwrap_or_else(|_| panic!("No stub configuration exists for {}", value));

        toml::from_str(&config_file_content).expect("There was an error with the stub configuration")
    }
}

impl Language {
    pub fn template_glob(&self) -> String {
        format!("config/stub_templates/{}/*.{}.jinja", self.name, self.source_file_ext)
    }
}
