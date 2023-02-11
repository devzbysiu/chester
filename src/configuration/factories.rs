use crate::configuration::config::Config;
use crate::data_providers::bus::LocalBus;
use crate::data_providers::change_watcher::DefaultChangeWatcher;
use crate::data_providers::state::InMemoryState;
use crate::data_providers::test_runner::DefaultTestRunner;
use crate::result::{BusErr, SetupErr};
use crate::use_cases::bus::EventBus;
use crate::use_cases::change_watcher::ChangeWatcher;
use crate::use_cases::state::State;
use crate::use_cases::test_runner::TestRunner;

use std::sync::Arc;

#[allow(unused)]
pub struct Context {
    pub cfg: Config,
    pub bus: EventBus,
    pub change_watcher: ChangeWatcher,
    pub test_runner: TestRunner,
    pub state: State,
}

impl Context {
    #[allow(unused)]
    fn new(cfg: Config) -> Result<Self, SetupErr> {
        Ok(Self {
            cfg,
            bus: event_bus()?,
            change_watcher: change_watcher(),
            test_runner: test_runner(),
            state: state(),
        })
    }
}

pub fn event_bus() -> Result<EventBus, BusErr> {
    Ok(Arc::new(LocalBus::new()?))
}

fn change_watcher() -> ChangeWatcher {
    DefaultChangeWatcher::make()
}

fn test_runner() -> TestRunner {
    DefaultTestRunner::make()
}

pub fn state() -> State {
    InMemoryState::make()
}
