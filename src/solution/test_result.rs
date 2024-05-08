pub enum CommandExit {
    Ok,
    Error,
    Timeout,
}

/// Represents outcome of running a test case. [TestResult::Success] means the
/// output of a solution command matched the `test_out` field of the
/// [TestCase](crate::clash::TestCase).
#[derive(Debug, Clone)]
pub enum TestResult {
    /// Solution command produced the expected output. A test run is considered
    /// a success even if it runs into a runtime error or times out if  its
    /// output was correct (just like it works on CodinGame).
    Success,
    /// Solution command failed to run. This may happen if the executable does
    /// not exist or current user does not have permissions to run it.
    UnableToRun { error_msg: String },
    /// Solution command exited normally but didn't produce the expected output.
    WrongOutput { stdout: String, stderr: String },
    /// Solution command encountered a runtime error (exited non-zero).
    RuntimeError { stdout: String, stderr: String },
    /// Solution command timed out.
    Timeout { stdout: String, stderr: String },
}

impl TestResult {
    pub(crate) fn from_output(
        expected: &str,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
        exit_status: CommandExit,
    ) -> Self {
        let stdout = String::from_utf8(stdout)
            .unwrap_or_default()
            .replace("\r\n", "\n")
            .trim_end()
            .to_string();
        let stderr = String::from_utf8(stderr).unwrap_or_default();

        match exit_status {
            _ if stdout == expected.trim_end() => TestResult::Success,
            CommandExit::Timeout => TestResult::Timeout { stdout, stderr },
            CommandExit::Ok => TestResult::WrongOutput { stdout, stderr },
            CommandExit::Error => TestResult::RuntimeError { stdout, stderr },
        }
    }

    /// Returns true if the test case passed. A test cases passes if the output
    /// of the solution command matches the expected output.
    pub fn is_success(&self) -> bool {
        matches!(self, TestResult::Success)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_testresult_success() {
        let result = TestResult::from_output("123", "123".into(), vec![], CommandExit::Ok);
        assert!(matches!(result, TestResult::Success));
    }

    #[test]
    fn test_testresult_success_with_trailing_whitespace() {
        let result = TestResult::from_output("abc\n", "abc".into(), vec![], CommandExit::Ok);
        assert!(matches!(result, TestResult::Success));
        let result = TestResult::from_output("abc", "abc\r\n".into(), vec![], CommandExit::Ok);
        assert!(matches!(result, TestResult::Success));
    }

    #[test]
    fn test_testresult_success_normalized_line_endings() {
        let result = TestResult::from_output("a\nb\nc", "a\r\nb\r\nc".into(), vec![], CommandExit::Ok);
        assert!(matches!(result, TestResult::Success));
    }

    #[test]
    fn test_testresult_success_on_timeout() {
        let result = TestResult::from_output("123", "123".into(), vec![], CommandExit::Timeout);
        assert!(
            matches!(result, TestResult::Success),
            "TestResult should be `Success` when stdout is correct even if execution timed out"
        )
    }

    #[test]
    fn test_testresult_success_on_runtime_error() {
        let result = TestResult::from_output("123", "123".into(), vec![], CommandExit::Error);
        assert!(
            matches!(result, TestResult::Success),
            "TestResult should be `Success` when stdout is correct even if a runtime error occurred"
        )
    }

    #[test]
    fn test_testresult_wrong_output() {
        let result = TestResult::from_output("x\ny\nz", "yyy".into(), "zzz".into(), CommandExit::Ok);
        match result {
            TestResult::WrongOutput { stdout, stderr } => {
                assert_eq!(stdout, "yyy");
                assert_eq!(stderr, "zzz");
            }
            other => panic!("expected TestResult::WrongOutput but found {:?}", other),
        }
    }

    #[test]
    fn test_testresult_timed_out() {
        let result = TestResult::from_output("xxx", "yyy".into(), "zzz".into(), CommandExit::Timeout);
        match result {
            TestResult::Timeout { stdout, stderr } => {
                assert_eq!(stdout, "yyy");
                assert_eq!(stderr, "zzz");
            }
            other => panic!("expected TestResult::Timeout but found {:?}", other),
        }
    }

    #[test]
    fn test_testresult_runtime_error() {
        let result = TestResult::from_output("xxx", "yyy".into(), "zzz".into(), CommandExit::Error);
        match result {
            TestResult::RuntimeError { stdout, stderr } => {
                assert_eq!(stdout, "yyy");
                assert_eq!(stderr, "zzz");
            }
            other => panic!("expected TestResult::RuntimeError but found {:?}", other),
        }
    }
}
