use thiserror::Error;

#[derive(Debug, Error)]
pub enum WatcherErr {
    #[error("Error when using bus.")]
    Bus(#[from] BusErr),

    #[error("Failed to receive event from watcher.")]
    Receive(#[from] std::sync::mpsc::RecvError),
}

#[derive(Debug, Error)]
pub enum RunnerErr {
    #[error("Error when using bus.")]
    Bus(#[from] BusErr),
}

#[derive(Debug, Error)]
pub enum SinkErr {
    #[error("Error when using bus.")]
    Bus(#[from] BusErr),

    #[error("Failed to write to repo.")]
    Write(#[from] RepoWriteErr),
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
}

#[derive(Debug, Error)]
pub enum RepoReadErr {}

#[derive(Debug, Error)]
pub enum RepoWriteErr {}
