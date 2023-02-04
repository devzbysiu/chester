use crate::result::RunnerErr;
use crate::use_cases::test_runner::{Runner, TestRunner, TestsStatus};

pub struct DefaultTestRunner;

impl DefaultTestRunner {
    pub fn make() -> TestRunner {
        Box::new(Self)
    }
}

impl Runner for DefaultTestRunner {
    fn run(&self) -> Result<TestsStatus, RunnerErr> {
        todo!()
    }
}
