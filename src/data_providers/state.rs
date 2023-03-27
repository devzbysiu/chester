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

#[derive(Debug, Default, Clone)]
struct Status<T: Clone> {
    value: Arc<RwLock<T>>,
}

impl<T: Clone> Status<T> {
    fn read(&self) -> T {
        self.value.read().expect("poisoned mutex").clone()
    }

    fn write(&self, new_val: T) {
        let mut old_val = self.value.write().expect("poisoned mutex");
        *old_val = new_val;
    }
}

#[derive(Debug, Default, Clone)]
struct StateValues {
    tests_state: Status<TestsState>,
    check_state: Status<CheckState>,
    coverage_state: Status<CoverageState>,
    repo_root: Status<RepoRoot>,
}

pub struct InMemoryState {
    reader: StateReader,
    writer: StateWriter,
}

impl InMemoryState {
    pub fn make(publ: EventPublisher) -> State {
        let state_values = StateValues::default();
        let reader = InMemoryStateReader::make(state_values.clone());
        let writer = InMemoryStateWriter::make(state_values, publ);
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
pub struct InMemoryStateReader {
    values: StateValues,
}

impl InMemoryStateReader {
    fn make(values: StateValues) -> StateReader {
        Arc::new(Self { values })
    }
}

impl AppStateReader for InMemoryStateReader {
    #[instrument(level = "trace")]
    fn tests(&self) -> Result<TestsState, StateReaderErr> {
        Ok(self.values.tests_state.read())
    }

    #[instrument(level = "trace")]
    fn check(&self) -> Result<CheckState, StateReaderErr> {
        Ok(self.values.check_state.read())
    }

    #[instrument(level = "trace")]
    fn coverage(&self) -> Result<CoverageState, StateReaderErr> {
        Ok(self.values.coverage_state.read())
    }

    #[instrument(level = "trace")]
    fn repo_root(&self) -> Result<RepoRoot, StateReaderErr> {
        Ok(self.values.repo_root.read())
    }
}

pub struct InMemoryStateWriter {
    values: StateValues,
    publ: EventPublisher,
}

impl InMemoryStateWriter {
    fn make(values: StateValues, publ: EventPublisher) -> StateWriter {
        Arc::new(Self { values, publ })
    }
}

impl AppStateWriter for InMemoryStateWriter {
    #[instrument(level = "trace", skip(self))]
    fn tests(&self, new_tests_state: TestsState) -> Result<(), StateWriterErr> {
        self.values.tests_state.write(new_tests_state);
        Ok(())
    }

    #[instrument(level = "trace", skip(self))]
    fn check(&self, new_check_state: CheckState) -> Result<(), StateWriterErr> {
        self.values.check_state.write(new_check_state);
        Ok(())
    }

    #[instrument(level = "trace", skip(self))]
    fn coverage(&self, new_coverage: CoverageState) -> Result<(), StateWriterErr> {
        self.values.coverage_state.write(new_coverage);
        Ok(())
    }

    #[instrument(level = "trace", skip(self))]
    fn repo_root(&self, new_repo_root: RepoRoot) -> Result<(), StateWriterErr> {
        self.values.repo_root.write(new_repo_root);
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
    fn pending_tests_status_is_set_as_default() -> Result<()> {
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
    fn pending_check_status_is_set_as_default() -> Result<()> {
        // given
        init_tracing();
        let bus = event_bus()?;
        let state = InMemoryState::make(bus.publisher());
        let state = state.reader();

        // when
        let status = state.check()?;

        // then
        assert_eq!(status, CheckState::Pending);

        Ok(())
    }

    #[test]
    fn pending_coverage_status_is_set_as_default() -> Result<()> {
        // given
        init_tracing();
        let bus = event_bus()?;
        let state = InMemoryState::make(bus.publisher());
        let state = state.reader();

        // when
        let status = state.coverage()?;

        // then
        assert_eq!(status, CoverageState::Pending);

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
    fn tests_status_written_to_state_can_be_read() -> Result<()> {
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
    fn check_status_written_to_state_can_be_read() -> Result<()> {
        // given
        init_tracing();
        let bus = event_bus()?;
        let state = InMemoryState::make(bus.publisher());
        let state_reader = state.reader();
        let state_writer = state.writer();
        assert_eq!(state_reader.check()?, CheckState::Pending);

        // when
        state_writer.check(CheckState::Success)?;

        // then
        assert_eq!(state_reader.check()?, CheckState::Success);

        Ok(())
    }

    #[test]
    fn coverage_status_written_to_state_can_be_read() -> Result<()> {
        // given
        init_tracing();
        let bus = event_bus()?;
        let state = InMemoryState::make(bus.publisher());
        let state_reader = state.reader();
        let state_writer = state.writer();
        assert_eq!(state_reader.coverage()?, CoverageState::Pending);

        // when
        state_writer.coverage(CoverageState::Success(90.0))?;

        // then
        assert_eq!(state_reader.coverage()?, CoverageState::Success(90.0));

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
}
