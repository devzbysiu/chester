use crate::configuration::config::Config;
use crate::entities::repo_root::RepoRoot;
use crate::result::CoverageErr;
use crate::use_cases::coverage_runner::{CovRunner, CoverageRunStatus, CoverageRunner};

use tracing::{debug, error, instrument};

/// It runs the command for a coverage stage. Command is passed in via `Config::coverage_cmd`.
///
/// The execution can fail. See the [`DefaultCoverageRunner::run`] for details.
#[derive(Debug)]
pub struct DefaultCoverageRunner {
    cfg: Config,
}

impl DefaultCoverageRunner {
    pub fn make(cfg: Config) -> CoverageRunner {
        Box::new(Self { cfg })
    }
}

impl CovRunner for DefaultCoverageRunner {
    /// It executes `coverage_cmd` on a path specified by `repo_root` and parses the output
    /// to read the code coverage by using output parser from `coverage_cmd`.
    ///
    /// It can fail in a few ways:
    /// - there was an error while running the command (for example no binary in PATH)
    /// - there is no parser in the `coverage_cmd` (parser is required in this case)
    /// - the parser failed to parse the output produced by the `coverage_cmd`
    #[instrument(skip(self))]
    fn run(&self, repo_root: RepoRoot) -> Result<CoverageRunStatus, CoverageErr> {
        let repo_root = repo_root.to_string();
        debug!("running coverage in {repo_root}");
        let Ok(output) = self.cfg.coverage_cmd.stdout(repo_root) else {
            error!("command failed");
            return Ok(CoverageRunStatus::Failure);
        };

        let Some(parser) = self.cfg.coverage_cmd.parser() else {
            error!("this command requires output parser");
            return Ok(CoverageRunStatus::Failure);
        };

        let parser = parser.lock().expect("poisoned mutex");

        let Ok(coverage) = parser.parse(output) else {
            error!("failed to parse the output");
            return Ok(CoverageRunStatus::Failure);
        };

        Ok(CoverageRunStatus::Success(coverage))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::config::ConfigBuilder;
    use crate::configuration::tracing::init_tracing;
    use crate::data_providers::command::Cmd;
    use crate::testingtools::output_parser::{failing, working};

    use anyhow::Result;

    #[test]
    fn it_fails_with_invalid_cmd() -> Result<()> {
        // given
        init_tracing();
        let cfg = ConfigBuilder::default()
            .coverage_cmd(Cmd::new("no-such-command", &[]))
            .build()?;
        let cov_runner = DefaultCoverageRunner::make(cfg);
        let repo_root = RepoRoot::new("/tmp");

        // when
        let res = cov_runner.run(repo_root)?;

        // then
        assert_eq!(res, CoverageRunStatus::Failure);

        Ok(())
    }

    #[test]
    fn it_fails_when_coverage_cmd_does_not_have_parser() -> Result<()> {
        // given
        init_tracing();
        let cfg = ConfigBuilder::default()
            .coverage_cmd(Cmd::new("ls", &[]))
            .build()?;
        let cov_runner = DefaultCoverageRunner::make(cfg);
        let repo_root = RepoRoot::new("/tmp");

        // when
        let res = cov_runner.run(repo_root)?;

        // then
        assert_eq!(res, CoverageRunStatus::Failure);

        Ok(())
    }

    #[test]
    fn it_fails_when_parser_fails() -> Result<()> {
        // given
        init_tracing();
        let cfg = ConfigBuilder::default()
            .coverage_cmd(Cmd::with_parser("ls", &[], failing()))
            .build()?;
        let cov_runner = DefaultCoverageRunner::make(cfg);
        let repo_root = RepoRoot::new("/tmp");

        // when
        let res = cov_runner.run(repo_root)?;

        // then
        assert_eq!(res, CoverageRunStatus::Failure);

        Ok(())
    }

    #[test]
    fn it_returns_coverage_value_when_parser_succeeds() -> Result<()> {
        // given
        init_tracing();
        let cfg = ConfigBuilder::default()
            .coverage_cmd(Cmd::with_parser("ls", &[], working(60.0)))
            .build()?;
        let cov_runner = DefaultCoverageRunner::make(cfg);
        let repo_root = RepoRoot::new("/tmp");

        // when
        let res = cov_runner.run(repo_root)?;

        // then
        assert_eq!(res, CoverageRunStatus::Success(60.0));

        Ok(())
    }
}
