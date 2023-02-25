use crate::configuration::config::Config;
use crate::entities::repo_root::RepoRoot;
use crate::result::RunnerErr;
use crate::use_cases::test_runner::{Runner, TestRunner, TestsStatus};

use cmd_lib::run_cmd;
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

impl Runner for DefaultTestRunner {
    #[instrument(skip(self))]
    fn run_all(&self, repo_root: RepoRoot) -> Result<TestsStatus, RunnerErr> {
        let repo_root = repo_root.to_string();
        debug!("running tests in {repo_root}");
        let test_tool = &self.cfg.cmd.tool;
        let test_args = &self.cfg.cmd.args;
        if let Err(e) = run_cmd!(cd $repo_root ; $test_tool $test_args) {
            debug!("tests failed: {e}");
            Ok(TestsStatus::Failure)
        } else {
            debug!("tests succeeded");
            Ok(TestsStatus::Success)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::{config::Cmd, tracing::init_tracing};

    use anyhow::Result;
    use tempfile::tempdir;

    #[test]
    fn when_tests_fail_then_failure_status_is_returned() -> Result<()> {
        // given
        init_tracing();
        let cfg = Config {
            ignored_paths: Vec::new(),
            cmd: Cmd::new("cargo", "test"),
        };
        let runner = DefaultTestRunner::make(cfg);
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
        let cfg = Config {
            ignored_paths: Vec::new(),
            cmd: Cmd::new("cargo", "test"),
        };
        let runner = DefaultTestRunner::make(cfg);
        let project_path = tmpdir_path.join("test_project");
        let root = RepoRoot::new(project_path);

        // when
        let res = runner.run_all(root)?;

        // then
        assert_eq!(res, TestsStatus::Success);

        Ok(())
    }
}
