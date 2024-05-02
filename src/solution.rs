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
    testcases.into_iter().map(|test| run_testcase(test, run_command, timeout))
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
            let program = run_command.get_program().to_str().unwrap_or("Unable to run command");
            let error_msg = format!("{}: {}", program, error);
            return TestRun::new(test, TestResult::UnableToRun { error_msg })
        }
    };

    run.stdin
        .as_mut()
        .expect("STDIN of child process should be captured")
        .write_all(test.test_in.as_bytes())
        .expect("STDIN of child process should be writable");

    TestRun::new(test, get_result(run, &test.test_out, timeout))
}

fn get_result(mut run: std::process::Child, expected: &str, timeout: &Duration) -> TestResult {
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

    if stdout == expected.trim_end() {
        TestResult::Success
    } else if timed_out {
        TestResult::Timeout { stdout, stderr }
    } else if output.status.success() {
        TestResult::WrongOutput { stdout, stderr }
    } else {
        TestResult::RuntimeError { stdout, stderr }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_passing_solution() {
//         let clash = crate::test_helper::sample_puzzle("stub_tester").unwrap();
//         let mut run_cmd = crate::test_helper::sh_cmd("read input; echo 123");
//         let timeout = Duration::from_secs(1);
//         assert!(lazy_run(clash.testcases(), &mut run_cmd, &timeout)
//             .into_iter()
//             .all(|test_run| test_run.is_successful()))
//     }

//     #[test]
//     fn test_failing_solution() {
//         let clash = crate::test_helper::sample_puzzle("stub_tester").unwrap();
//         let mut run_cmd = crate::test_helper::sh_cmd("read input; echo nada");
//         let timeout = Duration::from_secs(1);
//         assert!(lazy_run(clash.testcases(), &mut run_cmd, &timeout)
//             .into_iter()
//             .all(|test_run| !test_run.is_successful()))
//     }
// }
