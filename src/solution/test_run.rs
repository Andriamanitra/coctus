use crate::clash::TestCase;

pub enum CommandExit {
    Ok,
    Error,
    Timeout,
}

#[derive(Debug, Clone)]
pub enum TestResult {
    Success,
    UnableToRun { error_msg: String },
    WrongOutput { stdout: String, stderr: String },
    RuntimeError { stdout: String, stderr: String },
    Timeout { stdout: String, stderr: String },
}

impl TestResult {
    pub fn from_output(expected: &str, stdout: Vec<u8>, stderr: Vec<u8>, exit_status: CommandExit) -> Self {
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
}

#[derive(Debug, Clone)]
pub struct TestRun<'a> {
    testcase: &'a TestCase,
    result: TestResult,
}

impl<'a> TestRun<'a> {
    pub fn new(testcase: &'a TestCase, result: TestResult) -> Self {
        Self { testcase, result }
    }

    pub fn expected(&self) -> &str {
        &self.testcase.test_out
    }

    pub fn actual(&self) -> &str {
        match &self.result {
            TestResult::Success => self.expected(),
            TestResult::UnableToRun { .. } => "",
            TestResult::RuntimeError { stdout, .. } => stdout,
            TestResult::WrongOutput { stdout, .. } => stdout,
            TestResult::Timeout { stdout, .. } => stdout,
        }
    }

    pub fn is_successful(&self) -> bool {
        matches!(self.result, TestResult::Success)
    }

    pub fn testcase(&self) -> &'a TestCase {
        self.testcase
    }

    pub fn result(&'a self) -> &'a TestResult {
        &self.result
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
