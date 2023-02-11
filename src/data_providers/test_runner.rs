use std::path::PathBuf;

use cmd_lib::run_cmd;

use crate::result::RunnerErr;
use crate::use_cases::test_runner::{Runner, TestRunner, TestsStatus};

pub struct DefaultTestRunner;

impl DefaultTestRunner {
    pub fn make() -> TestRunner {
        Box::new(Self)
    }
}

impl Runner for DefaultTestRunner {
    fn run_all(&self, _repo_root: PathBuf) -> Result<TestsStatus, RunnerErr> {
        if run_cmd!(cd ~/Projects/chester ; cargo test).is_ok() {
            Ok(TestsStatus::Success)
        } else {
            Ok(TestsStatus::Failure)
        }
    }
}
