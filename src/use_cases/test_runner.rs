use crate::entities::repo_root::RepoRoot;
use crate::result::RunnerErr;

pub type TestRunner = Box<dyn TRunner>;

pub trait TRunner: Send {
    fn run_all(&self, repo_root: RepoRoot) -> Result<TestsRunStatus, RunnerErr>;
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestsRunStatus {
    Success,
    Failure,
}
