use std::process::Command;
use anyhow::{anyhow, Result};
use crate::Clash;
use crate::outputstyle::OutputStyle;

pub struct Solution {
    clash: Clash,
    build_command: String,
    run_command: String,
    style: OutputStyle,
}

impl Solution {
    pub fn new(clash: Clash, build_command: String, run_command: String, style: OutputStyle) -> Self {
        Self { clash, build_command, run_command, style }
    }
}

pub fn make_command(cmd_str: &str) -> Result<Command> {
    match shlex::split(cmd_str) {
        Some(shlexed_cmd) if shlexed_cmd.is_empty() => Err(anyhow!("COMMAND is required")),
        Some(shlexed_cmd) => {
            let exe = &shlexed_cmd[0];
            let exe_args = &shlexed_cmd[1..];
            let mut cmd = Command::new(exe);
            cmd.args(exe_args);
            Ok(cmd)
        }
        _ => Err(anyhow!("Invalid COMMAND")),
    }
}
