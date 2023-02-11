use crate::entities::status::TestsStatus;
use crate::result::{StateReaderErr, StateWriterErr};
use crate::use_cases::state::{
    AppState, AppStateReader, AppStateWriter, State, StateReader, StateWriter,
};

use std::sync::{Arc, RwLock};

type StatusState = Arc<RwLock<TestsStatus>>;

pub struct InMemoryState {
    state_reader: StateReader,
    state_writer: StateWriter,
}

impl InMemoryState {
    pub fn make() -> State {
        let status = Arc::new(RwLock::new(TestsStatus::Pending));
        let state_reader = InMemoryStateRead::make(status.clone());
        let state_writer = InMemoryStateWrite::make(status);
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

pub struct InMemoryStateRead {
    status: StatusState,
}

impl InMemoryStateRead {
    fn make(status: StatusState) -> StateReader {
        Arc::new(Self { status })
    }
}

impl AppStateReader for InMemoryStateRead {
    fn status(&self) -> Result<TestsStatus, StateReaderErr> {
        let status = self.status.read().expect("poisoned mutex");
        Ok(status.clone())
    }
}

pub struct InMemoryStateWrite {
    status: StatusState,
}

impl InMemoryStateWrite {
    fn make(status: StatusState) -> StateWriter {
        Arc::new(Self { status })
    }
}

impl AppStateWriter for InMemoryStateWrite {
    fn status(&self, new_status: TestsStatus) -> Result<(), StateWriterErr> {
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
