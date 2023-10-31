use super::test_run::TestRun;

pub struct SuiteRun(Vec<TestRun>);

impl SuiteRun {
    pub fn new(results: Vec<TestRun>) -> Self {
        Self(results)
    }

    pub fn results(&self) -> &Vec<TestRun> {
        &self.0
    }

    pub fn is_successful(&self) -> bool  {
        self.0.last().expect("At least one test should have been run")
            .is_successful()
    }
}
