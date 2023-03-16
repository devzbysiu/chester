use crate::entities::check::CheckState;
use crate::entities::coverage::CoverageState;
use crate::entities::repo_root::RepoRoot;
use crate::entities::tests::TestsState;
use crate::result::{StateReaderErr, StateWriterErr};
use crate::use_cases::bus::{BusEvent, EventPublisher};
use crate::use_cases::state::{
    AppState, AppStateReader, AppStateWriter, State, StateReader, StateWriter,
};

use std::sync::{Arc, RwLock};
use tracing::instrument;

type TestsStatusState = Arc<RwLock<TestsState>>;
type CheckStatusState = Arc<RwLock<CheckState>>;
type CoverageStatusState = Arc<RwLock<CoverageState>>;
type RepoRootState = Arc<RwLock<RepoRoot>>;

pub struct InMemoryState {
    reader: StateReader,
    writer: StateWriter,
}

impl InMemoryState {
    pub fn make(publ: EventPublisher) -> State {
        let tests_state = Arc::new(RwLock::new(TestsState::default()));
        let check_state = Arc::new(RwLock::new(CheckState::default()));
        let coverage_state = Arc::new(RwLock::new(CoverageState::default()));
        let repo_root = Arc::new(RwLock::new(RepoRoot::default()));
        let reader = InMemoryStateRead::make(
            tests_state.clone(),
            check_state.clone(),
            coverage_state.clone(),
            repo_root.clone(),
        );
        let writer =
            InMemoryStateWrite::make(tests_state, check_state, coverage_state, repo_root, publ);
        Arc::new(Self { reader, writer })
    }
}

impl AppState for InMemoryState {
    fn reader(&self) -> StateReader {
        self.reader.clone()
    }

    fn writer(&self) -> StateWriter {
        self.writer.clone()
    }
}

#[derive(Debug)]
pub struct InMemoryStateRead {
    tests_state: TestsStatusState,
    check_state: CheckStatusState,
    coverage_state: CoverageStatusState,
    repo_root: RepoRootState,
}

impl InMemoryStateRead {
    fn make(
        tests_state: TestsStatusState,
        check_state: CheckStatusState,
        coverage_state: CoverageStatusState,
        repo_root: RepoRootState,
    ) -> StateReader {
        Arc::new(Self {
            tests_state,
            check_state,
            coverage_state,
            repo_root,
        })
    }
}

impl AppStateReader for InMemoryStateRead {
    #[instrument(level = "trace")]
    fn tests(&self) -> Result<TestsState, StateReaderErr> {
        let tests_state = self.tests_state.read().expect("poisoned mutex");
        Ok(tests_state.clone())
    }

    #[instrument(level = "trace")]
    fn check(&self) -> Result<CheckState, StateReaderErr> {
        let check_state = self.check_state.read().expect("poisoned mutex");
        Ok(check_state.clone())
    }

    #[instrument(level = "trace")]
    fn coverage(&self) -> Result<CoverageState, StateReaderErr> {
        let coverage_state = self.coverage_state.read().expect("poisoned mutex");
        Ok(coverage_state.clone())
    }

    #[instrument(level = "trace")]
    fn repo_root(&self) -> Result<RepoRoot, StateReaderErr> {
        let repo_root = self.repo_root.read().expect("poisoned mutex");
        Ok(repo_root.clone())
    }
}

pub struct InMemoryStateWrite {
    tests_state: TestsStatusState,
    check_state: CheckStatusState,
    coverage_state: CoverageStatusState,
    repo_root: RepoRootState,
    publ: EventPublisher,
}

impl InMemoryStateWrite {
    fn make(
        tests_state: TestsStatusState,
        check_state: CheckStatusState,
        coverage_state: CoverageStatusState,
        repo_root: RepoRootState,
        publ: EventPublisher,
    ) -> StateWriter {
        Arc::new(Self {
            tests_state,
            check_state,
            coverage_state,
            repo_root,
            publ,
        })
    }
}

impl AppStateWriter for InMemoryStateWrite {
    #[instrument(level = "trace", skip(self))]
    fn tests(&self, new_tests_state: TestsState) -> Result<(), StateWriterErr> {
        let mut tests_state = self.tests_state.write().expect("poisoned mutex");
        *tests_state = new_tests_state;
        Ok(())
    }

    #[instrument(level = "trace", skip(self))]
    fn check(&self, new_check_state: CheckState) -> Result<(), StateWriterErr> {
        let mut check_state = self.check_state.write().expect("poisoned mutex");
        *check_state = new_check_state;
        Ok(())
    }

    #[instrument(level = "trace", skip(self))]
    fn coverage(&self, new_coverage: CoverageState) -> Result<(), StateWriterErr> {
        let mut coverage_state = self.coverage_state.write().expect("poisoned mutex");
        *coverage_state = new_coverage;
        Ok(())
    }

    #[instrument(level = "trace", skip(self))]
    fn repo_root(&self, new_repo_root: RepoRoot) -> Result<(), StateWriterErr> {
        let mut repo_root = self.repo_root.write().expect("poisoned mutex");
        *repo_root = new_repo_root;
        self.publ.send(BusEvent::ChangeDetected)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::factories::event_bus;
    use crate::configuration::tracing::init_tracing;

    use anyhow::Result;
    use fake::{Fake, Faker};

    #[test]
    fn pending_status_is_set_as_default() -> Result<()> {
        // given
        init_tracing();
        let bus = event_bus()?;
        let state = InMemoryState::make(bus.publisher());
        let state = state.reader();

        // when
        let status = state.tests()?;

        // then
        assert_eq!(status, TestsState::Pending);

        Ok(())
    }

    #[test]
    fn empty_root_is_set_as_default() -> Result<()> {
        // given
        init_tracing();
        let bus = event_bus()?;
        let state = InMemoryState::make(bus.publisher());
        let state = state.reader();

        // when
        let root = state.repo_root()?;

        // then
        assert_eq!(root, RepoRoot::new(""));

        Ok(())
    }

    #[test]
    fn status_written_to_state_can_be_read() -> Result<()> {
        // given
        init_tracing();
        let bus = event_bus()?;
        let state = InMemoryState::make(bus.publisher());
        let state_reader = state.reader();
        let state_writer = state.writer();
        assert_eq!(state_reader.tests()?, TestsState::Pending);

        // when
        state_writer.tests(TestsState::Success)?;

        // then
        assert_eq!(state_reader.tests()?, TestsState::Success);

        Ok(())
    }

    #[test]
    fn repo_root_written_to_state_can_be_read() -> Result<()> {
        // given
        init_tracing();
        let bus = event_bus()?;
        let state = InMemoryState::make(bus.publisher());
        let state_reader = state.reader();
        let state_writer = state.writer();
        let root = Faker.fake::<String>();
        assert_eq!(state_reader.repo_root()?, RepoRoot::default());

        // when
        state_writer.repo_root(RepoRoot::new(&root))?;

        // then
        assert_eq!(state_reader.repo_root()?, RepoRoot::new(root));

        Ok(())
    }

    #[test]
    fn change_in_repo_root_publishes_change_detected_message() -> Result<()> {
        // given
        init_tracing();
        let bus = event_bus()?;
        let sub = bus.subscriber();
        let state = InMemoryState::make(bus.publisher());
        let state_writer = state.writer();

        // when
        state_writer.repo_root(RepoRoot::new(Faker.fake::<String>()))?;

        // then
        assert_eq!(sub.recv()?, BusEvent::ChangeDetected);

        Ok(())
    }

    // TODO: Add tests for check status change
}
