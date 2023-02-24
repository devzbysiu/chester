use crate::configuration::config::Config;
use crate::entities::repo_root::RepoRoot;
use crate::result::WatcherErr;
use crate::use_cases::change_watcher::{ChangeWatcher, Watcher};

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
    repo_root: RefCell<RepoRoot>,
    cfg: Config,
}

impl DefaultChangeWatcher {
    pub fn make(repo_root: RepoRoot, cfg: Config) -> Result<ChangeWatcher, WatcherErr> {
        let (rx, watcher) = setup_watcher(&repo_root)?;
        Ok(Box::new(Self {
            rx: RefCell::new(rx),
            watcher: RefCell::new(watcher),
            repo_root: RefCell::new(repo_root),
            cfg,
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

    #[instrument(level = "trace", skip(self, events))]
    fn change_is_valid(&self, events: &[DebouncedEvent]) -> bool {
        if self.cfg.ignored_paths.is_empty() {
            return true;
        }
        let ignored_paths = &self.cfg.ignored_paths;
        for ev in events {
            let event_path = &ev.path;
            if ignored_paths.iter().any(|p| p.matched_by(event_path)) {
                trace!("ignored path: {event_path:?}");
                continue;
            }
            trace!("change detected: {event_path:?}");
            return true;
        }
        false
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
    fn wait_for_change(&self, current_root: RepoRoot) -> Result<(), WatcherErr> {
        if *self.repo_root.borrow() != current_root {
            self.update_watcher(current_root)?;
        }
        let rx = self.rx.borrow();
        loop {
            match rx.recv() {
                Ok(Ok(events)) if self.change_is_valid(&events) => return Ok(()),
                _ => trace!("no valid change detected"),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::tracing::init_tracing;
    use crate::entities::ignored_path::IgnoredPath;

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
        let watcher = DefaultChangeWatcher::make(repo_root.clone(), Config::default())?;

        // when
        let (tx, rx) = channel();
        thread::spawn(move || -> Result<()> {
            watcher.wait_for_change(repo_root)?;
            tx.send(())?;
            Ok(())
        });
        fs::write(tmpdir.path().join("test-file"), "some-content")?;

        // then
        assert!(rx.recv().is_ok());

        Ok(())
    }

    #[test]
    fn change_in_ignored_file_is_not_detected() -> Result<()> {
        // given
        init_tracing();
        let repo_dir = tempdir()?;
        let repo_root = RepoRoot::new(&repo_dir);
        let ignored_path = IgnoredPath::new("target")?;
        let ignored_paths = vec![ignored_path];
        let cfg = Config { ignored_paths };
        let watcher = DefaultChangeWatcher::make(repo_root.clone(), cfg)?;

        // when
        let (tx, rx) = channel();
        thread::spawn(move || -> Result<()> {
            watcher.wait_for_change(repo_root)?;
            tx.send(())?;
            Ok(())
        });
        fs::write(repo_dir.path().join("target"), "some-content")?;

        // then
        assert!(rx.recv_timeout(Duration::from_millis(500)).is_err());

        Ok(())
    }

    #[test]
    fn change_in_ignored_dir_is_not_detected() -> Result<()> {
        // given
        init_tracing();
        let repo_dir = tempdir()?;
        let ignored_path = IgnoredPath::new("target")?;
        let ignored_dir = repo_dir.path().join("target");
        create_dir(&ignored_dir)?;
        let repo_root = RepoRoot::new(&repo_dir);
        let ignored_paths = vec![ignored_path];
        let cfg = Config { ignored_paths };
        let watcher = DefaultChangeWatcher::make(repo_root.clone(), cfg)?;

        // when
        let (tx, rx) = channel();
        thread::spawn(move || -> Result<()> {
            watcher.wait_for_change(repo_root)?;
            tx.send(())?;
            Ok(())
        });
        fs::write(ignored_dir.join("some-file"), "some-content")?;

        // then
        assert!(rx.recv_timeout(Duration::from_millis(500)).is_err());

        Ok(())
    }

    // NOTE: There was a bug which exited the check loop too early, becouse
    // one of the ignored path didn't match the event path, so it was interpreted
    // as valid change, but in reality all ignored paths should be checked before
    // making the decision.
    #[test]
    fn multiple_ignored_paths_are_checked() -> Result<()> {
        // given
        init_tracing();
        let repo_dir = tempdir()?;
        let repo_root = RepoRoot::new(&repo_dir);
        let ignored_path = IgnoredPath::new("target")?;
        let ignored_paths = vec![IgnoredPath::new(".git")?, ignored_path];
        let cfg = Config { ignored_paths };
        let watcher = DefaultChangeWatcher::make(repo_root.clone(), cfg)?;

        // when
        let (tx, rx) = channel();
        thread::spawn(move || -> Result<()> {
            watcher.wait_for_change(repo_root)?;
            tx.send(())?;
            Ok(())
        });
        fs::write(repo_dir.path().join("target"), "some-content")?;

        // then
        assert!(rx.recv_timeout(Duration::from_millis(1000)).is_err());

        Ok(())
    }
}
