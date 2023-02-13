use tracing::instrument;

use crate::entities::repo_root::RepoRoot;
use crate::entities::status::TestsStatus;
use crate::result::{StateReaderErr, StateWriterErr};
use crate::use_cases::state::{
    AppState, AppStateReader, AppStateWriter, State, StateReader, StateWriter,
};

use std::sync::{Arc, RwLock};

type StatusState = Arc<RwLock<TestsStatus>>;
type RepoRootState = Arc<RwLock<RepoRoot>>;

pub struct InMemoryState {
    state_reader: StateReader,
    state_writer: StateWriter,
}

impl InMemoryState {
    pub fn make() -> State {
        let status = Arc::new(RwLock::new(TestsStatus::Pending));
        let repo_root = Arc::new(RwLock::new(RepoRoot::new("/tmp/testest")));
        let state_reader = InMemoryStateRead::make(status.clone(), repo_root.clone());
        let state_writer = InMemoryStateWrite::make(status, repo_root);
        Box::new(Self {
            state_reader,
            state_writer,
        })
    }
}

impl AppState for InMemoryState {
    fn reader(&self) -> StateReader {
        self.state_reader.clone()
    }

    fn writer(&self) -> StateWriter {
        self.state_writer.clone()
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
    #[instrument]
    fn status(&self) -> Result<TestsStatus, StateReaderErr> {
        let status = self.status.read().expect("poisoned mutex");
        Ok(status.clone())
    }

    #[instrument]
    fn repo_root(&self) -> Result<RepoRoot, StateReaderErr> {
        let repo_root = self.repo_root.read().expect("poisoned mutex");
        Ok(repo_root.clone())
    }
}

#[derive(Debug)]
pub struct InMemoryStateWrite {
    status: StatusState,
    repo_root: RepoRootState,
}

impl InMemoryStateWrite {
    fn make(status: StatusState, repo_root: RepoRootState) -> StateWriter {
        Arc::new(Self { status, repo_root })
    }
}

impl AppStateWriter for InMemoryStateWrite {
    #[instrument]
    fn status(&self, new_status: TestsStatus) -> Result<(), StateWriterErr> {
        let mut status = self.status.write().expect("poisoned mutex");
        *status = new_status;
        Ok(())
    }

    #[instrument]
    fn repo_root(&self, new_repo_root: RepoRoot) -> Result<(), StateWriterErr> {
        let mut repo_root = self.repo_root.write().expect("poisoned mutex");
        *repo_root = new_repo_root;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::tracing::init_tracing;

    use anyhow::Result;

    #[test]
    fn what_is_written_to_state_can_be_read() -> Result<()> {
        // given
        init_tracing();
        let state = InMemoryState::make();
        let state_reader = state.reader();
        let state_writer = state.writer();
        assert_eq!(state_reader.status()?, TestsStatus::Pending);

        // when
        state_writer.status(TestsStatus::Success)?;

        // then
        assert_eq!(state_reader.status()?, TestsStatus::Success);

        Ok(())
    }
}
