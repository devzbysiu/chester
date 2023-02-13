use crate::entities::repo_root::RepoRoot;
use crate::result::WatcherErr;
use crate::use_cases::change_watcher::{Change, ChangeWatcher, Watcher};

use anyhow::anyhow;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher as FileWatcher};
use std::cell::RefCell;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};

type Rx = Receiver<Result<Event, notify::Error>>;

pub struct DefaultChangeWatcher {
    rx: RefCell<Rx>,
    watcher: RefCell<RecommendedWatcher>,
    repo_root: RepoRoot,
}

impl DefaultChangeWatcher {
    pub fn make(repo_root: RepoRoot) -> Result<ChangeWatcher, WatcherErr> {
        let (rx, watcher) = setup_watcher(&repo_root)?;
        Ok(Box::new(Self {
            rx: RefCell::new(rx),
            watcher: RefCell::new(watcher),
            repo_root,
        }))
    }
}

fn setup_watcher<P: AsRef<Path>>(path: P) -> Result<(Rx, RecommendedWatcher), WatcherErr> {
    let path = path.as_ref();
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(path, RecursiveMode::Recursive)?;
    Ok((rx, watcher))
}

impl Watcher for DefaultChangeWatcher {
    fn next_change(&self, path: RepoRoot) -> Result<Change, WatcherErr> {
        if self.repo_root != path {
            let (new_rx, new_watcher) = setup_watcher(&path)?;
            let mut rx = self.rx.borrow_mut();
            let mut watcher = self.watcher.borrow_mut();
            *rx = new_rx;
            *watcher = new_watcher;
        }
        let rx = self.rx.borrow();
        if rx.recv().is_ok() {
            return Ok(Change::Any);
        }
        Err(WatcherErr::Generic(anyhow!("Failed to receive event")))
    }
}
