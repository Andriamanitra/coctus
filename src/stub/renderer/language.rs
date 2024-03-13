use std::fs;

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Debug, Clone)]
pub struct Language {
    pub name: String,
    pub variable_format: VariableNameFormat,
    pub source_file_ext: String,
    pub type_tokens: TypeTokens,
    pub keywords: Vec<String>,
    pub aliases: Option<Vec<String>>,
}

impl Language {
    pub fn transform_variable_name(&self, variable_name: &str) -> String {
        self.escape_keywords(self.variable_format.convert(variable_name))
    }

    pub fn escape_keywords(&self, variable_name: String) -> String {
        if self.keywords.contains(&variable_name) {
            format!("_{variable_name}")
        } else {
            variable_name
        }
    }
}

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

impl TryFrom<&str> for Language {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let lang_folders: Vec<String> = std::fs::read_dir("config/stub_templates/")
            .context("Should have stub_templates directiory")?
            .map(|read_dir| read_dir.unwrap().file_name().into_string().expect("Template path must be UTF"))
            .collect();

        Self::find_lang_by_name(&value.to_lowercase(), &lang_folders)?
            .or(Self::find_lang_by_alias(&value.to_lowercase(), &lang_folders))
            .ok_or(anyhow!("Unsupported language: {}", value))
    }
}

impl Language {
    pub fn template_glob(&self) -> String {
        format!("config/stub_templates/{}/*.{}.jinja", self.name, self.source_file_ext)
    }

    fn find_lang_by_name<'a>(name: &'a str, lang_folders: &'a [String]) -> Result<Option<Language>> {
        if lang_folders.iter().any(|l| l == name) {
            let language_config_filepath = format!("config/stub_templates/{}/stub_config.toml", name);
            let config_file_content = fs::read_to_string(language_config_filepath)
                .context(format!("No stub configuration exists for {}", name))?;

            Ok(toml::from_str(&config_file_content)
                .context("There was an error loading the stub configuration")?)
        } else {
            Ok(None)
        }
    }

    fn find_lang_by_alias<'a>(name: &'a str, lang_folders: &'a [String]) -> Option<Language> {
        lang_folders
            .iter()
            .filter_map(|folder| {
                let language_config_filepath = format!("config/stub_templates/{}/stub_config.toml", folder);
                match fs::read_to_string(language_config_filepath) {
                    Ok(config_file_content) => toml::from_str::<Language>(&config_file_content).ok(),
                    _ => None,
                }
            })
            .find(|l| match &l.aliases {
                Some(aliases) => aliases.contains(&name.to_string()),
                None => false,
            })
    }
}
