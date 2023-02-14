use crate::entities::repo_root::RepoRoot;
use crate::result::RunnerErr;

pub type TestRunner = Box<dyn Runner>;

pub trait Runner: Send {
    fn run_all(&self, repo_root: RepoRoot) -> Result<TestsStatus, RunnerErr>;
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestsStatus {
    Success,
    Failure,
}
