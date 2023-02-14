use crate::entities::repo_root::RepoRoot;
use crate::entities::status::TestsStatus;
use crate::result::{StateReaderErr, StateWriterErr};

use std::fmt::Debug;
use std::sync::Arc;

pub type State = Arc<dyn AppState>;
pub type StateReader = Arc<dyn AppStateReader>;
pub type StateWriter = Arc<dyn AppStateWriter>;

pub trait AppState: Sync + Send {
    fn reader(&self) -> StateReader;
    fn writer(&self) -> StateWriter;
}

pub trait AppStateReader: Sync + Send {
    fn status(&self) -> Result<TestsStatus, StateReaderErr>;
    fn repo_root(&self) -> Result<RepoRoot, StateReaderErr>;
}

impl Debug for dyn AppStateReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = self.status().unwrap_or_default();
        let repo_root = self.repo_root().unwrap_or_default();
        write!(f, "status: {status}, repo_root: {repo_root}")
    }
}

pub trait AppStateWriter: Sync + Send {
    fn status(&self, status: TestsStatus) -> Result<(), StateWriterErr>;
    fn repo_root(&self, repo_root: RepoRoot) -> Result<(), StateWriterErr>;
}
