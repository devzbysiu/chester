use crate::entities::repo_root::RepoRoot;
use crate::result::IndexErr;

pub type TestsIndex = Box<dyn TIndex>;

pub trait TIndex: Send {
    fn refresh(&self, repo_root: RepoRoot) -> Result<IndexStatus, IndexErr>;
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexStatus {
    TestsChanged,
    TestsNotChanged,
    Failure,
}
