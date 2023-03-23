use crate::entities::repo_root::RepoRoot;
use crate::result::{BusErr, IndexErr};
use crate::testingtools::{pipe, MutexExt, Spy, Tx};
use crate::use_cases::tests_index::IndexStatus;
use crate::use_cases::tests_index::{TIndex, TestsIndex};

use anyhow::anyhow;

pub fn tracked(index: TestsIndex) -> (TestsIndexSpy, TestsIndex) {
    TrackedTestsIndex::wrap(index)
}

pub struct TrackedTestsIndex {
    index: TestsIndex,
    tx: Tx,
}

impl TrackedTestsIndex {
    fn wrap(index: TestsIndex) -> (TestsIndexSpy, TestsIndex) {
        let (tx, spy) = pipe();

        (TestsIndexSpy::new(spy), Box::new(Self { index, tx }))
    }
}

impl TIndex for TrackedTestsIndex {
    fn refresh(&self, repo_root: RepoRoot) -> Result<IndexStatus, IndexErr> {
        let res = self.index.refresh(repo_root);
        self.tx.signal(());
        res
    }
}

pub struct TestsIndexSpy {
    spy: Spy,
}

impl TestsIndexSpy {
    fn new(spy: Spy) -> Self {
        Self { spy }
    }

    pub fn refresh_called(&self) -> bool {
        self.spy.method_called()
    }
}

pub fn working(result: IndexStatus) -> TestsIndex {
    WorkingTestsIndex::make(result)
}

pub struct WorkingTestsIndex {
    result: IndexStatus,
}

impl WorkingTestsIndex {
    fn make(result: IndexStatus) -> TestsIndex {
        Box::new(Self { result })
    }
}

impl TIndex for WorkingTestsIndex {
    fn refresh(&self, _repo_root: RepoRoot) -> Result<IndexStatus, IndexErr> {
        Ok(self.result.clone())
    }
}

pub fn failing() -> TestsIndex {
    FailingTestsIndex::make()
}

pub struct FailingTestsIndex;

impl FailingTestsIndex {
    fn make() -> TestsIndex {
        Box::new(Self)
    }
}

impl TIndex for FailingTestsIndex {
    fn refresh(&self, _repo_root: RepoRoot) -> Result<IndexStatus, IndexErr> {
        Err(IndexErr::Bus(BusErr::Generic(anyhow!("Failure"))))
    }
}
