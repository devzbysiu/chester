use crate::configuration::config::ConfigBuilderError;

use actix_web::ResponseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WatcherErr {
    #[error("Error when using bus.")]
    Bus(#[from] BusErr),

    #[error("Failed to receive event from watcher.")]
    Receive(#[from] std::sync::mpsc::RecvError),

    #[error("Failed to watch for events.")]
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

    #[error("Error when writing to state.")]
    Write(#[from] StateWriterErr),
}

#[derive(Debug, Error)]
pub enum CheckErr {
    #[error("Error when using bus.")]
    Bus(#[from] BusErr),

    #[error("Error when reading state.")]
    Read(#[from] StateReaderErr),

    #[error("Error when writing to state.")]
    Write(#[from] StateWriterErr),
}

#[derive(Debug, Error)]
pub enum IndexErr {
    #[error("Error when using bus.")]
    Bus(#[from] BusErr),

    #[error("Error when reading state.")]
    Read(#[from] StateReaderErr),

    #[error("Error while executing command.")]
    Cmd(#[from] CmdErr),
}

#[derive(Debug, Error)]
pub enum CoverageErr {
    #[error("Error when using bus.")]
    Bus(#[from] BusErr),

    #[error("Error when creating regex.")]
    Regex(#[from] regex::Error),

    #[error("Error when reading state.")]
    Read(#[from] StateReaderErr),

    #[error("Error when writing to state.")]
    Write(#[from] StateWriterErr),
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
pub enum StateWriterErr {
    #[error("Failed to send event.")]
    Bus(#[from] BusErr),
}

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

#[derive(Debug, Error)]
pub enum IgnoredPathErr {
    #[error("Failed to create ignored path.")]
    Regex(#[from] regex::Error),
}

#[derive(Debug, Error)]
pub enum CmdErr {
    #[error("Error while executing cmd.")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum CfgErr {
    #[error("Error while executing cmd.")]
    Builder(#[from] ConfigBuilderError),

    #[error("Failed to configure ignored paths.")]
    IgnoredPath(#[from] IgnoredPathErr),
}

#[derive(Debug, Error, Clone, Default)]
pub enum CoverageParseErr {
    #[error("Failed to get last line of the output.")]
    #[default]
    NoLastLine,

    #[error("Received output does not contain code coverage info.")]
    InvalidOutput,

    #[error("Invalid coverage value.")]
    InvalidValue(String),
}

#[cfg(test)]
mod test {
    use super::*;

    use anyhow::anyhow;

    #[test]
    fn server_error_implements_response_error() {
        // given
        let err = ServerErr::Generic(anyhow!("some error"));

        // when
        let status = err.status_code();

        // then
        assert_eq!(status, 500);
    }
}
