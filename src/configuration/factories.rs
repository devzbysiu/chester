use crate::configuration::config::Config;
use crate::data_providers::bus::LocalBus;
use crate::data_providers::change_watcher::FsChangeWatcher;
use crate::data_providers::state::InMemoryState;
use crate::data_providers::test_runner::DefaultTestRunner;
use crate::entities::repo_root::RepoRoot;
use crate::result::{BusErr, SetupErr};
use crate::use_cases::bus::{EventBus, EventPublisher};
use crate::use_cases::change_watcher::ChangeWatcher;
use crate::use_cases::state::State;
use crate::use_cases::test_runner::TestRunner;

use std::sync::Arc;

pub struct Runtime {
    pub cfg: Config,
    pub bus: EventBus,
    pub change_watcher: ChangeWatcher,
    pub test_runner: TestRunner,
    pub state: State,
}

impl Runtime {
    pub fn new(cfg: Config) -> Result<Self, SetupErr> {
        let bus = event_bus()?;
        let state = state(bus.publisher());
        Ok(Self {
            cfg: cfg.clone(),
            bus,
            change_watcher: change_watcher(state.reader().repo_root()?, cfg)?,
            test_runner: test_runner(),
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

fn test_runner() -> TestRunner {
    DefaultTestRunner::make()
}

pub fn state(publ: EventPublisher) -> State {
    InMemoryState::make(publ)
}
