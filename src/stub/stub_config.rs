use std::fs;
use std::path::Path;
use tera::Tera;
use include_dir::include_dir;
use super::Language;

use anyhow::{anyhow, Context, Result};

const HARDCODED_TEMPLATE_DIR: include_dir::Dir<'static> =
    include_dir!("$CARGO_MANIFEST_DIR/config/stub_templates");

#[derive(Clone)]
pub struct StubConfig {
    pub language: Language,
    pub tera: Tera,
}

impl StubConfig {
    /// This function is responsible for searching locations where language config
    /// files can be stored.
    ///
    /// This function checks the following locations (earlier items take precedence)
    /// - in the user config dir: `stub_templates/#{lang_arg}/stub_config.toml`
    /// - in the user config dir: `stub_templates/*/stub_config.toml` where
    ///   `lang_arg` is an alias
    /// - in embedded templates: `stub_templates/#{lang_arg}/stub_config.toml`
    /// - in embedded templates: `stub_templates/*/stub_config.toml` where `lang_arg`
    ///   is an alias
    ///
    /// where the user config dir is in `~/.config/clash` (Linux)
    /// and the embedded templates are under `config/stub_templates` in this repo
    pub fn find_stub_config(lang_name: &str, config_path: &Path) -> Result<Self> {
        let lang_name = &lang_name.to_lowercase();
        Ok(
            Self::find_user_folder_config(lang_name, config_path)
                .context("Unrecoverable error loaing language from user config dir")?
                .or_else(|| Self::find_hardcoded_config(lang_name).ok()).unwrap()
        )

    }

    /// Looks for Language definition in hardcoded stub templates
    /// compiled into the binary using the `include_dir` crate.
    ///
    /// It is intended to be a final source for a configuration,
    /// therefore it cannot be None.
    ///
    /// # Errors
    ///
    /// - An empty `stub_config.toml`
    /// - serde Deserialize failure
    /// - Failure to read hardcoded template "files"
    pub fn find_hardcoded_config(lang_name: &str) -> Result<Self> {
        let lang = Self::hardcoded_lang_by_name(lang_name)
            .context(format!("Failed to load hardcoded config for {}, please report", lang_name))?
            .or_else(|| Self::hardcoded_lang_by_alias(lang_name))
            .ok_or(anyhow!("Could not find hardcoded language config"))?;

        Ok(Self {
            tera: Self::build_tera_from_embedded_config(&lang.name)?,
            language: lang,
        })
    }

    // Tries to find a folder in the binary-embedded `config` folder
    // and parse its `stub_config.toml`.
    fn hardcoded_lang_by_name(input_lang_name: &str) -> Result<Option<Language>> {
        match HARDCODED_TEMPLATE_DIR.get_file(&format!("{input_lang_name}/stub_config.toml")) {
            Some(config_file) => {
                let config_file_content = config_file
                    .contents_utf8()
                    .context(format!("Could not get contents of config file for {}", input_lang_name))?;

                Ok(toml::from_str(config_file_content)
                    .context("There was an error loading the stub configuration")?)
            }
            None => Ok(None),
        }
    }

    // Looks through every template folder in the binary-embedded Strings and
    // loads them until it finds one that has `input_lang_name` listed as an alias.
    fn hardcoded_lang_by_alias(input_lang_name: &str) -> Option<Language> {
        let Ok(mut config_files) = HARDCODED_TEMPLATE_DIR.find("*/stub_config.toml") 
            else { return None };

        config_files.find_map(|dir_entry| {
            let config_file = dir_entry.as_file()?;
            let lang: Language = toml::from_str(config_file.contents_utf8()?).ok()?;

            if lang.aliases.clone()?.contains(&input_lang_name.to_string()) {
                Some(lang)
            } else {
                None
            }
        })
    }

    // Creates a new Tera instance from embedded Strings which Tera does not really
    // support. Reads every file into a string as a workaround and uses raw
    // templates.
    fn build_tera_from_embedded_config(lang_name: &str) -> Result<Tera> {
        let templates = HARDCODED_TEMPLATE_DIR
            .find(&format!("{}/*.jinja", lang_name))
            .context("Could not read embedded template files")?
            .filter_map(|dir_entry| {
                let file = dir_entry.as_file()?;

                Some((file.path().file_name()?.to_str()?, file.contents_utf8()?))
            });

        let mut tera = Tera::default();

        tera.add_raw_templates(templates)
            .context("Failed to load templates into Tera, this should not have happened")?;

        Ok(tera)
    }

    /// Looks for Language config definition in stub templates in the user's
    /// config directory (i. e. `~/.config/clash/stub_templates/` for linux).
    ///
    /// It returns a `Result<Option<_>>` in order to differentiate from errors
    /// that ought to pause execution and be reported to the user (`Err`), or
    /// simply failure to find a configuration that matches the input query
    /// (`None`).
    ///
    /// # Errors
    ///
    /// - An empty `stub_config.toml`
    /// - serde Deserialize failure
    /// - Failure to convert folder file name to str from OsStr
    pub fn find_user_folder_config(lang_name: &str, config_path: &Path) -> Result<Option<Self>> {
        // If user does not have a config directory (with correct permissions) carry on without error
        if !config_path.is_dir() {
            return Ok(None);
        }

        let lang_result = Self::user_config_lang_by_name(lang_name, config_path)?
            .or_else(|| Self::user_config_lang_by_alias(lang_name, config_path));

        if let Some(lang) = lang_result {
            let cfg = Self {
                tera: Self::build_tera_from_user_folder(&lang.name, config_path)?,
                language: lang,
            };

            Ok(Some(cfg))
        } else { 
            Ok(None) 
        }

    }

    // Tries to find a folder in the user config dir
    // that matches `input_lang_name` and parse its `stub_config.toml`.
    fn user_config_lang_by_name(input_lang_name: &str, config_path: &Path) -> Result<Option<Language>> {
        let lang_dir = config_path.join(input_lang_name);

        if lang_dir.is_dir() {
            let config_file_content = fs::read_to_string(lang_dir.join("stub_config.toml"))
                .context(format!("Could not get contents of config file for {}", input_lang_name))?;

            Ok(toml::from_str(&config_file_content)
                .context("There was an error loading the stub configuration")?)
        } else {
            Ok(None)
        }
    }

    // Looks through every template folder in the user config dir and
    // loads them until it finds one that has `input_lang_name` listed as an alias.
    fn user_config_lang_by_alias(input_lang_name: &str, config_path: &Path) -> Option<Language> {
        std::fs::read_dir(config_path).expect("Should already be checked").find_map(|folder| {
            let folder_path = folder.ok()?.path();
            let language_config_filepath = format!("{}/stub_config.toml", folder_path.to_str()?);
            let config_file_content = fs::read_to_string(language_config_filepath).ok()?;

            let lang: Language = toml::from_str::<Language>(&config_file_content).ok()?;

            if lang.aliases.clone()?.contains(&input_lang_name.to_string()) {
                Some(lang)
            } else {
                None
            }
        })
    }

    // Creates a new Tera instance from real files in the filesystem
    // as opposed to Strings embedded in the binary.
    fn build_tera_from_user_folder(lang_name: &str, config_path: &Path) -> Result<Tera> {
        Tera::new(
            config_path
                .join(lang_name)
                .join("*.jinja")
                .to_str()
                .ok_or(anyhow!("Template file name could not be converted to str"))?,
        )
        .context("Failed to create Tera instance")
    }
}
