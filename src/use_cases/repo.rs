use crate::entities::status::TestsStatus;
use crate::result::{RepoReadErr, RepoWriteErr};

use std::sync::Arc;

pub type Repo = Box<dyn Repository>;
pub type RepoRead = Arc<dyn RepositoryRead>;
pub type RepoWrite = Arc<dyn RepositoryWrite>;

pub trait Repository: Send {
    fn read(&self) -> RepoRead;
    fn write(&self) -> RepoWrite;
}

pub trait RepositoryRead: Sync + Send {
    fn status(&self) -> Result<TestsStatus, RepoReadErr>;
}

pub trait RepositoryWrite: Sync + Send {
    fn status(&self, status: TestsStatus) -> Result<(), RepoWriteErr>;
}
