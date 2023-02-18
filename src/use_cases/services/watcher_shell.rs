use crate::result::WatcherErr;
use crate::use_cases::bus::BusEvent;
use crate::use_cases::bus::EventBus;
use crate::use_cases::bus::EventPublisher;
use crate::use_cases::change_watcher::Change;
use crate::use_cases::change_watcher::ChangeWatcher;
use crate::use_cases::state::StateReader;

use std::thread;
use tracing::{debug, instrument, trace};

type Result<T> = std::result::Result<T, WatcherErr>;

pub struct ChangeWatcherShell {
    bus: EventBus,
}

impl ChangeWatcherShell {
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    #[instrument(skip(self, change_watcher))]
    pub fn run(self, change_watcher: ChangeWatcher, state: StateReader) {
        thread::spawn(move || -> Result<()> {
            loop {
                let repo_root = state.repo_root()?;
                if let Ok(Change::Any) = change_watcher.next_change(repo_root) {
                    debug!("detected change, triggering tests");
                    trigger_tests(&self.bus.publisher())?;
                } else {
                    trace!("no change detected");
                }
            }
        });
    }
}

#[instrument(skip(publ))]
pub fn trigger_tests(publ: &EventPublisher) -> Result<()> {
    publ.send(BusEvent::ChangeDetected)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::factories::{event_bus, state};
    use crate::configuration::tracing::init_tracing;
    use crate::entities::repo_root::RepoRoot;
    use crate::testingtools::unit::create_test_shim;
    use crate::use_cases::change_watcher::Watcher;

    use anyhow::Result;
    use std::sync::mpsc::Receiver;

    #[test]
    fn any_change_in_watched_repo_triggers_tests() -> Result<()> {
        // given
        init_tracing();
        let mut shim = create_test_shim()?;
        let change_watcher = MockChangeWatcher::make(shim.rx());
        let bus = event_bus()?;
        let state = state(bus.publisher());
        ChangeWatcherShell::new(shim.bus()).run(change_watcher, state.reader());

        // when
        shim.trigger_watcher()?;

        // then
        assert!(shim.event_on_bus(&BusEvent::ChangeDetected)?);

        Ok(())
    }

    pub struct MockChangeWatcher {
        rx: Receiver<Change>,
    }

    impl MockChangeWatcher {
        fn make(rx: Receiver<Change>) -> ChangeWatcher {
            Box::new(Self { rx })
        }
    }

    impl Watcher for MockChangeWatcher {
        fn next_change(&self, _repo_root: RepoRoot) -> Result<Change, WatcherErr> {
            Ok(self.rx.recv()?)
        }
    }
}
