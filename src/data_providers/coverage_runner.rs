use crate::configuration::config::Config;
use crate::entities::repo_root::RepoRoot;
use crate::result::CoverageErr;
use crate::use_cases::coverage_runner::{CovRunner, CoverageRunStatus, CoverageRunner};

use regex::Regex;
use tracing::{debug, error, instrument};

const COVERAGE: usize = 1;

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
    #[instrument(skip(self))]
    fn run(&self, repo_root: RepoRoot) -> Result<CoverageRunStatus, CoverageErr> {
        let repo_root = repo_root.to_string();
        debug!("running coverage in {repo_root}");
        let re = Regex::new(r"(\d+.\d{2})% coverage")?;
        let Ok(output) = self.cfg.coverage_cmd.stdout(repo_root) else {
            error!("command failed");
            return Ok(CoverageRunStatus::Failure);
        };
        let Some(last_line) = output.lines().last() else {
            error!("no last line in command output");
            return Ok(CoverageRunStatus::Failure);
        };
        let Some(caps) = re.captures(last_line) else {
            error!("no captures in {last_line}");
            return Ok(CoverageRunStatus::Failure);
        };
        // NOTE: the capture with idx `1` is always present, because we have only one group,
        // so if there is any match (checked above) then we are sure, that the idx `1` is present
        // (idx `0` is always whole match (all groups))
        let coverage = &caps[COVERAGE];
        // NOTE: if we captured two groups of digits divided by a dot, we can be sure that
        // it will parse to a `f32`
        let coverage = coverage.parse::<f32>().unwrap();
        if !(0.0..=100.0).contains(&coverage) {
            return Err(CoverageErr::InvalidValue(format!(
                "{coverage} value is invalid for code coverage"
            )));
        }
        Ok(CoverageRunStatus::Success(coverage))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::config::{Cmd, ConfigBuilder};
    use crate::configuration::tracing::init_tracing;

    use anyhow::Result;

    #[test]
    fn when_cmd_fails_it_returns_failure_status() -> Result<()> {
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
    fn when_there_is_no_output_it_returns_failure_status() -> Result<()> {
        init_tracing();
        // given
        let cfg = ConfigBuilder::default()
            .coverage_cmd(Cmd::new("true", &[]))
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
    fn when_output_does_not_contain_coverage_percentage_it_returns_failure_status() -> Result<()> {
        // given
        init_tracing();
        let cfg = ConfigBuilder::default()
            .coverage_cmd(Cmd::new("echo", &["some line"]))
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
    fn with_correct_percentage_output_it_returns_sucess_along_with_percentage() -> Result<()> {
        // given
        init_tracing();
        let cfg = ConfigBuilder::default()
            .coverage_cmd(Cmd::new("echo", &["10.11% coverage"]))
            .build()?;
        let cov_runner = DefaultCoverageRunner::make(cfg);
        let repo_root = RepoRoot::new("/tmp");

        // when
        let res = cov_runner.run(repo_root)?;

        // then
        assert_eq!(res, CoverageRunStatus::Success(10.11));

        Ok(())
    }

    #[test]
    fn it_works_with_single_digit_percentage() -> Result<()> {
        // given
        init_tracing();
        let cfg = ConfigBuilder::default()
            .coverage_cmd(Cmd::new("echo", &["1.11% coverage"]))
            .build()?;
        let cov_runner = DefaultCoverageRunner::make(cfg);
        let repo_root = RepoRoot::new("/tmp");

        // when
        let res = cov_runner.run(repo_root)?;

        // then
        assert_eq!(res, CoverageRunStatus::Success(1.11));

        Ok(())
    }

    #[test]
    fn it_works_when_decimal_digits_are_0() -> Result<()> {
        // given
        init_tracing();
        let cfg = ConfigBuilder::default()
            .coverage_cmd(Cmd::new("echo", &["1.00% coverage"]))
            .build()?;
        let cov_runner = DefaultCoverageRunner::make(cfg);
        let repo_root = RepoRoot::new("/tmp");

        // when
        let res = cov_runner.run(repo_root)?;

        // then
        assert_eq!(res, CoverageRunStatus::Success(1.00));

        Ok(())
    }

    #[test]
    fn it_works_with_hundred_percents() -> Result<()> {
        // given
        init_tracing();
        let cfg = ConfigBuilder::default()
            .coverage_cmd(Cmd::new("echo", &["100.00% coverage"]))
            .build()?;
        let cov_runner = DefaultCoverageRunner::make(cfg);
        let repo_root = RepoRoot::new("/tmp");

        // when
        let res = cov_runner.run(repo_root)?;

        // then
        assert_eq!(res, CoverageRunStatus::Success(100.00));

        Ok(())
    }

    #[test]
    fn it_fails_with_three_digits_after_decimal_point() -> Result<()> {
        // given
        init_tracing();
        let cfg = ConfigBuilder::default()
            .coverage_cmd(Cmd::new("echo", &["1.001% coverage"]))
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
    fn it_fails_with_more_then_hundred_percent() -> Result<()> {
        // given
        init_tracing();
        let cfg = ConfigBuilder::default()
            .coverage_cmd(Cmd::new("echo", &["101.01% coverage"]))
            .build()?;
        let cov_runner = DefaultCoverageRunner::make(cfg);
        let repo_root = RepoRoot::new("/tmp");

        // when
        let res = cov_runner.run(repo_root);

        // then
        assert!(matches!(res, Err(CoverageErr::InvalidValue(_))));

        Ok(())
    }
}
