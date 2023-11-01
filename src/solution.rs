use std::process::Command;
use anyhow::{anyhow, Result};
use crate::Clash;

mod suite_run;
use suite_run::SuiteRun;
mod test_run;
use test_run::TestRun;

use self::test_run::TestRunResult;

pub struct Solution {
    clash: Clash,
    build_command: Option<Command>,
    run_command: Command,
}

impl Solution {
    pub fn new(clash: Clash, build_command: Option<Command>, run_command: Command) -> Self {
        Self { clash, build_command, run_command }
    }

    pub fn run(&mut self, ignore_failures: bool) -> Result<SuiteRun> {
        let testcases = self.clash.testcases();
        let mut results: Vec<TestRun> = Vec::with_capacity(testcases.len());

        for (test_i, test) in testcases.iter().enumerate() {
            let mut run = self.run_command
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
            println!("{}/{} tests passed", test_i, testcases.len());
        }

        Ok(SuiteRun::new(results))
    }

    pub fn build(&mut self) -> Result<()> {
        let command: &mut Command = self.build_command.as_mut().unwrap();
        
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

