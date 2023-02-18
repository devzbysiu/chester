use crate::entities::repo_root::RepoRoot;
use crate::result::WatcherErr;
use crate::use_cases::change_watcher::{Change, ChangeWatcher, Watcher};

use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, Debouncer};
use std::cell::RefCell;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;
use tracing::{debug, instrument, trace};

type Rx = Receiver<Result<Vec<DebouncedEvent>, Vec<notify::Error>>>;
type Dbcr = Debouncer<RecommendedWatcher>;

pub struct DefaultChangeWatcher {
    rx: RefCell<Rx>,
    watcher: RefCell<Dbcr>,
    repo_root: RefCell<RepoRoot>,
}

impl DefaultChangeWatcher {
    pub fn make(repo_root: RepoRoot) -> Result<ChangeWatcher, WatcherErr> {
        let (rx, watcher) = setup_watcher(&repo_root)?;
        Ok(Box::new(Self {
            rx: RefCell::new(rx),
            watcher: RefCell::new(watcher),
            repo_root: RefCell::new(repo_root),
        }))
    }

    #[instrument(skip(self))]
    fn update_watcher(&self, current_root: RepoRoot) -> Result<(), WatcherErr> {
        debug!("repo root changed, recreating watcher");
        let (new_rx, new_watcher) = setup_watcher(&current_root)?;
        let mut rx = self.rx.borrow_mut();
        let mut watcher = self.watcher.borrow_mut();
        let mut repo_root = self.repo_root.borrow_mut();

        *rx = new_rx;
        *watcher = new_watcher;
        *repo_root = current_root;

        Ok(())
    }
}

#[instrument(skip(path))]
fn setup_watcher<P: AsRef<Path>>(path: P) -> Result<(Rx, Dbcr), WatcherErr> {
    let path = path.as_ref();
    let (tx, rx) = channel();
    let mut debouncer = new_debouncer(Duration::from_millis(500), None, tx)?;
    debouncer.watcher().watch(path, RecursiveMode::Recursive)?;
    Ok((rx, debouncer))
}

impl Watcher for DefaultChangeWatcher {
    #[instrument(level = "trace", skip(self))]
    fn next_change(&self, current_root: RepoRoot) -> Result<Change, WatcherErr> {
        if *self.repo_root.borrow() != current_root {
            self.update_watcher(current_root.clone())?;
        }
        let rx = self.rx.borrow();
        thread::sleep(Duration::from_secs(3));
        match rx.try_recv() {
            Ok(Ok(events)) if change_detected(&current_root, &events) => Ok(Change::Any),
            _ => {
                trace!("no valid change detected");
                Ok(Change::No)
            }
        }
    }
}

#[instrument(level = "trace", skip(events))]
fn change_detected(repo_root: &RepoRoot, events: &[DebouncedEvent]) -> bool {
    let mut valid_change = false;
    let repo_root = repo_root.as_ref();
    for ev in events {
        let event_path = &ev.path;
        if event_path.starts_with(repo_root.join("target")) {
            trace!("ignored path: {event_path:?}");
        } else {
            trace!("change detected: {event_path:?}");
            valid_change = true;
            break;
        }
    }
    trace!("changed: {}", if valid_change { "yes" } else { "no" });
    valid_change
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::tracing::init_tracing;

    use anyhow::Result;
    use std::fs::{self, create_dir};
    use std::thread;
    use tempfile::tempdir;

    #[test]
    fn write_to_file_is_detected_as_change() -> Result<()> {
        // given
        init_tracing();
        let tmpdir = tempdir()?;
        let repo_root = RepoRoot::new(&tmpdir);
        let watcher = DefaultChangeWatcher::make(repo_root.clone())?;

        // when
        let (tx, rx) = channel();
        thread::spawn(move || -> Result<()> {
            let change = watcher.next_change(repo_root)?;
            tx.send(change)?;
            Ok(())
        });
        fs::write(tmpdir.path().join("test-file"), "some-content")?;
        let change = rx.recv()?;

        // then
        assert_eq!(change, Change::Any);

        Ok(())
    }

    #[test]
    fn change_in_ignored_file_is_not_detected() -> Result<()> {
        // given
        init_tracing();
        let repo_dir = tempdir()?;
        let repo_root = RepoRoot::new(&repo_dir);
        let watcher = DefaultChangeWatcher::make(repo_root.clone())?;

        // when
        let (tx, rx) = channel();
        thread::spawn(move || -> Result<()> {
            let change = watcher.next_change(repo_root)?;
            tx.send(change)?;
            Ok(())
        });
        fs::write(repo_dir.path().join("target"), "some-content")?;
        let change = rx.recv()?;

        // then
        assert_eq!(change, Change::No);

        Ok(())
    }

    #[test]
    fn change_in_ignored_dir_is_not_detected() -> Result<()> {
        // given
        init_tracing();
        let repo_dir = tempdir()?;
        let ignored_dir = repo_dir.path().join("target");
        create_dir(&ignored_dir)?;
        let repo_root = RepoRoot::new(&repo_dir);
        let watcher = DefaultChangeWatcher::make(repo_root.clone())?;

        // when
        let (tx, rx) = channel();
        thread::spawn(move || -> Result<()> {
            let change = watcher.next_change(repo_root)?;
            tx.send(change)?;
            Ok(())
        });
        fs::write(ignored_dir.join("some-file"), "some-content")?;
        let change = rx.recv()?;

        // then
        assert_eq!(change, Change::No);

        Ok(())
    }
}
