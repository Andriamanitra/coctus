use crate::outputstyle::OutputStyle;
use super::test_run::TestRun;

pub struct SuiteRun(Vec<TestRun>);

impl SuiteRun {
    pub fn new(results: Vec<TestRun>) -> Self {
        Self(results)
    }

    pub fn results(&self) -> &Vec<TestRun> {
        &self.0
    }

    pub fn tests_count(&self) -> usize {
        self.0.len()
    }

    pub fn is_successful(&self) -> bool  {
        self.0.last().expect("At least one test should have been run")
            .is_successful()
    }

    pub fn print_mistakes(&self, style: OutputStyle) {
        for failed_run in self.0.iter().filter(|run| !run.is_successful() ) {
            failed_run.print_mistakes(&style);
        }
    }
}
