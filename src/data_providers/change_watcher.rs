use crate::entities::repo_root::RepoRoot;
use crate::result::WatcherErr;
use crate::use_cases::change_watcher::{Change, ChangeWatcher, Watcher};

use anyhow::anyhow;
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
}

fn setup_watcher<P: AsRef<Path>>(path: P) -> Result<(Rx, Dbcr), WatcherErr> {
    let path = path.as_ref();
    let (tx, rx) = channel();
    let mut debouncer = new_debouncer(Duration::from_secs(2), None, tx)?;
    debouncer.watcher().watch(path, RecursiveMode::Recursive)?;
    Ok((rx, debouncer))
}

impl Watcher for DefaultChangeWatcher {
    #[instrument(skip(self))]
    fn next_change(&self, path: RepoRoot) -> Result<Change, WatcherErr> {
        if self.repo_root != path {
            debug!("repo root changed, recreating watcher");
            let (new_rx, new_watcher) = setup_watcher(&path)?;
            let mut rx = self.rx.borrow_mut();
            let mut watcher = self.watcher.borrow_mut();
            *rx = new_rx;
            *watcher = new_watcher;
        }
        let rx = self.rx.borrow();
        let events: Result<Vec<DebouncedEvent>, Vec<notify::Error>> = rx.recv()?;
        let p = path.as_ref();
        match events {
            Ok(events) => {
                let mut valid_change = false;
                for ev in events {
                    let event_path = ev.path;
                    if event_path.starts_with(p.join("target")) {
                        trace!("ignored path: {event_path:?}");
                    } else {
                        debug!("valid path changed: {event_path:?}");
                        valid_change = true;
                        break;
                    }
                }
                if valid_change {
                    debug!("detected change");
                    Ok(Change::Any)
                } else {
                    Ok(Change::No)
                }
            }
            _ => Err(WatcherErr::Generic(anyhow!("Failed to receive event"))),
        }
    }
}
