use thiserror::Error;

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
