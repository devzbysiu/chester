use crate::result::RunnerErr;

use std::path::PathBuf;

pub type TestRunner = Box<dyn Runner>;

pub trait Runner: Send {
    fn run_all(&self, repo_root: PathBuf) -> Result<TestsStatus, RunnerErr>;
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum TestsStatus {
    Success,
    Failure,
}
