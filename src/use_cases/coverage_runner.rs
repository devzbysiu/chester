use crate::entities::repo_root::RepoRoot;
use crate::result::RunnerErr;

pub type CoverageRunner = Box<dyn CovRunner>;

pub trait CovRunner: Send {
    fn run(&self, repo_root: RepoRoot) -> Result<CoverageRunStatus, RunnerErr>;
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoverageRunStatus {
    Success(u8),
    Failure,
}
