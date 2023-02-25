use crate::entities::repo_root::RepoRoot;
use crate::entities::status::TestsStatus;
use crate::result::{StateReaderErr, StateWriterErr};
use crate::testingtools::{pipe, MutexExt, Spy, Tx};
use crate::use_cases::state::{
    AppState, AppStateReader, AppStateWriter, State, StateReader, StateWriter,
};

use anyhow::Result;
use std::sync::Arc;

pub fn tracked(state: &State) -> (StateSpies, State) {
    TrackedState::wrap(state)
}

pub struct TrackedState {
    read: StateReader,
    write: StateWriter,
}

impl TrackedState {
    fn wrap(state: &State) -> (StateSpies, State) {
        let (read_status_tx, read_status_spy) = pipe();

        let (write_status_tx, write_status_spy) = pipe::<TestsStatus>();

        let (write_repo_root_tx, write_repo_root_spy) = pipe::<RepoRoot>();

        (
            StateSpies::new(read_status_spy, write_status_spy, write_repo_root_spy),
            Arc::new(Self {
                read: TrackedStateRead::create(state.reader(), read_status_tx),
                write: TrackedStateWrite::create(
                    state.writer(),
                    write_status_tx,
                    write_repo_root_tx,
                ),
            }),
        )
    }
}

impl AppState for TrackedState {
    fn reader(&self) -> StateReader {
        self.read.clone()
    }

    fn writer(&self) -> StateWriter {
        self.write.clone()
    }
}

pub struct TrackedStateRead {
    read: StateReader,
    #[allow(unused)]
    read_status_tx: Tx,
}

impl TrackedStateRead {
    fn create(read: StateReader, read_status_tx: Tx) -> StateReader {
        Arc::new(Self {
            read,
            read_status_tx,
        })
    }
}

impl AppStateReader for TrackedStateRead {
    fn status(&self) -> Result<TestsStatus, StateReaderErr> {
        self.read.status()
    }

    fn repo_root(&self) -> Result<RepoRoot, StateReaderErr> {
        self.read.repo_root()
    }
}

pub struct TrackedStateWrite {
    write: StateWriter,
    write_status_tx: Tx<TestsStatus>,
    write_repo_root_tx: Tx<RepoRoot>,
}

impl TrackedStateWrite {
    fn create(
        write: StateWriter,
        write_status_tx: Tx<TestsStatus>,
        write_repo_root_tx: Tx<RepoRoot>,
    ) -> StateWriter {
        Arc::new(Self {
            write,
            write_status_tx,
            write_repo_root_tx,
        })
    }
}

impl AppStateWriter for TrackedStateWrite {
    fn status(&self, status: TestsStatus) -> Result<(), StateWriterErr> {
        let res = self.write.status(status.clone());
        self.write_status_tx.signal(status);
        res
    }

    fn repo_root(&self, repo_root: RepoRoot) -> Result<(), StateWriterErr> {
        let res = self.write.repo_root(repo_root.clone());
        self.write_repo_root_tx.signal(repo_root);
        res
    }
}

pub struct StateSpies {
    #[allow(unused)]
    read_status_spy: Spy,
    write_status_spy: Spy<TestsStatus>,
    write_repo_root_spy: Spy<RepoRoot>,
}

impl StateSpies {
    fn new(
        read_status_spy: Spy,
        write_status_spy: Spy<TestsStatus>,
        write_repo_root_spy: Spy<RepoRoot>,
    ) -> Self {
        Self {
            read_status_spy,
            write_status_spy,
            write_repo_root_spy,
        }
    }

    #[allow(unused)]
    pub fn read_called(&self) -> bool {
        self.read_status_spy.method_called()
    }

    pub fn status_called_with_val(&self, status: &TestsStatus) -> bool {
        self.write_status_spy.method_called_with_val(status)
    }

    #[allow(unused)]
    pub fn repo_root_called_with_val(&self, repo_root: &RepoRoot) -> bool {
        self.write_repo_root_spy.method_called_with_val(repo_root)
    }
}

pub fn noop() -> State {
    working()
}

pub fn working() -> State {
    WorkingState::make()
}

struct WorkingState {
    read: StateReader,
    write: StateWriter,
}

impl WorkingState {
    fn make() -> State {
        Arc::new(Self {
            read: WorkingStateRead::new(),
            write: WorkingStateWrite::new(),
        })
    }
}

impl AppState for WorkingState {
    fn reader(&self) -> StateReader {
        self.read.clone()
    }

    fn writer(&self) -> StateWriter {
        self.write.clone()
    }
}

struct WorkingStateRead;

impl WorkingStateRead {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl AppStateReader for WorkingStateRead {
    fn status(&self) -> Result<TestsStatus, StateReaderErr> {
        Ok(TestsStatus::Success)
    }

    fn repo_root(&self) -> Result<RepoRoot, StateReaderErr> {
        Ok(RepoRoot::default())
    }
}

struct WorkingStateWrite;

impl WorkingStateWrite {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl AppStateWriter for WorkingStateWrite {
    fn status(&self, _status: TestsStatus) -> Result<(), StateWriterErr> {
        Ok(())
    }

    fn repo_root(&self, _repo_root: RepoRoot) -> Result<(), StateWriterErr> {
        Ok(())
    }
}
