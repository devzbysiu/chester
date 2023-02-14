use actix_web::ResponseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WatcherErr {
    #[error("Error when using bus.")]
    Bus(#[from] BusErr),

    #[error("Failed to receive event from watcher.")]
    Receive(#[from] std::sync::mpsc::RecvError),

    #[error("Failed to watch for events: {0}")]
    FsWatcher(#[from] notify::Error),

    #[error("Failed to receive event")]
    Generic(#[from] anyhow::Error),

    #[error("Error when reading state.")]
    Read(#[from] StateReaderErr),
}

#[derive(Debug, Error)]
pub enum RunnerErr {
    #[error("Error when using bus.")]
    Bus(#[from] BusErr),

    #[error("Error when reading state.")]
    Read(#[from] StateReaderErr),
}

#[derive(Debug, Error)]
pub enum SinkErr {
    #[error("Error when using bus.")]
    Bus(#[from] BusErr),

    #[error("Failed to write to state.")]
    Write(#[from] StateWriterErr),
}

#[derive(Debug, Error)]
pub enum BusErr {
    #[error("Failed to create Eventador instance.")]
    Generic(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum SetupErr {
    #[error("Failed to create event bus.")]
    Bus(#[from] BusErr),

    #[error("IO operation failed.")]
    Io(#[from] std::io::Error),

    #[error("Failed to setup server.")]
    Hyper(#[from] hyper::Error),

    #[error("Failed to read from state.")]
    Read(#[from] StateReaderErr),

    #[error("Failed to create Watcher.")]
    Watch(#[from] WatcherErr),
}

#[derive(Debug, Error)]
pub enum StateReaderErr {}

#[derive(Debug, Error)]
pub enum StateWriterErr {}

#[derive(Debug, Error)]
pub enum ServerErr {
    #[error("Failed to serve results.")]
    Generic(#[from] anyhow::Error),
}

impl ResponseError for ServerErr {
    fn status_code(&self) -> hyper::StatusCode {
        hyper::StatusCode::from_u16(500).unwrap()
    }
}
