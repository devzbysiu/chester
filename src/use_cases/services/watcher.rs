use crate::use_cases::{bus::EventBus, change_watcher::ChangeWatcher};

pub struct ChangeWatcherShell {
    #[allow(unused)]
    bus: EventBus,
}

impl ChangeWatcherShell {
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    #[allow(clippy::unused_self)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn run(&self, _change_watcher: ChangeWatcher) {
        todo!()
    }
}
