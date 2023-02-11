use crate::entities::repo_root::RepoRoot;
use crate::result::RunnerErr;
use crate::use_cases::test_runner::{Runner, TestRunner, TestsStatus};

use cmd_lib::run_cmd;

pub struct DefaultTestRunner;

impl DefaultTestRunner {
    pub fn make() -> TestRunner {
        Box::new(Self)
    }
}

impl Runner for DefaultTestRunner {
    fn run_all(&self, _repo_root: RepoRoot) -> Result<TestsStatus, RunnerErr> {
        if run_cmd!(cd ~/Projects/chester ; cargo test).is_ok() {
            Ok(TestsStatus::Success)
        } else {
            Ok(TestsStatus::Failure)
        }
    }
}
