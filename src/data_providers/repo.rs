use crate::entities::status::TestsStatus;
use crate::result::{RepoReadErr, RepoWriteErr};
use crate::use_cases::repo::{
    Repo, RepoRead, RepoWrite, Repository, RepositoryRead, RepositoryWrite,
};

use std::sync::{Arc, RwLock};

type RepoStatus = Arc<RwLock<TestsStatus>>;

pub struct InMemoryRepo {
    repo_read: RepoRead,
    repo_write: RepoWrite,
}

impl InMemoryRepo {
    pub fn make() -> Repo {
        let status = Arc::new(RwLock::new(TestsStatus::Pending));
        let repo_read = InMemoryRepoRead::make(status.clone());
        let repo_write = InMemoryRepoWrite::make(status);
        Box::new(Self {
            repo_read,
            repo_write,
        })
    }
}

impl Repository for InMemoryRepo {
    fn read(&self) -> RepoRead {
        self.repo_read.clone()
    }

    fn write(&self) -> RepoWrite {
        self.repo_write.clone()
    }
}

pub struct InMemoryRepoRead {
    status: RepoStatus,
}

impl InMemoryRepoRead {
    fn make(status: RepoStatus) -> RepoRead {
        Arc::new(Self { status })
    }
}

impl RepositoryRead for InMemoryRepoRead {
    fn status(&self) -> Result<TestsStatus, RepoReadErr> {
        let status = self.status.read().expect("poisoned mutex");
        Ok(status.clone())
    }
}

pub struct InMemoryRepoWrite {
    status: RepoStatus,
}

impl InMemoryRepoWrite {
    fn make(status: RepoStatus) -> RepoWrite {
        Arc::new(Self { status })
    }
}

impl RepositoryWrite for InMemoryRepoWrite {
    fn status(&self, new_status: TestsStatus) -> Result<(), RepoWriteErr> {
        let mut status = self.status.write().expect("poisoned mutex");
        *status = new_status;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::tracing::init_tracing;

    use anyhow::Result;

    #[test]
    fn what_is_written_to_repo_can_be_read() -> Result<()> {
        // given
        init_tracing();
        let repo = InMemoryRepo::make();
        let repo_read = repo.read();
        let repo_write = repo.write();
        assert_eq!(repo_read.status()?, TestsStatus::Pending);

        // when
        repo_write.status(TestsStatus::Success)?;

        // then
        assert_eq!(repo_read.status()?, TestsStatus::Success);

        Ok(())
    }
}
