use crate::entities::status::Status;
use crate::result::{RepoReadErr, RepoWriteErr};
use crate::use_cases::repo::{
    Repo, RepoRead, RepoWrite, Repository, RepositoryRead, RepositoryWrite,
};

use std::sync::{Arc, RwLock};

type RepoStatus = Arc<RwLock<Status>>;

pub struct DefaultRepo {
    repo_read: RepoRead,
    repo_write: RepoWrite,
}

impl DefaultRepo {
    pub fn make() -> Repo {
        let status = Arc::new(RwLock::new(Status::Pending));
        let repo_read = DefaultRepoRead::make(status.clone());
        let repo_write = DefaultRepoWrite::make(status);
        Box::new(Self {
            repo_read,
            repo_write,
        })
    }
}

impl Repository for DefaultRepo {
    fn read(&self) -> RepoRead {
        self.repo_read.clone()
    }

    fn write(&self) -> RepoWrite {
        self.repo_write.clone()
    }
}

pub struct DefaultRepoRead {
    status: RepoStatus,
}

impl DefaultRepoRead {
    fn make(status: RepoStatus) -> RepoRead {
        Arc::new(Self { status })
    }
}

impl RepositoryRead for DefaultRepoRead {
    fn status(&self) -> Result<Status, RepoReadErr> {
        let status = self.status.read().expect("poisoned mutex");
        Ok(status.clone())
    }
}

pub struct DefaultRepoWrite {
    status: RepoStatus,
}

impl DefaultRepoWrite {
    fn make(status: RepoStatus) -> RepoWrite {
        Arc::new(Self { status })
    }
}

impl RepositoryWrite for DefaultRepoWrite {
    fn status(&self, new_status: Status) -> Result<(), RepoWriteErr> {
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
        let repo = DefaultRepo::make();
        let repo_read = repo.read();
        let repo_write = repo.write();
        assert_eq!(repo_read.status()?, Status::Pending);

        // when
        repo_write.status(Status::Success)?;

        // then
        assert_eq!(repo_read.status()?, Status::Success);

        Ok(())
    }
}
