use std::{fs, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use include_dir::include_dir;
use tera::Tera;

use crate::stub::VariableCommand;

const HARDCODED_TEMPLATE_DIR: include_dir::Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/config/stub_templates");

lazy_static! {
    static ref SC_WORD_BREAK: Regex = Regex::new(r"([a-z])([A-Z])").unwrap();
    static ref PC_WORD_BREAK: Regex = Regex::new(r"([A-Z]*)([A-Z][a-z])").unwrap();
    static ref PC_WORD_END: Regex = Regex::new(r"([A-Z])([A-Z]*$)").unwrap();
}

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
        SC_WORD_BREAK
            .replace_all(variable_name, |caps: &regex::Captures| {
                format!("{}_{}", &caps[1], &caps[2].to_lowercase())
            })
            .to_lowercase()
            .to_string()
    }

    fn covert_to_pascal_case(variable_name: &str) -> String {
        variable_name[0..1].to_uppercase() + &Self::pascalize(&variable_name[1..])
    }

    fn covert_to_camel_case(variable_name: &str) -> String {
        variable_name[0..1].to_lowercase() + &Self::pascalize(&variable_name[1..])
    }

    fn pascalize(variable_slice: &str) -> String {
        let start_replaced = PC_WORD_BREAK.replace_all(variable_slice, |caps: &regex::Captures| {
            format!("{}{}", &caps[1].to_lowercase(), &caps[2])
        });

        PC_WORD_END
            .replace_all(&start_replaced, |caps: &regex::Captures| {
                format!("{}{}", &caps[1], &caps[2].to_lowercase())
            })
            .to_string()
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

#[derive(Deserialize, Debug, Clone)]
pub struct Language {
    pub name: String,
    pub variable_format: VariableNameFormat,
    pub allow_uppercase_vars: Option<bool>,
    pub source_file_ext: String,
    pub type_tokens: TypeTokens,
    pub keywords: Vec<String>,
    pub aliases: Option<Vec<String>>,

    #[serde(skip_deserializing)]
    pub tera: Tera,
}

fn is_uppercase_string(string: &str) -> bool {
    string.chars().all(|c| c.is_uppercase())
}

impl Language {
    pub fn transform_variable_name(&self, variable_name: &str) -> String {
        // CG has special treatment for variables with all uppercase identifiers
        // In most languages they remain uppercase regardless of variable format
        // In others (such as ruby where constants are uppercase) they get downcased
        let converted_variable_name = match (is_uppercase_string(variable_name), self.allow_uppercase_vars) {
            (true, Some(false)) => variable_name.to_lowercase(),
            (true, _) => variable_name.to_string(),
            (false, _) => self.variable_format.convert(variable_name),
        };

        self.escape_keywords(converted_variable_name)
    }

    pub fn transform_variable_command(&self, var: &VariableCommand) -> VariableCommand {
        VariableCommand {
            ident: self.transform_variable_name(&var.ident),
            var_type: var.var_type.clone(),
            input_comment: var.input_comment.clone(),
            max_length: var.max_length.clone(),
        }
    }

    pub fn escape_keywords(&self, variable_name: String) -> String {
        if self.keywords.contains(&variable_name) {
            format!("_{variable_name}")
        } else {
            variable_name
        }
    }

    pub fn from_hardcoded_config(value: &str) -> Result<Option<Self>> {
        Ok(
            Self::hardcoded_lang_by_name(&value.to_lowercase())?
                .or(Self::hardcoded_lang_by_alias(&value.to_lowercase())?)
        )
    }

    fn hardcoded_lang_by_name(name: &str) -> Result<Option<Language>> {
        match HARDCODED_TEMPLATE_DIR.get_file(&format!("{name}/stub_config.toml")) {
            Some(config_file) => {
                let config_file_content = config_file.contents_utf8()
                    .context(format!("No stub configuration exists for {}", name))?;

                let mut lang: Language = toml::from_str(config_file_content)
                    .context("There was an error loading the stub configuration")?;

                lang.attach_tera_from_embedded_config()?;

                Ok(Some(lang))
            }
            None => Ok(None)
        }
    }

    fn hardcoded_lang_by_alias(name: &str) -> Result<Option<Language>> {
        let mut config_files = HARDCODED_TEMPLATE_DIR.find("*/stub_config.toml")?;

        if let Some(mut lang) = config_files.find_map(|dir_entry| {
            let config_file = dir_entry.as_file()?;
            let lang: Self = toml::from_str(config_file.contents_utf8()?).ok()?;

            if lang.aliases.clone()?.contains(&name.to_string()) {
                Some(lang)
            } else {
                None
            }
        }) {
            lang.attach_tera_from_embedded_config()?;
            Ok(Some(lang))
        } else {
            Ok(None)
        }
    }

    fn attach_tera_from_embedded_config(&mut self) -> Result<()> {
        let template_files = HARDCODED_TEMPLATE_DIR.find(&format!("{}/*.jinja", self.name))
            .context("Could not read embedded template files")?
            .filter_map(|dir_entry| {
                let file = dir_entry.as_file()?;

                Some((
                    file.path().file_name()?.to_str()?,
                    file.contents_utf8()?
                ))
            });

        self.tera = Tera::default();

        self.tera.add_raw_templates(template_files)
            .context("Failed to load templates into Tera, this should not have happened")?;

        self.tera.build_inheritance_chains()
            .context("Failed to build tera inheritance chains, this should not have happened")?;

        Ok(())
    }

    pub fn find_in_user_config(name: &str, config_path: &PathBuf) -> Result<Option<Language>> {
        // If user does not have a config directory carry on without error
        if !config_path.exists() {
            return Ok(None);
        }

        match Self::user_config_lang_by_name(&name.to_lowercase(), config_path)? {
            Some(lang) => Ok(Some(lang)),
            None => Ok(Self::user_config_lang_by_alias(&name.to_lowercase(), config_path)?)
        }
    }

    fn user_config_lang_by_name<'a>(name: &'a str, config_path: &PathBuf) -> Result<Option<Language>> {
        let lang_dir = config_path.join(name);
        if lang_dir.is_dir() {
            let config_file_content = fs::read_to_string(lang_dir.join("stub_config.toml"))
                .context(format!("No stub configuration exists for {}", name))?;

            let mut lang: Self = toml::from_str(&config_file_content)
                .context("There was an error loading the stub configuration")?;

            lang.attach_tera_from_user_config(config_path)?;

            Ok(Some(lang))
        } else {
            Ok(None)
        }
    }

    fn user_config_lang_by_alias<'a>(name: &'a str, config_path: &PathBuf) -> Result<Option<Language>> {
        if let Some(mut lang) = std::fs::read_dir(config_path)?
            .find_map(|folder| {
                let folder_path = folder.ok()?.path();
                let language_config_filepath = format!("{}/stub_config.toml", folder_path.to_str()?);
                let config_file_content = fs::read_to_string(language_config_filepath).ok()?;
                
                let lang: Language = toml::from_str::<Language>(&config_file_content).ok()?;

                if lang.aliases.clone()?.contains(&name.to_string()) {
                    Some(lang)
                } else {
                    None
                }
            }) {
            lang.attach_tera_from_user_config(config_path)?;
            Ok(Some(lang))
        } else {
            Ok(None)
        }
    }

    fn attach_tera_from_user_config(&mut self, config_path: &PathBuf) -> Result<()> {
        self.tera = Tera::new(
            config_path.join(&self.name).join("*.jinja").to_str()
                .ok_or(anyhow!("Template file name could not be converted to str"))?
        ).context("Failed to create Tera instance")?;

        self.tera.build_inheritance_chains()?;

        Ok(())
    }
}
