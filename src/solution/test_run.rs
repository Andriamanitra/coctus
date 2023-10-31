use crate::clash::TestCase;

#[derive(Debug, PartialEq)]
pub enum TestRunResult {
    Success,
    WrongOutput { stdout: String, stderr: String },
    RuntimeError { stdout: String, stderr: String },
}

pub struct TestRun {
    testcase: TestCase,
    result: TestRunResult,
}

impl TestRun {
    pub fn new(testcase: TestCase, result: TestRunResult) -> Self {
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
        }
    }

    pub fn is_successful(&self) -> bool {
        self.result == TestRunResult::Success
    }
}
