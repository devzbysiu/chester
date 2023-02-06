use crate::entities::status::TestsStatus;
use crate::result::{RepoReadErr, RepoWriteErr};
use crate::testingtools::{pipe, MutexExt, Spy, Tx};
use crate::use_cases::repo::{
    Repo, RepoRead, RepoWrite, Repository, RepositoryRead, RepositoryWrite,
};

use anyhow::Result;
use std::sync::Arc;

pub fn tracked(repo: &Repo) -> (RepoSpies, Repo) {
    TrackedRepo::wrap(repo)
}

pub struct TrackedRepo {
    read: RepoRead,
    write: RepoWrite,
}

impl TrackedRepo {
    fn wrap(repo: &Repo) -> (RepoSpies, Repo) {
        let (read_status_tx, read_status_spy) = pipe();

        let (write_status_tx, write_status_spy) = pipe::<TestsStatus>();

        (
            RepoSpies::new(read_status_spy, write_status_spy),
            Box::new(Self {
                read: TrackedRepoRead::create(repo.read(), read_status_tx),
                write: TrackedRepoWrite::create(repo.write(), write_status_tx),
            }),
        )
    }
}

impl Repository for TrackedRepo {
    fn read(&self) -> RepoRead {
        self.read.clone()
    }

    fn write(&self) -> RepoWrite {
        self.write.clone()
    }
}

pub struct TrackedRepoRead {
    read: RepoRead,
    #[allow(unused)]
    read_status_tx: Tx,
}

impl TrackedRepoRead {
    fn create(read: RepoRead, read_status_tx: Tx) -> RepoRead {
        Arc::new(Self {
            read,
            read_status_tx,
        })
    }
}

impl RepositoryRead for TrackedRepoRead {
    fn status(&self) -> Result<TestsStatus, RepoReadErr> {
        self.read.status()
    }
}

pub struct TrackedRepoWrite {
    write: RepoWrite,
    write_status_tx: Tx<TestsStatus>,
}

impl TrackedRepoWrite {
    fn create(write: RepoWrite, write_status_tx: Tx<TestsStatus>) -> RepoWrite {
        Arc::new(Self {
            write,
            write_status_tx,
        })
    }
}

impl RepositoryWrite for TrackedRepoWrite {
    fn status(&self, status: TestsStatus) -> Result<(), RepoWriteErr> {
        let res = self.write.status(status.clone());
        self.write_status_tx.signal(status);
        res
    }
}

pub struct RepoSpies {
    #[allow(unused)]
    read_status_spy: Spy,
    write_status_spy: Spy<TestsStatus>,
}

impl RepoSpies {
    fn new(read_status_spy: Spy, write_status_spy: Spy<TestsStatus>) -> Self {
        Self {
            read_status_spy,
            write_status_spy,
        }
    }

    #[allow(unused)]
    pub fn read_called(&self) -> bool {
        self.read_status_spy.method_called()
    }

    pub fn write_called_with_val(&self, status: &TestsStatus) -> bool {
        self.write_status_spy.method_called_with_val(status)
    }
}

pub fn working() -> Repo {
    WorkingRepo::make()
}

struct WorkingRepo {
    read: RepoRead,
    write: RepoWrite,
}

impl WorkingRepo {
    fn make() -> Repo {
        Box::new(Self {
            read: WorkingRepoRead::new(),
            write: WorkingRepoWrite::new(),
        })
    }
}

impl Repository for WorkingRepo {
    fn read(&self) -> RepoRead {
        self.read.clone()
    }

    fn write(&self) -> RepoWrite {
        self.write.clone()
    }
}

struct WorkingRepoRead;

impl WorkingRepoRead {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl RepositoryRead for WorkingRepoRead {
    fn status(&self) -> Result<TestsStatus, RepoReadErr> {
        Ok(TestsStatus::Success)
    }
}

struct WorkingRepoWrite;

impl WorkingRepoWrite {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl RepositoryWrite for WorkingRepoWrite {
    fn status(&self, _status: TestsStatus) -> Result<(), RepoWriteErr> {
        Ok(())
    }
}
