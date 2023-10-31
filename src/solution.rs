use std::process::Command;
use anyhow::{anyhow, Result};
use crate::Clash;
use crate::outputstyle::OutputStyle;

mod suite_run;
use suite_run::SuiteRun;
mod test_run;
use test_run::TestRun;

use self::test_run::TestRunResult;

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

    pub fn run(&self, ignore_failures: bool) -> Result<SuiteRun> {
        self.build()?;

        let mut command = make_command(&self.run_command)
            .expect("Error parsing --command");

        let testcases = self.clash.testcases();
        let mut results: Vec<TestRun> = Vec::with_capacity(testcases.len());

        for test in testcases {
            let mut run = command
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn().expect("Failed to run --command");

            let mut stdin = run.stdin.as_mut().unwrap();
            std::io::Write::write(&mut stdin, test.test_in.as_bytes())
                .expect("Fatal error: could not write to stdin.");

            let output = run.wait_with_output().expect("Could not wait for program execution.");
            let stdout = String::from_utf8(output.stdout).unwrap_or(String::new());
            let stdout = stdout.replace("\r\n", "\n").trim_end().to_string();
            let stderr = String::from_utf8(output.stderr).unwrap_or(String::new());
            let result = if stdout == test.test_out.trim_end() {
                TestRunResult::Success
            } else if output.status.success() {
                TestRunResult::WrongOutput { stdout, stderr }
            } else {
                TestRunResult::RuntimeError { stdout, stderr }
            };

            let failure = result != TestRunResult::Success;
            results.push(TestRun::new(test.to_owned(), result));

            if !ignore_failures && failure { break }
        }

        Ok(SuiteRun::new(results))
    }

    fn build(&self) -> Result<()> {
        if self.build_command.is_empty() { return Ok(()) };

        let mut build_command = make_command(&self.build_command)?;
        let build = build_command.output()?;

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
