use crate::configuration::config::Config;
use crate::data_providers::bus::LocalBus;
use crate::data_providers::change_watcher::FsChangeWatcher;
use crate::data_providers::check_runner::DefaultCheckRunner;
use crate::data_providers::coverage_runner::DefaultCoverageRunner;
use crate::data_providers::state::InMemoryState;
use crate::data_providers::test_runner::DefaultTestRunner;
use crate::data_providers::tests_index::DefaultTestsIndex;
use crate::entities::repo_root::RepoRoot;
use crate::result::{BusErr, SetupErr};
use crate::use_cases::bus::{EventBus, EventPublisher};
use crate::use_cases::change_watcher::ChangeWatcher;
use crate::use_cases::check_runner::CheckRunner;
use crate::use_cases::coverage_runner::CoverageRunner;
use crate::use_cases::state::{State, StateReader};
use crate::use_cases::test_runner::TestRunner;
use crate::use_cases::tests_index::TestsIndex;

use std::sync::Arc;

pub struct Runtime {
    pub bus: EventBus,
    pub change_watcher: ChangeWatcher,
    pub tests_index: TestsIndex,
    pub test_runner: TestRunner,
    pub check_runner: CheckRunner,
    pub coverage_runner: CoverageRunner,
    pub state: State,
}

impl Runtime {
    pub fn new(cfg: Config) -> Result<Self, SetupErr> {
        let bus = event_bus()?;
        let state = state(bus.publisher());
        Ok(Self {
            bus,
            change_watcher: change_watcher(state.reader().repo_root()?, cfg.clone())?,
            tests_index: tests_index(cfg.clone(), state.reader()),
            test_runner: test_runner(cfg.clone()),
            check_runner: check_runner(cfg.clone()),
            coverage_runner: coverage_runner(cfg),
            state,
        })
    }
}

pub fn event_bus() -> Result<EventBus, BusErr> {
    Ok(Arc::new(LocalBus::new()?))
}

fn change_watcher(repo_root: RepoRoot, cfg: Config) -> Result<ChangeWatcher, SetupErr> {
    Ok(FsChangeWatcher::make(repo_root, cfg)?)
}

fn tests_index(cfg: Config, sr: StateReader) -> TestsIndex {
    DefaultTestsIndex::make(cfg, sr)
}

fn test_runner(cfg: Config) -> TestRunner {
    DefaultTestRunner::make(cfg)
}

fn check_runner(cfg: Config) -> CheckRunner {
    DefaultCheckRunner::make(cfg)
}

fn coverage_runner(cfg: Config) -> CoverageRunner {
    DefaultCoverageRunner::make(cfg)
}

pub fn state(publ: EventPublisher) -> State {
    InMemoryState::make(publ)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn creating_runtime_works() {
        assert!(Runtime::new(Config::default()).is_ok());
    }
}
