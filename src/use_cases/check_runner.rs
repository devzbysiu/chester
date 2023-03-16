use crate::entities::repo_root::RepoRoot;
use crate::result::CheckErr;

pub type CheckRunner = Box<dyn CRunner>;

pub trait CRunner: Send {
    fn run(&self, repo_root: RepoRoot) -> Result<CheckRunStatus, CheckErr>;
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckRunStatus {
    Success,
    Failure,
}
