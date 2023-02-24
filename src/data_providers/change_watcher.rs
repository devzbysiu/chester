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

pub struct FsChangeWatcher {
    rx: RefCell<Rx>,
    watcher: RefCell<Dbcr>,
    repo_root: RefCell<RepoRoot>,
    cfg: Config,
}

impl FsChangeWatcher {
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
    fn is_ignored(&self, events: &[DebouncedEvent]) -> bool {
        if self.cfg.ignored_paths.is_empty() {
            return false;
        }
        let ignored_paths = &self.cfg.ignored_paths;
        for ev in events {
            let event_path = &ev.path;
            if ignored_paths.iter().any(|p| p.matched_by(event_path)) {
                trace!("ignored path: {event_path:?}");
                continue;
            }
            trace!("change detected: {event_path:?}");
            return false;
        }
        true
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

impl Watcher for FsChangeWatcher {
    #[instrument(level = "trace", skip(self))]
    fn wait_for_change(&self, current_root: RepoRoot) -> Result<(), WatcherErr> {
        if *self.repo_root.borrow() != current_root {
            self.update_watcher(current_root)?;
        }
        let rx = self.rx.borrow();
        loop {
            match rx.recv() {
                Ok(Ok(events)) if !self.is_ignored(&events) => return Ok(()),
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
    use crate::testingtools::unit::{create_test_shim, ChangeDetector};

    use anyhow::Result;
    use fake::{Fake, Faker};
    use std::thread::{self, JoinHandle};

    #[test]
    fn write_to_file_is_detected_as_change() -> Result<()> {
        // given
        init_tracing();
        let shim = create_test_shim()?;
        let watcher = FsChangeWatcher::make(shim.repo_root(), Config::default())?;

        // when
        let (_handle, detector) = run_watcher(shim.repo_root(), watcher);
        shim.mk_file(Faker.fake::<String>())?;

        // then
        assert!(detector.change_detected());

        Ok(())
    }

    fn run_watcher(
        repo_root: RepoRoot,
        watcher: ChangeWatcher,
    ) -> (JoinHandle<Result<()>>, ChangeDetector) {
        let (tx, rx) = channel();
        (
            thread::spawn(move || -> Result<()> {
                watcher.wait_for_change(repo_root)?;
                tx.send(())?;
                Ok(())
            }),
            ChangeDetector(rx),
        )
    }

    #[test]
    fn change_in_ignored_file_is_not_detected() -> Result<()> {
        // given
        init_tracing();
        let shim = create_test_shim()?;
        let ignored_paths = vec![IgnoredPath::new("target")?];
        let cfg = Config { ignored_paths };
        let watcher = FsChangeWatcher::make(shim.repo_root(), cfg)?;

        // when
        let (_handle, detector) = run_watcher(shim.repo_root(), watcher);
        shim.mk_file("target")?;

        // then
        assert!(detector.no_change_detected());

        Ok(())
    }

    #[test]
    fn change_in_ignored_dir_is_not_detected() -> Result<()> {
        // given
        init_tracing();
        let shim = create_test_shim()?;

        let ignored_paths = vec![IgnoredPath::new(shim.dir_in_repo())?];
        let cfg = Config { ignored_paths };
        let watcher = FsChangeWatcher::make(shim.repo_root(), cfg)?;

        // when
        let (_handle, detector) = run_watcher(shim.repo_root(), watcher);
        shim.mk_file(shim.dir_in_repo().join("some-file"))?;

        // then
        assert!(detector.no_change_detected());

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
        let shim = create_test_shim()?;
        let ignored_paths = vec![IgnoredPath::new(".git")?, IgnoredPath::new("target")?];
        let cfg = Config { ignored_paths };
        let watcher = FsChangeWatcher::make(shim.repo_root(), cfg)?;

        // when
        let (_handle, detector) = run_watcher(shim.repo_root(), watcher);
        shim.mk_file("target")?;

        // then
        assert!(detector.no_change_detected());

        Ok(())
    }

    #[test]
    fn regex_is_accepted_in_ignored_path() -> Result<()> {
        // given
        init_tracing();
        let shim = create_test_shim()?;
        let ignored_paths = vec![IgnoredPath::new(".*123.*456")?];
        let cfg = Config { ignored_paths };
        let watcher = FsChangeWatcher::make(shim.repo_root(), cfg)?;

        // when
        let (_handle, detector) = run_watcher(shim.repo_root(), watcher);
        shim.mk_file("123something456")?;

        // then
        assert!(detector.no_change_detected());

        Ok(())
    }
}
