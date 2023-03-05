use crate::configuration::config::Config;
use crate::entities::repo_root::RepoRoot;
use crate::result::RunnerErr;
use crate::use_cases::coverage_runner::{CovRunner, CoverageRunStatus, CoverageRunner};

use cmd_lib::run_cmd;
use tracing::{debug, instrument};

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
    fn run(&self, repo_root: RepoRoot) -> Result<CoverageRunStatus, RunnerErr> {
        let repo_root = repo_root.to_string();
        debug!("running coverage in {repo_root}");
        let coverage_tool = &self.cfg.tests_cmd.tool;
        let coverage_args = &self.cfg.tests_cmd.args;
        if let Err(e) = run_cmd!(cd $repo_root ; $coverage_tool $coverage_args) {
            debug!("coverage failed: {e}");
            Ok(CoverageRunStatus::Failure)
        } else {
            debug!("coverage succeeded");
            Ok(CoverageRunStatus::Success(19))
        }
    }
}

// TODO: Add tests
