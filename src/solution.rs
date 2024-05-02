mod test_run;

use std::io::Write;
use std::process::Command;
use std::time::Duration;

pub use test_run::{TestResult, TestRun};
use wait_timeout::ChildExt;

use crate::clash::TestCase;

pub fn lazy_run<'a>(
    testcases: impl IntoIterator<Item = &'a TestCase>,
    run_command: &'a mut Command,
    timeout: &'a Duration,
) -> impl IntoIterator<Item = TestRun<'a>> {
    testcases.into_iter().map(|test| {
        let cmd_results = run_solution(run_command, &test.test_in, timeout);
        check_testcase(test, cmd_results)
    })
}

enum CmdStatus {
    Success,
    Timeout,
    Error,
    Failure,
}

fn run_solution(cmd: &mut Command, input: &str, timeout: &Duration) -> (CmdStatus, String, String) {
    let mut run = match cmd
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(run) => run,
        Err(error) => {
            return (
                CmdStatus::Failure,
                cmd.get_program().to_string_lossy().to_string(),
                error.to_string(),
            )
        }
    };

    run.stdin
        .as_mut()
        .expect("STDIN of child process should be captured")
        .write_all(input.as_bytes())
        .expect("STDIN of child process should be writable");

    let timed_out = run
        .wait_timeout(*timeout)
        .expect("Process should be able to wait for execution")
        .is_none();

    if timed_out {
        run.kill().expect("Process should have been killed");
    }

    let output = run.wait_with_output().expect("Process should allow waiting for its execution");

    let stdout = String::from_utf8(output.stdout)
        .unwrap_or_default()
        .replace("\r\n", "\n")
        .trim_end()
        .to_string();
    let stderr = String::from_utf8(output.stderr).unwrap_or_default();

    if timed_out {
        (CmdStatus::Timeout, stdout, stderr)
    } else if output.status.success() {
        (CmdStatus::Success, stdout, stderr)
    } else {
        (CmdStatus::Error, stdout, stderr)
    }
}

fn check_testcase<'a>(testcase: &'a TestCase, run_results: (CmdStatus, String, String)) -> TestRun<'a> {
    let result = match run_results {
        (CmdStatus::Success, stdout, stderr) => {
            if stdout == testcase.test_out.trim_end() {
                TestResult::Success
            } else {
                TestResult::WrongOutput { stdout, stderr }
            }
        }
        (CmdStatus::Timeout, stdout, stderr) => TestResult::Timeout { stdout, stderr },
        (CmdStatus::Error, stdout, stderr) => TestResult::RuntimeError { stdout, stderr },
        (CmdStatus::Failure, program_name, message) => {
            let error_msg = format!("{}: {}", program_name, message);
            TestResult::UnableToRun { error_msg }
        }
    };

    TestRun::new(testcase, result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passing_testcase() {
        let clash = crate::test_helper::sample_puzzle("stub_tester").unwrap();
        let testcase = clash.testcases().first().unwrap();

        let result = check_testcase(testcase, (CmdStatus::Success, "123".to_string(), String::new()));
        assert!(result.is_successful());
    }

    #[test]
    fn test_failing_testcase() {
        let clash = crate::test_helper::sample_puzzle("stub_tester").unwrap();
        let testcase = clash.testcases().first().unwrap();

        let result = check_testcase(testcase, (CmdStatus::Success, "1234".to_string(), String::new()));
        assert!(!result.is_successful());
    }
}
