use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use clash::Clash;

use super::*;

pub fn sample_puzzle(name: &str) -> Result<Clash> {
    let puzzle_file: PathBuf = ["fixtures", "puzzles", format!("{}.json", name).as_str()].iter().collect();
    let contents = std::fs::read_to_string(&puzzle_file)
        .with_context(|| format!("Unable to test puzzle with name {}", name))?;
    let clash: Clash = serde_json::from_str(&contents)
        .with_context(|| format!("Unable to deserialize test puzzle {} from {:?}", name, &puzzle_file))?;

    Ok(clash)
}

pub fn sh_cmd(script: &str) -> Command {
    let mut command = Command::new("sh");
    command.arg("-c").arg(script);
    command
}
