use std::process::Command;
use anyhow::{anyhow, Result};
use crate::Clash;

mod suite_run;
use suite_run::SuiteRun;
mod test_run;

pub struct Solution {
    clash: Clash,
}

impl Solution {
    pub fn new(clash: Clash) -> Self {
        Self { clash }
    }

    pub fn run(&mut self, run_command: Command) -> Result<SuiteRun> {
        Ok(SuiteRun::new(self.clash.testcases().to_owned(), run_command))
    }

    pub fn build(&mut self, build_command: Option<Command>) -> Result<()> {
        let mut command: Command = match build_command {
            Some(cmd) => cmd,
            None => return Ok(()),
        };
        
        let build = command.output()?;

        if !build.status.success() {
            if !build.stderr.is_empty() {
                println!("Build command STDERR:\n{}", String::from_utf8(build.stderr)?);
            }
            if !build.stdout.is_empty() {
                println!("Build command STDOUT:\n{}", String::from_utf8(build.stdout)?);
            }
            return Err(anyhow!("Build failed"));
        }

        Ok(())
    }
}

