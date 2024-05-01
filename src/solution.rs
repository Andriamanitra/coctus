mod suite_run;
mod test_run;

use std::process::Command;
use std::time::Duration;

use suite_run::SuiteRun;
pub use test_run::{TestResult, TestRun};

use crate::clash::TestCase;

pub fn run(
    testcases: Vec<&TestCase>,
    run_command: Command,
    timeout: Duration,
) -> impl Iterator<Item = TestRun> {
    SuiteRun::new(testcases, run_command, timeout)
}
