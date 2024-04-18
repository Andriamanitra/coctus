use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use include_dir::include_dir;
use tera::Tera;

use super::Language;

const HARDCODED_TEMPLATE_DIR: include_dir::Dir<'static> =
    include_dir!("$CARGO_MANIFEST_DIR/config/stub_templates");

#[derive(Clone)]
pub struct StubConfig {
    pub language: Language,
    pub tera: Tera,
}

impl StubConfig {
    /// This function is responsible for searching locations where language
    /// config files can be stored.
    ///
    /// Language config files are stored in: (ordered by precedence)
    /// 1. The user config dir: `stub_templates/#{lang_arg}/stub_config.toml`
    /// 2. This repo, embedded into the binary:
    ///    `config/stub_templates/#{lang_arg}/stub_config.toml`
    ///
    /// where the user config dir is in `~/.config/clash` (Linux, see the
    /// [directories documentation](https://docs.rs/directories/latest/directories/struct.ProjectDirs.html#method.config_dir)
    /// for others).
    pub fn find_stub_config(lang_name: &str, config_path: &Path) -> Result<Self> {
        let user_config_lang_dir = config_path.join(lang_name);

        if user_config_lang_dir.is_file() {
            Self::read_from_dir(user_config_lang_dir)
        } else {
            Self::read_from_embedded(&lang_name.to_lowercase())
        }
    }

    pub fn read_from_dir(dir: std::path::PathBuf) -> Result<Self> {
        let fname = dir.join("stub_config.toml");
        let toml_str = fs::read_to_string(fname)?;
        let language: Language = toml::from_str(&toml_str)?;
        let jinja_glob = dir.join("*.jinja");
        let tera = Tera::new(jinja_glob.to_str().expect("language directory path should be valid utf8"))
            .context("Failed to create Tera instance")?;
        Ok(Self { language, tera })
    }

    pub fn read_from_embedded(lang_name: &str) -> Result<Self> {
        // If you just created a new template for a language and you get:
        // Error: No stub generator found for 'language'
        // you may need to recompile the binaries to update: `cargo build`
        let embedded_config_dir = HARDCODED_TEMPLATE_DIR
            .get_dir(lang_name)
            .context(format!("No stub generator found for '{lang_name}'"))?;
        let config_file = embedded_config_dir
            .get_file(format!("{lang_name}/stub_config.toml"))
            .expect("Embedded stub generators should have stub_config.toml");
        let toml_str = config_file
            .contents_utf8()
            .expect("Embedded stub_config.toml contents should be valid utf8");
        let language: Language = toml::from_str(toml_str)?;
        let templates = embedded_config_dir
            .find("*.jinja")
            .expect("*.jinja should be a valid glob pattern")
            .filter_map(|dir_entry| {
                let file = dir_entry.as_file()?;
                Some((file.path().file_name()?.to_str()?, file.contents_utf8()?))
            });

        let mut tera = Tera::default();

        tera.add_raw_templates(templates)
            .expect("Adding embedded templates to tera should not fail");
        Ok(Self { language, tera })
    }
}
