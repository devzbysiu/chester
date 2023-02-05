use crate::entities::status::Status;
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

        let (write_status_tx, write_status_spy) = pipe::<Status>();

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
    fn status(&self) -> Result<Status, RepoReadErr> {
        self.read.status()
    }
}

pub struct TrackedRepoWrite {
    write: RepoWrite,
    write_status_tx: Tx<Status>,
}

impl TrackedRepoWrite {
    fn create(write: RepoWrite, write_status_tx: Tx<Status>) -> RepoWrite {
        Arc::new(Self {
            write,
            write_status_tx,
        })
    }
}

impl RepositoryWrite for TrackedRepoWrite {
    fn status(&self, status: Status) -> Result<(), RepoWriteErr> {
        let res = self.write.status(status.clone());
        self.write_status_tx.signal(status);
        res
    }
}

pub struct RepoSpies {
    #[allow(unused)]
    read_status_spy: Spy,
    write_status_spy: Spy<Status>,
}

impl RepoSpies {
    fn new(read_status_spy: Spy, write_status_spy: Spy<Status>) -> Self {
        Self {
            read_status_spy,
            write_status_spy,
        }
    }

    #[allow(unused)]
    pub fn read_called(&self) -> bool {
        self.read_status_spy.method_called()
    }

    pub fn write_called_with_val(&self, status: &Status) -> bool {
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
    fn status(&self) -> Result<Status, RepoReadErr> {
        Ok(Status::Success)
    }
}

struct WorkingRepoWrite;

impl WorkingRepoWrite {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl RepositoryWrite for WorkingRepoWrite {
    fn status(&self, _status: Status) -> Result<(), RepoWriteErr> {
        Ok(())
    }
}

// pub fn failing() -> Repo {
//     FailingRepo::make()
// }

// struct FailingRepo {
//     read: RepoRead,
//     write: RepoWrite,
// }

// impl FailingRepo {
//     fn make() -> Repo {
//         Box::new(Self {
//             read: FailingRepoRead::new(),
//             write: FailingRepoWrite::new(),
//         })
//     }
// }

// impl Repository for FailingRepo {
//     fn read(&self) -> RepoRead {
//         self.read.clone()
//     }

//     fn write(&self) -> RepoWrite {
//         self.write.clone()
//     }
// }

// struct FailingRepoRead;

// impl FailingRepoRead {
//     fn new() -> Arc<Self> {
//         Arc::new(Self)
//     }
// }

// impl RepositoryRead for FailingRepoRead {
//     fn search(&self, _user: User, _q: String) -> Result<SearchResult, SearchErr> {
//         Err(SearchErr::MissingIndex("error".into()))
//     }

//     fn all_docs(&self, _user: User) -> Result<SearchResult, SearchErr> {
//         Err(SearchErr::MissingIndex("error".into()))
//     }
// }

// struct FailingRepoWrite;

// impl FailingRepoWrite {
//     fn new() -> Arc<Self> {
//         Arc::new(Self)
//     }
// }

// impl RepositoryWrite for FailingRepoWrite {
//     fn index(&self, _docs_details: &[DocDetails]) -> std::result::Result<(), IndexerErr> {
//         Err(IndexerErr::Bus(BusErr::Generic(anyhow!("error"))))
//     }

//     fn delete(&self, _loc: &Location) -> Result<(), IndexerErr> {
//         unimplemented!()
//     }
// }

// pub fn noop() -> Repo {
//     NoOpRepo::make()
// }

// struct NoOpRepo {
//     read: RepoRead,
//     write: RepoWrite,
// }

// impl NoOpRepo {
//     fn make() -> Repo {
//         Box::new(Self {
//             read: NoOpRepoRead::new(),
//             write: NoOpRepoWrite::new(),
//         })
//     }
// }

// impl Repository for NoOpRepo {
//     fn read(&self) -> RepoRead {
//         self.read.clone()
//     }

//     fn write(&self) -> RepoWrite {
//         self.write.clone()
//     }
// }

// struct NoOpRepoRead;

// impl NoOpRepoRead {
//     fn new() -> Arc<Self> {
//         Arc::new(Self)
//     }
// }

// impl RepositoryRead for NoOpRepoRead {
//     fn search(&self, _user: User, _q: String) -> Result<SearchResult, SearchErr> {
//         // nothing to do
//         Ok(Vec::new().into())
//     }

//     fn all_docs(&self, _user: User) -> Result<SearchResult, SearchErr> {
//         // nothing to do
//         Ok(Vec::new().into())
//     }
// }

// struct NoOpRepoWrite;

// impl NoOpRepoWrite {
//     fn new() -> Arc<Self> {
//         Arc::new(Self)
//     }
// }

// impl RepositoryWrite for NoOpRepoWrite {
//     fn index(&self, _docs_details: &[DocDetails]) -> std::result::Result<(), IndexerErr> {
//         // nothing to do here
//         Ok(())
//     }

//     fn delete(&self, _loc: &Location) -> Result<(), IndexerErr> {
//         // nothing to do here
//         Ok(())
//     }
// }
