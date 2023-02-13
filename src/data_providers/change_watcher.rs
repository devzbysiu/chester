use crate::entities::repo_root::RepoRoot;
use crate::result::WatcherErr;
use crate::use_cases::change_watcher::{Change, ChangeWatcher, Watcher};

use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, Debouncer};
use std::cell::RefCell;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use tracing::{debug, instrument, trace};

type Rx = Receiver<Result<Vec<DebouncedEvent>, Vec<notify::Error>>>;
type Dbcr = Debouncer<RecommendedWatcher>;

pub struct DefaultChangeWatcher {
    rx: RefCell<Rx>,
    watcher: RefCell<Dbcr>,
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

    #[instrument(skip(self))]
    fn update_watcher(&self, current_root: &RepoRoot) -> Result<(), WatcherErr> {
        debug!("repo root changed, recreating watcher");
        let (new_rx, new_watcher) = setup_watcher(current_root)?;
        let mut rx = self.rx.borrow_mut();
        let mut watcher = self.watcher.borrow_mut();
        *rx = new_rx;
        *watcher = new_watcher;
        Ok(())
    }
}

#[instrument(skip(path))]
fn setup_watcher<P: AsRef<Path>>(path: P) -> Result<(Rx, Dbcr), WatcherErr> {
    let path = path.as_ref();
    let (tx, rx) = channel();
    let mut debouncer = new_debouncer(Duration::from_secs(2), None, tx)?;
    debouncer.watcher().watch(path, RecursiveMode::Recursive)?;
    Ok((rx, debouncer))
}

impl Watcher for DefaultChangeWatcher {
    #[instrument(skip(self))]
    fn next_change(&self, current_root: RepoRoot) -> Result<Change, WatcherErr> {
        if self.repo_root != current_root {
            self.update_watcher(&current_root)?;
        }
        let rx = self.rx.borrow();
        let events = rx.recv()?;
        match events {
            Ok(events) if change_detected(&current_root, &events) => Ok(Change::Any),
            _ => {
                debug!("no valid change detected");
                Ok(Change::No)
            }
        }
    }
}

#[instrument(skip(events))]
fn change_detected(repo_root: &RepoRoot, events: &[DebouncedEvent]) -> bool {
    let mut valid_change = false;
    let repo_root = repo_root.as_ref();
    for ev in events {
        let event_path = &ev.path;
        if event_path.starts_with(repo_root.join("target")) {
            trace!("ignored path: {event_path:?}");
        } else {
            debug!("change detected: {event_path:?}");
            valid_change = true;
            break;
        }
    }
    debug!("changed: {}", if valid_change { "yes" } else { "no" });
    valid_change
}
