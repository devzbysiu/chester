use crate::entities::repo_root::RepoRoot;
use crate::result::RunnerErr;
use crate::use_cases::test_runner::{Runner, TestRunner, TestsStatus};

use cmd_lib::run_cmd;
use tracing::{debug, instrument};

#[derive(Debug)]
pub struct DefaultTestRunner;

impl DefaultTestRunner {
    pub fn make() -> TestRunner {
        Box::new(Self)
    }
}

impl Runner for DefaultTestRunner {
    #[instrument(skip(self))]
    fn run_all(&self, repo_root: RepoRoot) -> Result<TestsStatus, RunnerErr> {
        let repo_root = repo_root.to_string();
        debug!("running tests in {repo_root}");
        let cmd_res = run_cmd!(cd $repo_root ; cargo test);
        if cmd_res.is_ok() {
            debug!("tests succeeded");
            Ok(TestsStatus::Success)
        } else {
            debug!("tests failed: {cmd_res:?}");
            Ok(TestsStatus::Failure)
        }
    }
}
