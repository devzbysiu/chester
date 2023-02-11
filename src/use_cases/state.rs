use crate::entities::status::TestsStatus;
use crate::result::{StateReaderErr, StateWriterErr};

use std::sync::Arc;

pub type State = Box<dyn AppState>;
pub type StateReader = Arc<dyn AppStateReader>;
pub type StateWriter = Arc<dyn AppStateWriter>;

pub trait AppState: Send {
    fn reader(&self) -> StateReader;
    fn writer(&self) -> StateWriter;
}

pub trait AppStateReader: Sync + Send {
    fn status(&self) -> Result<TestsStatus, StateReaderErr>;
}

pub trait AppStateWriter: Sync + Send {
    fn status(&self, status: TestsStatus) -> Result<(), StateWriterErr>;
}
