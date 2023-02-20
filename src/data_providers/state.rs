use crate::entities::repo_root::RepoRoot;
use crate::entities::status::TestsStatus;
use crate::result::{StateReaderErr, StateWriterErr};
use crate::use_cases::bus::{BusEvent, EventPublisher};
use crate::use_cases::state::{
    AppState, AppStateReader, AppStateWriter, State, StateReader, StateWriter,
};

use std::sync::{Arc, RwLock};
use tracing::instrument;

type StatusState = Arc<RwLock<TestsStatus>>;
type RepoRootState = Arc<RwLock<RepoRoot>>;

pub struct InMemoryState {
    reader: StateReader,
    writer: StateWriter,
}

impl InMemoryState {
    pub fn make(publ: EventPublisher) -> State {
        let status = Arc::new(RwLock::new(TestsStatus::default()));
        let repo_root = Arc::new(RwLock::new(RepoRoot::default()));
        let reader = InMemoryStateRead::make(status.clone(), repo_root.clone());
        let writer = InMemoryStateWrite::make(status, repo_root, publ);
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
    status: StatusState,
    repo_root: RepoRootState,
}

impl InMemoryStateRead {
    fn make(status: StatusState, repo_root: RepoRootState) -> StateReader {
        Arc::new(Self { status, repo_root })
    }
}

impl AppStateReader for InMemoryStateRead {
    #[instrument(level = "trace")]
    fn status(&self) -> Result<TestsStatus, StateReaderErr> {
        let status = self.status.read().expect("poisoned mutex");
        Ok(status.clone())
    }

    #[instrument(level = "trace")]
    fn repo_root(&self) -> Result<RepoRoot, StateReaderErr> {
        let repo_root = self.repo_root.read().expect("poisoned mutex");
        Ok(repo_root.clone())
    }
}

pub struct InMemoryStateWrite {
    status: StatusState,
    repo_root: RepoRootState,
    publ: EventPublisher,
}

impl InMemoryStateWrite {
    fn make(status: StatusState, repo_root: RepoRootState, publ: EventPublisher) -> StateWriter {
        Arc::new(Self {
            status,
            repo_root,
            publ,
        })
    }
}

impl AppStateWriter for InMemoryStateWrite {
    #[instrument(level = "trace", skip(self))]
    fn status(&self, new_status: TestsStatus) -> Result<(), StateWriterErr> {
        let mut status = self.status.write().expect("poisoned mutex");
        *status = new_status;
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

    #[test]
    fn pending_status_is_set_as_default() -> Result<()> {
        // given
        init_tracing();
        let bus = event_bus()?;
        let state = InMemoryState::make(bus.publisher());
        let state = state.reader();

        // when
        let status = state.status()?;

        // then
        assert_eq!(status, TestsStatus::Pending);

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
        assert_eq!(state_reader.status()?, TestsStatus::Pending);

        // when
        state_writer.status(TestsStatus::Success)?;

        // then
        assert_eq!(state_reader.status()?, TestsStatus::Success);

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
        assert_eq!(state_reader.repo_root()?, RepoRoot::default());

        // when
        state_writer.repo_root(RepoRoot::new("/some/path"))?;

        // then
        assert_eq!(state_reader.repo_root()?, RepoRoot::new("/some/path"));

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
        state_writer.repo_root(RepoRoot::new("/some/path"))?;

        // then
        assert_eq!(sub.recv()?, BusEvent::ChangeDetected);

        Ok(())
    }
}
