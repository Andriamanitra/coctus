use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use include_dir::include_dir;
use serde::{Deserialize, Serialize};
use tera::Tera;

mod variable_name_options;
use variable_name_options::VariableNameOptions;

const HARDCODED_TEMPLATE_DIR: include_dir::Dir<'static> =
    include_dir!("$CARGO_MANIFEST_DIR/config/stub_templates");

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
    pub aliases: Option<Vec<String>>,

    #[serde(skip_deserializing)]
    pub tera: Tera,
}

impl Language {
    /// Looks for Language definition in hardcoded stub templates
    /// compiled into the binary using the `include_dir` crate.
    ///
    /// It returns a `Result<Option<_>>` in order to differentiate from errors
    /// that ought to pause execution and be reported to the user (`Err`), or
    /// simply failure to find a configuration that matches the input query
    /// (`None`).
    ///
    /// Err scenarios include:
    /// - An empty `stub_config.toml`
    /// - serde Deserialize failure
    /// - Failure to read hardcoded template "files"
    pub fn from_hardcoded_config(input_lang_name: &str) -> Result<Option<Self>> {
        match Self::hardcoded_lang_by_name(&input_lang_name.to_lowercase())? {
            Some(lang) => Ok(Some(lang)),
            None => Self::hardcoded_lang_by_alias(&input_lang_name.to_lowercase())
        }
    }

    // Tries to find a folder in the binary-embedded `config` folder
    // and parse its `stub_config.toml`.
    fn hardcoded_lang_by_name(input_lang_name: &str) -> Result<Option<Language>> {
        match HARDCODED_TEMPLATE_DIR.get_file(&format!("{input_lang_name}/stub_config.toml")) {
            Some(config_file) => {
                let config_file_content = config_file
                    .contents_utf8()
                    .context(format!("Could not get contents of config file for {}", input_lang_name))?;

                let mut lang: Language = toml::from_str(config_file_content)
                    .context("There was an error loading the stub configuration")?;

                lang.attach_tera_from_embedded_config()?;

                Ok(Some(lang))
            }
            None => Ok(None),
        }
    }

    // Looks through every template folder in the binary-embedded Strings and
    // loads them until it finds one that has `input_lang_name` listed as an alias.
    fn hardcoded_lang_by_alias(input_lang_name: &str) -> Result<Option<Language>> {
        let mut config_files = HARDCODED_TEMPLATE_DIR.find("*/stub_config.toml")?;

        let lang_result = config_files.find_map(|dir_entry| {
            let config_file = dir_entry.as_file()?;
            let lang: Self = toml::from_str(config_file.contents_utf8()?).ok()?;

            if lang.aliases.clone()?.contains(&input_lang_name.to_string()) {
                Some(lang)
            } else {
                None
            }
        });

        if let Some(mut lang) = lang_result {
            lang.attach_tera_from_embedded_config()?;
            Ok(Some(lang))
        } else {
            Ok(None)
        }
    }

    // Creates a new Tera instance from embedded Strings which Tera does not really
    // support. Reads every file into a string as a workaround and uses raw
    // templates.
    fn attach_tera_from_embedded_config(&mut self) -> Result<()> {
        let template_files = HARDCODED_TEMPLATE_DIR
            .find(&format!("{}/*.jinja", self.name))
            .context("Could not read embedded template files")?
            .filter_map(|dir_entry| {
                let file = dir_entry.as_file()?;

                Some((file.path().file_name()?.to_str()?, file.contents_utf8()?))
            });

        self.tera = Tera::default();

        self.tera
            .add_raw_templates(template_files)
            .context("Failed to load templates into Tera, this should not have happened")?;

        Ok(())
    }

    /// Looks for Language definition in stub templates in the user's
    /// config directory (i. e. `~/.config/clash/stub_templates/` for linux).
    ///
    /// It returns a `Result<Option<_>>` in order to differentiate from errors
    /// that ought to pause execution and be reported to the user (`Err`), or
    /// simply failure to find a configuration that matches the input query
    /// (`None`).
    ///
    /// Err scenarios include:
    /// - An empty `stub_config.toml`
    /// - serde Deserialize failure
    /// - Failure to convert folder file name to str from OsStr
    pub fn from_user_config(input_lang_name: &str, config_path: &Path) -> Result<Option<Language>> {
        // If user does not have a config directory carry on without error
        if !config_path.exists() {
            return Ok(None);
        }

        match Self::user_config_lang_by_name(&input_lang_name.to_lowercase(), config_path)? {
            Some(lang) => Ok(Some(lang)),
            None => Ok(Self::user_config_lang_by_alias(&input_lang_name.to_lowercase(), config_path)?),
        }
    }

    // Tries to find a folder in the user config dir
    // that matches `input_lang_name` and parse its `stub_config.toml`.
    fn user_config_lang_by_name(input_lang_name: &str, config_path: &Path) -> Result<Option<Language>> {
        let lang_dir = config_path.join(input_lang_name);

        if lang_dir.is_dir() {
            let config_file_content = fs::read_to_string(lang_dir.join("stub_config.toml"))
                .context(format!("Could not get contents of config file for {}", input_lang_name))?;

            let mut lang: Self = toml::from_str(&config_file_content)
                .context("There was an error loading the stub configuration")?;

            lang.attach_tera_from_user_config(config_path)?;

            Ok(Some(lang))
        } else {
            Ok(None)
        }
    }

    // Looks through every template folder in the user config dir and
    // loads them until it finds one that has `input_lang_name` listed as an alias.
    fn user_config_lang_by_alias(input_lang_name: &str, config_path: &Path) -> Result<Option<Language>> {
        let lang_result = std::fs::read_dir(config_path)?.find_map(|folder| {
            let folder_path = folder.ok()?.path();
            let language_config_filepath = format!("{}/stub_config.toml", folder_path.to_str()?);
            let config_file_content = fs::read_to_string(language_config_filepath).ok()?;

            let lang: Language = toml::from_str::<Language>(&config_file_content).ok()?;

            if lang.aliases.clone()?.contains(&input_lang_name.to_string()) {
                Some(lang)
            } else {
                None
            }
        });

        if let Some(mut lang) = lang_result {
            lang.attach_tera_from_user_config(config_path)?;
            Ok(Some(lang))
        } else {
            Ok(None)
        }
    }

    // Creates a new Tera instance from real files in the filesystem
    // as opposed to Strings embedded in the binary.
    fn attach_tera_from_user_config(&mut self, config_path: &Path) -> Result<()> {
        self.tera = Tera::new(
            config_path
                .join(&self.name)
                .join("*.jinja")
                .to_str()
                .ok_or(anyhow!("Template file name could not be converted to str"))?,
        )
        .context("Failed to create Tera instance")?;

        Ok(())
    }
}
