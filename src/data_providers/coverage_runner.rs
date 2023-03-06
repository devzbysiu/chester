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
        let re = Regex::new(r"(\d{2}.\d{2})% coverage")?;
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
        let Some(coverage) = caps.get(COVERAGE) else {
            error!("capture not found");
            return Ok(CoverageRunStatus::Failure);
        };
        let coverage = coverage.as_str();
        let Ok(coverage) = coverage.parse::<f32>() else {
            error!("failed to parse: {coverage}");
            return Ok(CoverageRunStatus::Failure)
        };
        Ok(CoverageRunStatus::Success(coverage))
    }
}

// TODO: Add tests
