use crate::entities::status::TestsStatus;
use crate::result::{RepoReaderErr, RepoWriterErr};

use std::sync::Arc;

pub type Repo = Box<dyn Repository>;
pub type RepoReader = Arc<dyn RepositoryReader>;
pub type RepoWriter = Arc<dyn RepositoryWriter>;

pub trait Repository: Send {
    fn reader(&self) -> RepoReader;
    fn writer(&self) -> RepoWriter;
}

pub trait RepositoryReader: Sync + Send {
    fn status(&self) -> Result<TestsStatus, RepoReaderErr>;
}

pub trait RepositoryWriter: Sync + Send {
    fn status(&self, status: TestsStatus) -> Result<(), RepoWriterErr>;
}
