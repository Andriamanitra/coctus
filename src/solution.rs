mod test_run;

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
    testcases.into_iter().map(|test|
        run_testcase(test, run_command, timeout)
    )
}

fn run_testcase<'a>(test: &'a TestCase, run_command: &mut Command, timeout: &Duration) -> TestRun<'a> {
    let mut run = match run_command
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(run) => run,
        Err(error) => {
            return TestRun::new(
                test,
                TestResult::UnableToRun {
                    error_msg: format!(
                        "{}: {}",
                        run_command.get_program().to_str().unwrap_or("Unable to run command"),
                        error
                    ),
                },
            )
        }
    };

    let mut stdin = run.stdin.as_mut().unwrap();
    std::io::Write::write(&mut stdin, test.test_in.as_bytes())
        .expect("Fatal error: could not write to stdin.");

    let timed_out = match run.wait_timeout(*timeout).expect("Could not wait for program execution.") {
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
        TestResult::Success
    } else if timed_out {
        TestResult::Timeout { stdout, stderr }
    } else if output.status.success() {
        TestResult::WrongOutput { stdout, stderr }
    } else {
        TestResult::RuntimeError { stdout, stderr }
    };

    TestRun::new(test, result)
}
