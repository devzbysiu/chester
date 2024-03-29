use crate::configuration::config::Config;
use crate::entities::repo_root::RepoRoot;
use crate::result::CheckErr;
use crate::use_cases::check_runner::{CRunner, CheckRunStatus, CheckRunner};

use tracing::{debug, instrument};

/// It runs the command for a check stage. Command is passed in via `Config::check_cmd`.
///
/// The execution can fail in two ways. See the [`DefaultCheckRunner::run`] for details.
#[derive(Debug)]
pub struct DefaultCheckRunner {
    cfg: Config,
}

impl DefaultCheckRunner {
    pub fn make(cfg: Config) -> CheckRunner {
        Box::new(Self { cfg })
    }
}

impl CRunner for DefaultCheckRunner {
    /// Executes the `check_cmd` on the path specified by `repo_root`.
    ///
    /// The execution can fail in two ways:
    /// - there is an error while executing `check_cmd` command
    /// - the command succeeds, but there are issues with the code
    ///   (`check_cmd` exits with non-zero status code).
    #[instrument(skip(self))]
    fn run(&self, repo_root: RepoRoot) -> Result<CheckRunStatus, CheckErr> {
        debug!("running check in {repo_root}");
        let Ok(status) = self.cfg.check_cmd.status(&repo_root) else {
            debug!("command failed");
            return Ok(CheckRunStatus::Failure);
        };

        if !status.success() {
            debug!("check failed with: {status}");
            return Ok(CheckRunStatus::Failure);
        }

        debug!("check succeeded");
        Ok(CheckRunStatus::Success)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::config::ConfigBuilder;
    use crate::configuration::tracing::init_tracing;
    use crate::data_providers::command::Cmd;

    use anyhow::Result;
    use cmd_lib::run_cmd;
    use tempfile::tempdir;

    #[test]
    fn when_check_command_fail_then_failure_status_is_returned() -> Result<()> {
        // given
        init_tracing();
        let cfg = ConfigBuilder::default()
            .check_cmd(Cmd::new("cargo", &["check"]))
            .build()?;
        let runner = DefaultCheckRunner::make(cfg);
        let invalid_repo_root = RepoRoot::new("/not/existing/path");

        // when
        let res = runner.run(invalid_repo_root)?;

        // then
        assert_eq!(res, CheckRunStatus::Failure);

        Ok(())
    }

    #[test]
    fn when_check_fail_then_failure_status_is_returned() -> Result<()> {
        // given
        init_tracing();
        let tmpdir = tempdir()?;
        let tmpdir_path = tmpdir.path();
        run_cmd!(
            cd $tmpdir_path;
            cargo new test_project;
            echo "fn main() {" >> $tmpdir_path/test_project/src/main.rs
        )?;
        let cfg = ConfigBuilder::default()
            .check_cmd(Cmd::new("cargo", &["check"]))
            .build()?;
        let runner = DefaultCheckRunner::make(cfg);
        let project_path = tmpdir_path.join("test_project");
        let root = RepoRoot::new(project_path);

        // when
        let res = runner.run(root)?;

        // then
        assert_eq!(res, CheckRunStatus::Failure);

        Ok(())
    }

    #[test]
    fn when_check_succeed_then_success_status_is_returned() -> Result<()> {
        // given
        init_tracing();
        let tmpdir = tempdir()?;
        let tmpdir_path = tmpdir.path();
        run_cmd!(cd $tmpdir_path ; cargo new test_project)?;
        let cfg = ConfigBuilder::default()
            .check_cmd(Cmd::new("cargo", &["check"]))
            .build()?;
        let runner = DefaultCheckRunner::make(cfg);
        let project_path = tmpdir_path.join("test_project");
        let root = RepoRoot::new(project_path);

        // when
        let res = runner.run(root)?;

        // then
        assert_eq!(res, CheckRunStatus::Success);

        Ok(())
    }
}
