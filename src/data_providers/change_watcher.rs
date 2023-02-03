use crate::use_cases::change_watcher::{ChangeWatcher, Watcher};

pub struct DefaultChangeWatcher;

impl DefaultChangeWatcher {
    pub fn make() -> ChangeWatcher {
        Box::new(Self)
    }
}

impl Watcher for DefaultChangeWatcher {
    fn run(&self) {
        todo!()
    }
}
