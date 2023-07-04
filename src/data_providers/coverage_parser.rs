use crate::result::CoverageParseErr;
use crate::use_cases::output_parser::OutputParser;

use once_cell::sync::Lazy;
use regex::Regex;
use tracing::{error, instrument};

const COVERAGE: usize = 1;
static COVERAGE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+.\d{2})% coverage").unwrap());

pub struct CoverageParser;

impl OutputParser for CoverageParser {
    type Output = f32;
    type Error = CoverageParseErr;

    #[instrument(skip(self))]
    fn parse(&self, output: String) -> Result<Self::Output, Self::Error> {
        let Some(last_line) = output.lines().last() else {
            error!("no last line in command output");
            return Err(CoverageParseErr::NoLastLine);
        };

        let Some(captures) = COVERAGE_RE.captures(last_line) else {
            error!("no captures in '{last_line}'");
            return Err(CoverageParseErr::InvalidOutput);
        };

        // NOTE: the capture with idx `1` is always present, because we have only one group,
        // so if there is any match (checked above) then we are sure, that the idx `1` is present
        // (idx `0` is always whole match (all groups))
        let coverage = &captures[COVERAGE];
        // NOTE: if we captured two groups of digits divided by a dot, we can be sure that
        // it will parse to a `f32`
        let coverage = coverage.parse::<f32>().unwrap();
        if !(0.0..=100.0).contains(&coverage) {
            return Err(CoverageParseErr::InvalidValue(format!(
                "{coverage} value is invalid for code coverage"
            )));
        }

        Ok(coverage)
    }
}

// TODO: Update tests
#[cfg(test)]
mod test {
    use crate::configuration::config::ConfigBuilder;
    use crate::configuration::tracing::init_tracing;
    use crate::data_providers::command::Cmd;

    use anyhow::Result;

    #[test]
    #[ignore]
    fn when_there_is_no_output_it_returns_failure_status() -> Result<()> {
        init_tracing();
        // given
        let _cfg = ConfigBuilder::default()
            .coverage_cmd(Cmd::new("true", &[]))
            .build()?;
        // let cov_runner = DefaultCoverageRunner::make(cfg);
        // let repo_root = RepoRoot::new("/tmp");

        // // when
        // let res = cov_runner.run(repo_root)?;

        // // then
        // assert_eq!(res, CoverageRunStatus::Failure);

        Ok(())
    }

    #[test]
    #[ignore]
    fn when_output_does_not_contain_coverage_percentage_it_returns_failure_status() -> Result<()> {
        // given
        init_tracing();
        let _cfg = ConfigBuilder::default()
            .coverage_cmd(Cmd::new("echo", &["some line"]))
            .build()?;
        // let cov_runner = DefaultCoverageRunner::make(cfg);
        // let repo_root = RepoRoot::new("/tmp");

        // // when
        // let res = cov_runner.run(repo_root)?;

        // // then
        // assert_eq!(res, CoverageRunStatus::Failure);

        Ok(())
    }

    #[test]
    #[ignore]
    fn with_correct_percentage_output_it_returns_sucess_along_with_percentage() -> Result<()> {
        // given
        init_tracing();
        let _cfg = ConfigBuilder::default()
            .coverage_cmd(Cmd::new("echo", &["10.11% coverage"]))
            .build()?;
        // let cov_runner = DefaultCoverageRunner::make(cfg);
        // let repo_root = RepoRoot::new("/tmp");

        // // when
        // let res = cov_runner.run(repo_root)?;

        // // then
        // assert_eq!(res, CoverageRunStatus::Success(10.11));

        Ok(())
    }

    #[test]
    #[ignore]
    fn it_works_with_single_digit_percentage() -> Result<()> {
        // given
        init_tracing();
        let _cfg = ConfigBuilder::default()
            .coverage_cmd(Cmd::new("echo", &["1.11% coverage"]))
            .build()?;
        // let cov_runner = DefaultCoverageRunner::make(cfg);
        // let repo_root = RepoRoot::new("/tmp");

        // // when
        // let res = cov_runner.run(repo_root)?;

        // // then
        // assert_eq!(res, CoverageRunStatus::Success(1.11));

        Ok(())
    }

    #[test]
    #[ignore]
    fn it_works_when_decimal_digits_are_0() -> Result<()> {
        // given
        init_tracing();
        let _cfg = ConfigBuilder::default()
            .coverage_cmd(Cmd::new("echo", &["1.00% coverage"]))
            .build()?;
        // let cov_runner = DefaultCoverageRunner::make(cfg);
        // let repo_root = RepoRoot::new("/tmp");

        // // when
        // let res = cov_runner.run(repo_root)?;

        // // then
        // assert_eq!(res, CoverageRunStatus::Success(1.00));

        Ok(())
    }

    #[test]
    #[ignore]
    fn it_works_with_hundred_percents() -> Result<()> {
        // given
        init_tracing();
        let _cfg = ConfigBuilder::default()
            .coverage_cmd(Cmd::new("echo", &["100.00% coverage"]))
            .build()?;
        // let cov_runner = DefaultCoverageRunner::make(cfg);
        // let repo_root = RepoRoot::new("/tmp");

        // // when
        // let res = cov_runner.run(repo_root)?;

        // // then
        // assert_eq!(res, CoverageRunStatus::Success(100.00));

        Ok(())
    }

    #[test]
    #[ignore]
    fn it_fails_with_three_digits_after_decimal_point() -> Result<()> {
        // given
        init_tracing();
        let _cfg = ConfigBuilder::default()
            .coverage_cmd(Cmd::new("echo", &["1.001% coverage"]))
            .build()?;
        // let cov_runner = DefaultCoverageRunner::make(cfg);
        // let repo_root = RepoRoot::new("/tmp");

        // // when
        // let res = cov_runner.run(repo_root)?;

        // // then
        // assert_eq!(res, CoverageRunStatus::Failure);

        Ok(())
    }

    #[test]
    #[ignore]
    fn it_fails_with_more_then_hundred_percent() -> Result<()> {
        // given
        init_tracing();
        let _cfg = ConfigBuilder::default()
            .coverage_cmd(Cmd::new("echo", &["101.01% coverage"]))
            .build()?;
        // let cov_runner = DefaultCoverageRunner::make(cfg);
        // let repo_root = RepoRoot::new("/tmp");

        // // when
        // let res = cov_runner.run(repo_root);

        // // then
        // assert!(matches!(res, Err(CoverageErr::InvalidValue(_))));

        Ok(())
    }
}
