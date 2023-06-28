use crate::result::WatcherErr;
use crate::use_cases::bus::{BusEvent, EventBus};
use crate::use_cases::change_watcher::ChangeWatcher;
use crate::use_cases::state::StateReader;

use std::thread;
use tracing::{debug, instrument};

type Result<T> = std::result::Result<T, WatcherErr>;

/// Watches for the change in the watched directory.
///
/// When change appears, it publishes `BusEvent::ChangeDetected`.
pub struct ChangeWatcherShell {
    bus: EventBus,
}

impl ChangeWatcherShell {
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    #[instrument(skip(self, change_watcher))]
    pub fn run(self, change_watcher: ChangeWatcher, state: StateReader) {
        let publ = self.bus.publisher();
        thread::spawn(move || -> Result<()> {
            loop {
                change_watcher.wait_for_change(state.repo_root()?)?;
                debug!("detected change");
                publ.send(BusEvent::ChangeDetected)?;
            }
        });
    }
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
        rx: Receiver<()>,
    }

    impl MockChangeWatcher {
        fn make(rx: Receiver<()>) -> ChangeWatcher {
            Box::new(Self { rx })
        }
    }

    impl Watcher for MockChangeWatcher {
        fn wait_for_change(&self, _repo_root: RepoRoot) -> Result<(), WatcherErr> {
            Ok(self.rx.recv()?)
        }
    }
}
