use crate::entities::repo_root::RepoRoot;
use crate::result::CoverageErr;

pub type CoverageRunner = Box<dyn CovRunner>;

pub trait CovRunner: Send {
    fn run(&self, repo_root: RepoRoot) -> Result<CoverageRunStatus, CoverageErr>;
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub enum CoverageRunStatus {
    Success(f32),
    Failure,
}
