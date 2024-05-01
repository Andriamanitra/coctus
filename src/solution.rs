mod suite_run;
mod test_run;

use std::process::Command;
use std::time::Duration;

use anyhow::{anyhow, Result};
use suite_run::SuiteRun;
pub use test_run::{TestResult, TestRun};

use crate::clash::TestCase;

pub fn run(
    testcases: Vec<&TestCase>,
    run_command: Command,
    timeout: Duration,
) -> impl Iterator<Item = TestRun> {
    SuiteRun::new(testcases, run_command, timeout)
}

pub fn build(build_command: Option<Command>) -> Result<()> {
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
        return Err(anyhow!("Build failed"))
    }

    Ok(())
}
