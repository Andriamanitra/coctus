use std::fs;

use anyhow::{Context, Result};
use include_dir::include_dir;
use tera::Tera;

use super::Language;

const HARDCODED_EMBEDDED_TEMPLATE_DIR: include_dir::Dir<'static> =
    include_dir!("$CARGO_MANIFEST_DIR/config/stub_templates");

#[derive(Clone)]
pub struct StubConfig {
    pub(super) language: Language,
    pub(super) tera: Tera,
}

impl StubConfig {
    pub fn read_from_dir(dir: std::path::PathBuf) -> Result<Self> {
        let toml_file = dir.join("stub_config.toml");
        let toml_str = fs::read_to_string(toml_file)?;
        let language: Language = toml::from_str(&toml_str)?;
        let jinja_glob = dir.join("*.jinja");
        let tera = Tera::new(jinja_glob.to_str().expect("language directory path should be valid utf8"))
            .context("Failed to create Tera instance")?;
        Ok(Self { language, tera })
    }

    pub(super) fn read_from_embedded(lang_name: &str) -> Result<Self> {
        // If you just created a new template for a language and you get:
        // Error: No stub generator found for 'language'
        // you may need to recompile the binaries to update: `cargo build`
        let embedded_config_dir = HARDCODED_EMBEDDED_TEMPLATE_DIR
            .get_dir(lang_name)
            .context(format!("No stub generator found for '{lang_name}'"))?;
        let toml_file = embedded_config_dir
            .get_file(format!("{lang_name}/stub_config.toml"))
            .expect("Embedded stub generators should have stub_config.toml");
        let toml_str = toml_file
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

        match tera.add_raw_templates(templates) {
            Ok(_) => (),
            Err(err) => return Err(err.into()),
        }
        Ok(Self { language, tera })
    }
}
