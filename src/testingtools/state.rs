use crate::entities::check::CheckState;
use crate::entities::coverage::CoverageState;
use crate::entities::repo_root::RepoRoot;
use crate::entities::tests::TestsState;
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

        let (write_tests_status_tx, write_tests_status_spy) = pipe::<TestsState>();
        let (write_check_status_tx, write_check_status_spy) = pipe::<CheckState>();
        let (write_coverage_status_tx, write_coverage_status_spy) = pipe::<CoverageState>();

        let (write_repo_root_tx, write_repo_root_spy) = pipe::<RepoRoot>();

        (
            StateSpies::new(
                read_status_spy,
                write_tests_status_spy,
                write_check_status_spy,
                write_coverage_status_spy,
                write_repo_root_spy,
            ),
            Arc::new(Self {
                read: TrackedStateRead::create(state.reader(), read_status_tx),
                write: TrackedStateWrite::create(
                    state.writer(),
                    write_tests_status_tx,
                    write_check_status_tx,
                    write_coverage_status_tx,
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
    fn tests(&self) -> Result<TestsState, StateReaderErr> {
        self.read.tests()
    }

    fn check(&self) -> Result<CheckState, StateReaderErr> {
        self.read.check()
    }

    fn coverage(&self) -> Result<CoverageState, StateReaderErr> {
        self.read.coverage()
    }

    fn repo_root(&self) -> Result<RepoRoot, StateReaderErr> {
        self.read.repo_root()
    }
}

pub struct TrackedStateWrite {
    write: StateWriter,
    write_tests_state_tx: Tx<TestsState>,
    write_check_state_tx: Tx<CheckState>,
    write_coverage_state_tx: Tx<CoverageState>,
    write_repo_root_tx: Tx<RepoRoot>,
}

impl TrackedStateWrite {
    fn create(
        write: StateWriter,
        write_tests_state_tx: Tx<TestsState>,
        write_check_state_tx: Tx<CheckState>,
        write_coverage_state_tx: Tx<CoverageState>,
        write_repo_root_tx: Tx<RepoRoot>,
    ) -> StateWriter {
        Arc::new(Self {
            write,
            write_tests_state_tx,
            write_check_state_tx,
            write_coverage_state_tx,
            write_repo_root_tx,
        })
    }
}

impl AppStateWriter for TrackedStateWrite {
    fn tests(&self, status: TestsState) -> Result<(), StateWriterErr> {
        let res = self.write.tests(status.clone());
        self.write_tests_state_tx.signal(status);
        res
    }

    fn check(&self, status: CheckState) -> Result<(), StateWriterErr> {
        let res = self.write.check(status.clone());
        self.write_check_state_tx.signal(status);
        res
    }

    fn coverage(&self, coverage: CoverageState) -> Result<(), StateWriterErr> {
        let res = self.write.coverage(coverage.clone());
        self.write_coverage_state_tx.signal(coverage);
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
    write_tests_status_spy: Spy<TestsState>,
    #[allow(unused)]
    write_check_status_spy: Spy<CheckState>,
    write_coverage_status_spy: Spy<CoverageState>,
    write_repo_root_spy: Spy<RepoRoot>,
}

impl StateSpies {
    fn new(
        read_status_spy: Spy,
        write_tests_status_spy: Spy<TestsState>,
        write_check_status_spy: Spy<CheckState>,
        write_coverage_status_spy: Spy<CoverageState>,
        write_repo_root_spy: Spy<RepoRoot>,
    ) -> Self {
        Self {
            read_status_spy,
            write_tests_status_spy,
            write_check_status_spy,
            write_coverage_status_spy,
            write_repo_root_spy,
        }
    }

    #[allow(unused)]
    pub fn read_called(&self) -> bool {
        self.read_status_spy.method_called()
    }

    pub fn tests_state_called_with_val(&self, status: &TestsState) -> bool {
        self.write_tests_status_spy.method_called_with_val(status)
    }

    pub fn check_state_called_with_val(&self, status: &CheckState) -> bool {
        self.write_check_status_spy.method_called_with_val(status)
    }

    pub fn coverage_state_called_with_val(&self, status: &CoverageState) -> bool {
        self.write_coverage_status_spy
            .method_called_with_val(status)
    }

    #[allow(unused)]
    pub fn repo_root_called_with_val(&self, repo_root: &RepoRoot) -> bool {
        self.write_repo_root_spy.method_called_with_val(repo_root)
    }
}

pub fn noop() -> State {
    working()
}

pub fn working_with(values: StateValues) -> State {
    WorkingState::make(values)
}

pub fn working() -> State {
    WorkingState::make(StateValues::default())
}

pub struct StateValues {
    pub check_state: CheckState,
    pub tests_state: TestsState,
    pub cov_state: CoverageState,
    pub repo_root: RepoRoot,
}

impl Default for StateValues {
    fn default() -> Self {
        Self {
            check_state: CheckState::Success,
            tests_state: TestsState::Success,
            cov_state: CoverageState::Success(20.0),
            repo_root: RepoRoot::default(),
        }
    }
}

struct WorkingState {
    read: StateReader,
    write: StateWriter,
}

impl WorkingState {
    fn make(values: StateValues) -> State {
        Arc::new(Self {
            read: WorkingStateRead::new(values),
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

struct WorkingStateRead {
    values: StateValues,
}

impl WorkingStateRead {
    fn new(values: StateValues) -> Arc<Self> {
        Arc::new(Self { values })
    }
}

impl AppStateReader for WorkingStateRead {
    fn check(&self) -> Result<CheckState, StateReaderErr> {
        Ok(self.values.check_state.clone())
    }

    fn tests(&self) -> Result<TestsState, StateReaderErr> {
        Ok(self.values.tests_state.clone())
    }

    fn coverage(&self) -> Result<CoverageState, StateReaderErr> {
        Ok(self.values.cov_state.clone())
    }

    fn repo_root(&self) -> Result<RepoRoot, StateReaderErr> {
        Ok(self.values.repo_root.clone())
    }
}

struct WorkingStateWrite;

impl WorkingStateWrite {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl AppStateWriter for WorkingStateWrite {
    fn tests(&self, _status: TestsState) -> Result<(), StateWriterErr> {
        Ok(())
    }

    fn check(&self, _status: CheckState) -> Result<(), StateWriterErr> {
        Ok(())
    }

    fn coverage(&self, _coverage: CoverageState) -> Result<(), StateWriterErr> {
        Ok(())
    }

    fn repo_root(&self, _repo_root: RepoRoot) -> Result<(), StateWriterErr> {
        Ok(())
    }
}
