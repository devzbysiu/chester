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

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::tracing::init_tracing;

    use anyhow::Result;
    use tempfile::tempdir;

    #[test]
    fn when_tests_fail_then_failure_status_is_returned() -> Result<()> {
        // given
        init_tracing();
        let runner = DefaultTestRunner::make();
        let invalid_repo_root = RepoRoot::new("/not/existing/path");

        // when
        let res = runner.run_all(invalid_repo_root)?;

        // then
        assert_eq!(res, TestsStatus::Failure);

        Ok(())
    }

    #[test]
    fn when_tests_succeed_then_success_status_is_returned() -> Result<()> {
        // given
        init_tracing();
        let tmpdir = tempdir()?;
        let tmpdir_path = tmpdir.path();
        run_cmd!(cd $tmpdir_path ; cargo new test_project)?;
        let runner = DefaultTestRunner::make();
        let project_path = tmpdir_path.join("test_project");
        let root = RepoRoot::new(project_path);

        // when
        let res = runner.run_all(root)?;

        // then
        assert_eq!(res, TestsStatus::Success);

        Ok(())
    }
}
