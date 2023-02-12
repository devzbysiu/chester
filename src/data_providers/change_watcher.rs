use crate::result::WatcherErr;
use crate::use_cases::change_watcher::Change;
use crate::use_cases::change_watcher::{ChangeWatcher, Watcher};

use anyhow::anyhow;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher as FileWatcher};
use std::path::Path;
use std::sync::mpsc::channel;

pub struct DefaultChangeWatcher;

impl DefaultChangeWatcher {
    pub fn make() -> ChangeWatcher {
        Box::new(Self)
    }
}

impl Watcher for DefaultChangeWatcher {
    fn next_change(&self) -> Result<Change, WatcherErr> {
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
        let path = Path::new("~/Projects/chester");
        watcher.watch(path, RecursiveMode::Recursive)?;
        if rx.recv().is_ok() {
            return Ok(Change::Any);
        }
        Err(WatcherErr::Generic(anyhow!("Failed to receive event")))
    }
}
