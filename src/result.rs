use thiserror::Error;

#[derive(Debug, Error)]
pub enum WatcherErr {
    #[error("Error when using bus.")]
    Bus(#[from] BusErr),

    #[error("Failed to receive event from watcher.")]
    Receive(#[from] std::sync::mpsc::RecvError),
}

#[derive(Debug, Error)]
pub enum BusErr {
    #[error("Failed to create Eventador instance")]
    Generic(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum SetupErr {
    #[error("Failed to create event bus.")]
    Bus(#[from] BusErr),
}

#[derive(Debug, Error)]
pub enum RepoReadErr {}

#[derive(Debug, Error)]
pub enum RepoWriteErr {}
