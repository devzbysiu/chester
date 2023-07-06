use crate::result::CoverageParseErr;
use crate::use_cases::output_parser::OutputParser;

use once_cell::sync::Lazy;
use regex::Regex;
use tracing::{error, instrument};

const COVERAGE: usize = 1;
static COVERAGE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d+.\d{2})% coverage").unwrap());

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

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::tracing::init_tracing;

    use anyhow::Result;

    #[test]
    fn when_output_does_not_contain_last_line_it_return_error() {
        init_tracing();
        // given
        let cov_parser = CoverageParser;
        let coverage_output = String::new();

        // when
        let res = cov_parser.parse(coverage_output);

        // then
        assert!(matches!(res, Err(CoverageParseErr::NoLastLine)));
    }

    #[test]
    fn when_output_does_not_contain_coverage_percentage_it_returns_error() {
        init_tracing();
        // given
        let cov_parser = CoverageParser;
        let coverage_output = "\nno coverage data".to_string();

        // when
        let res = cov_parser.parse(coverage_output);

        // then
        assert!(matches!(res, Err(CoverageParseErr::InvalidOutput)));
    }

    #[test]
    fn with_correct_percentage_output_it_returns_percentage_value() -> Result<()> {
        init_tracing();
        // given
        let cov_parser = CoverageParser;
        let coverage_output = "\n25.05% coverage".to_string();

        // when
        let res = cov_parser.parse(coverage_output)?;

        // then
        assert!((res - 25.05).abs() < f32::EPSILON);

        Ok(())
    }

    #[test]
    fn it_works_with_single_digit_percentage() -> Result<()> {
        init_tracing();
        // given
        let cov_parser = CoverageParser;
        let coverage_output = "\n5.01% coverage".to_string();

        // when
        let res = cov_parser.parse(coverage_output)?;

        // then
        assert!((res - 5.01).abs() < f32::EPSILON);

        Ok(())
    }

    #[test]
    fn it_works_when_decimal_digits_are_0() -> Result<()> {
        init_tracing();
        // given
        let cov_parser = CoverageParser;
        let coverage_output = "\n5.00% coverage".to_string();

        // when
        let res = cov_parser.parse(coverage_output)?;

        // then
        assert!((res - 5.0).abs() < f32::EPSILON);

        Ok(())
    }

    #[test]
    fn it_works_with_hundred_percents() -> Result<()> {
        init_tracing();
        // given
        let cov_parser = CoverageParser;
        let coverage_output = "\n100.00% coverage".to_string();

        // when
        let res = cov_parser.parse(coverage_output)?;

        // then
        assert!((res - 100.0).abs() < f32::EPSILON);

        Ok(())
    }

    #[test]
    fn it_fails_with_three_digits_after_decimal_point() {
        init_tracing();
        // given
        let cov_parser = CoverageParser;
        let coverage_output = "\n50.007% coverage".to_string();

        // when
        let res = cov_parser.parse(coverage_output);

        // then
        assert!(matches!(res, Err(CoverageParseErr::InvalidOutput)));
    }

    #[test]
    fn it_fails_with_more_then_hundred_percent() {
        init_tracing();
        // given
        let cov_parser = CoverageParser;
        let coverage_output = "\n100.01% coverage".to_string();

        // when
        let res = cov_parser.parse(coverage_output);

        // then
        assert!(matches!(res, Err(CoverageParseErr::InvalidValue(_))));
    }

    #[test]
    fn it_fails_with_less_than_zero_value() {
        init_tracing();
        // given
        let cov_parser = CoverageParser;
        let coverage_output = "\n-0.01% coverage".to_string();

        // when
        let res = cov_parser.parse(coverage_output);

        // then
        assert!(matches!(res, Err(CoverageParseErr::InvalidOutput)));
    }

    #[test]
    fn it_fails_when_there_is_no_decimal_point() {
        init_tracing();
        // given
        let cov_parser = CoverageParser;
        let coverage_output = "\n50% coverage".to_string();

        // when
        let res = cov_parser.parse(coverage_output);

        // then
        assert!(matches!(res, Err(CoverageParseErr::InvalidOutput)));
    }

    #[test]
    fn it_fails_when_there_is_no_coverage_word() {
        init_tracing();
        // given
        let cov_parser = CoverageParser;
        let coverage_output = "\n50.05% value".to_string();

        // when
        let res = cov_parser.parse(coverage_output);

        // then
        assert!(matches!(res, Err(CoverageParseErr::InvalidOutput)));
    }

    #[test]
    fn it_fails_when_there_is_single_decimal_point() {
        init_tracing();
        // given
        let cov_parser = CoverageParser;
        let coverage_output = "\n50.5% coverage".to_string();

        // when
        let res = cov_parser.parse(coverage_output);

        // then
        assert!(matches!(res, Err(CoverageParseErr::InvalidOutput)));
    }
}
