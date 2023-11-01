use std::{process::Command, vec::IntoIter};

use crate::clash::TestCase;
use super::test_run::{TestRun, TestRunResult};

pub struct SuiteRun {
    testcases: IntoIter<TestCase>,
    run_command: Command,
}

impl Iterator for SuiteRun {
    type Item = TestRun;

    fn next(&mut self) -> Option<Self::Item> {
        let test = match self.testcases.next() {
            Some(testcase) => testcase,
            None => return None,
        };

        let run = self.run_testcase(test);
        Some(run)
    }
}

impl SuiteRun {
    pub fn new(testcases: Vec<TestCase>, run_command: Command) -> Self {
        Self { testcases: testcases.into_iter(),
               run_command }
    }

    fn run_testcase(&mut self, test: TestCase) -> TestRun {
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

        TestRun::new(test, result)
    }
}
