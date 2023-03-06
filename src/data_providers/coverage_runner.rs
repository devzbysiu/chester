use crate::configuration::config::Config;
use crate::entities::repo_root::RepoRoot;
use crate::result::CoverageErr;
use crate::use_cases::coverage_runner::{CovRunner, CoverageRunStatus, CoverageRunner};

use cmd_lib::run_fun;
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
        let coverage_tool = &self.cfg.coverage_cmd.tool;
        let coverage_args = &self.cfg.coverage_cmd.args;
        let re = Regex::new(r"(\d{2}.\d{2})% coverage")?;
        // TODO: get rid of that `--skip-clean`
        if let Ok(output) = run_fun!(cd $repo_root ; $coverage_tool $coverage_args --skip-clean ) {
            if let Some(last_line) = output.lines().last() {
                if let Some(caps) = re.captures(last_line) {
                    if let Some(coverage) = caps.get(COVERAGE) {
                        let coverage = coverage.as_str();
                        if let Ok(coverage) = coverage.parse::<f32>() {
                            Ok(CoverageRunStatus::Success(coverage))
                        } else {
                            error!("failed to parse: {coverage}");
                            Ok(CoverageRunStatus::Failure)
                        }
                    } else {
                        error!("capture not found");
                        Ok(CoverageRunStatus::Failure)
                    }
                } else {
                    error!("no captures in {last_line}");
                    Ok(CoverageRunStatus::Failure)
                }
            } else {
                error!("no last line in command output");
                Ok(CoverageRunStatus::Failure)
            }
        } else {
            error!("command failed");
            Ok(CoverageRunStatus::Failure)
        }
    }
}

// TODO: Add tests
