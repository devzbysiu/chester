use crate::result::WatcherErr;
use crate::use_cases::bus::BusEvent;
use crate::use_cases::bus::EventBus;
use crate::use_cases::bus::EventPublisher;
use crate::use_cases::change_watcher::Change;
use crate::use_cases::change_watcher::ChangeWatcher;

use log::debug;
use std::thread;

type Result<T> = std::result::Result<T, WatcherErr>;

pub struct ChangeWatcherShell {
    #[allow(unused)]
    bus: EventBus,
}

impl ChangeWatcherShell {
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    #[allow(clippy::unused_self)]
    pub fn run(self, change_watcher: ChangeWatcher) {
        thread::spawn(move || -> Result<()> {
            loop {
                if let Ok(Change::Any) = change_watcher.next_change() {
                    trigger_tests(self.bus.publisher())?;
                } else {
                    debug!("no change detected");
                }
            }
        });
    }
}

pub fn trigger_tests(mut publ: EventPublisher) -> Result<()> {
    publ.send(BusEvent::ChangeDetected)?;
    Ok(())
}
