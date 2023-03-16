use crate::configuration::config::Config;
use crate::entities::repo_root::RepoRoot;
use crate::result::RunnerErr;
use crate::use_cases::test_runner::{TRunner, TestRunner, TestsRunStatus};

use tracing::{debug, instrument};

#[derive(Debug)]
pub struct DefaultTestRunner {
    cfg: Config,
}

impl DefaultTestRunner {
    pub fn make(cfg: Config) -> TestRunner {
        Box::new(Self { cfg })
    }
}

impl TRunner for DefaultTestRunner {
    #[instrument(skip(self))]
    fn run(&self, repo_root: RepoRoot) -> Result<TestsRunStatus, RunnerErr> {
        let repo_root = repo_root.to_string();
        debug!("running tests in {repo_root}");
        let Ok(status) = self.cfg.tests_cmd.status(repo_root) else {
            debug!("command failed");
            return Ok(TestsRunStatus::Failure);
        };
        if !status.success() {
            debug!("tests failed with: {status}");
            return Ok(TestsRunStatus::Failure);
        }
        debug!("tests succeeded");
        Ok(TestsRunStatus::Success)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::config::{Cmd, ConfigBuilder};
    use crate::configuration::tracing::init_tracing;

    use anyhow::Result;
    use cmd_lib::run_cmd;
    use tempfile::tempdir;

    #[test]
    fn when_tests_fail_then_failure_status_is_returned() -> Result<()> {
        // given
        init_tracing();
        let cfg = ConfigBuilder::default()
            .tests_cmd(Cmd::new("cargo", &["test"]))
            .build()?;
        let runner = DefaultTestRunner::make(cfg);
        let invalid_repo_root = RepoRoot::new("/not/existing/path");

        // when
        let res = runner.run(invalid_repo_root)?;

        // then
        assert_eq!(res, TestsRunStatus::Failure);

        Ok(())
    }

    #[test]
    fn when_tests_succeed_then_success_status_is_returned() -> Result<()> {
        // given
        init_tracing();
        let tmpdir = tempdir()?;
        let tmpdir_path = tmpdir.path();
        run_cmd!(cd $tmpdir_path ; cargo new test_project)?;
        let cfg = ConfigBuilder::default()
            .tests_cmd(Cmd::new("cargo", &["test"]))
            .build()?;
        let runner = DefaultTestRunner::make(cfg);
        let project_path = tmpdir_path.join("test_project");
        let root = RepoRoot::new(project_path);

        // when
        let res = runner.run(root)?;

        // then
        assert_eq!(res, TestsRunStatus::Success);

        Ok(())
    }
}
