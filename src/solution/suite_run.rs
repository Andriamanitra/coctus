use std::process::Command;
use std::time::Duration;
use std::vec::IntoIter;

use wait_timeout::ChildExt;

use super::test_run::{TestRun, TestRunResult};
use crate::clash::TestCase;

pub struct SuiteRun<'a> {
    testcases: IntoIter<&'a TestCase>,
    run_command: Command,
    timeout: Duration,
}

impl<'a> Iterator for SuiteRun<'a> {
    type Item = TestRun<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let test = match self.testcases.next() {
            Some(testcase) => testcase,
            None => return None,
        };

        let run = self.run_testcase(test);
        Some(run)
    }
}

impl<'a> SuiteRun<'a> {
    pub fn new(testcases: Vec<&'a TestCase>, run_command: Command, timeout: Duration) -> Self {
        Self {
            testcases: testcases.into_iter(),
            run_command,
            timeout,
        }
    }

    fn run_testcase(&mut self, test: &'a TestCase) -> TestRun<'a> {
        let mut run = self
            .run_command
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to run --command");

        let mut stdin = run.stdin.as_mut().unwrap();
        std::io::Write::write(&mut stdin, test.test_in.as_bytes())
            .expect("Fatal error: could not write to stdin.");

        let timed_out = match run.wait_timeout(self.timeout).expect("Could not wait for program execution.") {
            Some(_) => false,
            None => {
                run.kill().expect("Failed to kill test run");
                true
            }
        };

        let output = run.wait_with_output().expect("Could not wait for program execution.");
        let stdout = String::from_utf8(output.stdout).unwrap_or_default();
        let stdout = stdout.replace("\r\n", "\n").trim_end().to_string();
        let stderr = String::from_utf8(output.stderr).unwrap_or_default();
        let result = if stdout == test.test_out.trim_end() {
            TestRunResult::Success
        } else if timed_out {
            TestRunResult::Timeout { stdout, stderr }
        } else if output.status.success() {
            TestRunResult::WrongOutput { stdout, stderr }
        } else {
            TestRunResult::RuntimeError { stdout, stderr }
        };

        TestRun::new(test, result)
    }
}
