use crate::clash::TestCase;

#[derive(Debug, PartialEq, Clone)]
pub enum TestRunResult {
    Success,
    WrongOutput { stdout: String, stderr: String },
    RuntimeError { stdout: String, stderr: String },
    Timeout { stdout: String, stderr: String },
}

#[derive(Clone)]
pub struct TestRun<'a> {
    testcase: &'a TestCase,
    result: TestRunResult,
}

impl<'a> TestRun<'a> {
    pub fn new(testcase: &'a TestCase, result: TestRunResult) -> Self {
        Self { testcase, result }
    }

    pub fn expected(&self) -> &String {
        &self.testcase.test_out
    }

    pub fn actual(&self) -> &String {
        match &self.result {
            TestRunResult::Success => self.expected(),
            TestRunResult::RuntimeError { stdout, .. } => stdout,
            TestRunResult::WrongOutput { stdout, .. } => stdout,
            TestRunResult::Timeout { stdout, .. } => stdout,
        }
    }

    pub fn is_successful(&self) -> bool {
        self.result == TestRunResult::Success
    }

    pub fn testcase(&self) -> &'a TestCase {
        self.testcase
    }

    pub fn result(&'a self) -> &'a TestRunResult {
        &self.result
    }
}
