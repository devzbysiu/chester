use crate::configuration::config::Config;
use crate::entities::repo_root::RepoRoot;
use crate::result::WatcherErr;
use crate::use_cases::change_watcher::{ChangeWatcher, Watcher};

use notify::{RecommendedWatcher, RecursiveMode::Recursive};
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, Debouncer};
use std::cell::RefCell;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use tracing::{debug, instrument, trace};

type Rx = Receiver<Result<Vec<DebouncedEvent>, Vec<notify::Error>>>;
type Dbcr = Debouncer<RecommendedWatcher>;

/// Specific implementation of [`Watcher`] trait.
///
/// It blocks waiting for the change to appear on the filesystem. Changes to files can be ignored
/// by setting [`Config::ignored_paths`] and passing such [`Config`] object to
/// [`FsChangeWatcher::make`] fn.
///
/// Initially, it watches for changes in directory pointed by `repo_root` passed as an argument to
/// [`FsChangeWatcher::make`] fn. See [`FsChangeWatcher::wait_for_change`] for details.
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
    fn reattach_watcher(&self, new_root: RepoRoot) -> Result<(), WatcherErr> {
        debug!("repo root changed to '{new_root:?}', recreating watcher");
        let (new_rx, new_watcher) = setup_watcher(&new_root)?;
        self.rx.replace(new_rx);
        self.watcher.replace(new_watcher);
        self.repo_root.replace(new_root);
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
    let (tx, rx) = channel();
    let mut debouncer = new_debouncer(Duration::from_millis(500), None, tx)?;
    debouncer.watcher().watch(path.as_ref(), Recursive)?;
    Ok((rx, debouncer))
}

impl Watcher for FsChangeWatcher {
    /// Each call to this fn blocks. You need to pass a [`RepoRoot`] to that function to inform
    /// it what directory to watch for changes.
    ///
    /// If the passed root is different than the one passed when creating
    /// `FsChangeWatcher`, then the filesystem watcher is reattached.
    ///
    /// Not every file change breaks the waiting loop. Some files can be ignored by setting
    /// [`Config::ignored_paths`] in a configuration passed as a second argument to
    /// [`FsChangeWatcher::make`].
    #[instrument(level = "trace", skip(self))]
    fn wait_for_change(&self, passed_root: RepoRoot) -> Result<(), WatcherErr> {
        if *self.repo_root.borrow() != passed_root {
            self.reattach_watcher(passed_root)?;
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

    use crate::configuration::config::ConfigBuilder;
    use crate::configuration::tracing::init_tracing;
    use crate::entities::ignored_path::IgnoredPath;
    use crate::testingtools::unit::{create_test_shim, mk_file, run_watcher};

    use anyhow::Result;
    use fake::{Fake, Faker};

    #[test]
    fn write_to_file_is_detected_as_change() -> Result<()> {
        // given
        init_tracing();
        let shim = create_test_shim()?;
        let watcher = FsChangeWatcher::make(shim.repo_root(), Config::default())?;

        // when
        let (_controller, detector) = run_watcher(watcher, shim.repo_root());
        mk_file(shim.repo_file(Faker.fake::<String>()))?;

        // then
        assert!(detector.change_detected());

        Ok(())
    }

    #[test]
    fn change_in_ignored_file_is_not_detected() -> Result<()> {
        // given
        init_tracing();
        let shim = create_test_shim()?;
        let cfg = ConfigBuilder::default()
            .ignored_paths(vec![IgnoredPath::new("target")?])
            .build()?;
        let watcher = FsChangeWatcher::make(shim.repo_root(), cfg)?;

        // when
        let (_controller, detector) = run_watcher(watcher, shim.repo_root());
        mk_file(shim.repo_file("target"))?;

        // then
        assert!(detector.no_change_detected());

        Ok(())
    }

    #[test]
    fn change_in_ignored_dir_is_not_detected() -> Result<()> {
        // given
        init_tracing();
        let shim = create_test_shim()?;
        let cfg = ConfigBuilder::default()
            .ignored_paths(vec![IgnoredPath::new(shim.dir_in_repo())?])
            .build()?;
        let watcher = FsChangeWatcher::make(shim.repo_root(), cfg)?;

        // when
        let (_controller, detector) = run_watcher(watcher, shim.repo_root());
        mk_file(shim.repo_file(shim.dir_in_repo().join("some-file")))?;

        // then
        assert!(detector.no_change_detected());

        Ok(())
    }

    // NOTE: There was a bug which exited the check loop too early, because
    // one of the ignored path didn't match the event path, so it was interpreted
    // as valid change, but in reality all ignored paths should be checked before
    // making the decision.
    #[test]
    fn multiple_ignored_paths_are_checked() -> Result<()> {
        // given
        init_tracing();
        let shim = create_test_shim()?;
        let cfg = ConfigBuilder::default()
            .ignored_paths(vec![IgnoredPath::new(".git")?, IgnoredPath::new("target")?])
            .build()?;
        let watcher = FsChangeWatcher::make(shim.repo_root(), cfg)?;

        // when
        let (_controller, detector) = run_watcher(watcher, shim.repo_root());
        mk_file(shim.repo_file("target"))?;

        // then
        assert!(detector.no_change_detected());

        Ok(())
    }

    #[test]
    fn regex_is_accepted_in_ignored_path() -> Result<()> {
        // given
        init_tracing();
        let shim = create_test_shim()?;
        let cfg = ConfigBuilder::default()
            .ignored_paths(vec![IgnoredPath::new(".*123.*456")?])
            .build()?;
        let watcher = FsChangeWatcher::make(shim.repo_root(), cfg)?;

        // when
        let (_controller, detector) = run_watcher(watcher, shim.repo_root());
        mk_file(shim.repo_file("123something456"))?;

        // then
        assert!(detector.no_change_detected());

        Ok(())
    }

    #[test]
    fn when_repo_root_is_changed_watcher_is_reattached() -> Result<()> {
        // given
        init_tracing();
        let shim = create_test_shim()?;
        let watcher = FsChangeWatcher::make(shim.repo_root(), Config::default())?;
        let (controller, detector) = run_watcher(watcher, shim.repo_root());
        mk_file(shim.repo_file(Faker.fake::<String>()))?;
        assert!(detector.change_detected());

        // when
        controller.change_repo(shim.new_repo_root())?;
        mk_file(shim.repo_file(Faker.fake::<String>()))?;
        assert!(detector.no_change_detected());
        mk_file(shim.new_repo_file(Faker.fake::<String>()))?;

        // then
        assert!(detector.change_detected());

        Ok(())
    }

    #[test]
    fn when_repo_root_is_set_to_the_same_value_watcher_is_not_reattached() -> Result<()> {
        // given
        init_tracing();
        let shim = create_test_shim()?;
        let watcher = FsChangeWatcher::make(shim.repo_root(), Config::default())?;
        let (controller, detector) = run_watcher(watcher, shim.repo_root());
        mk_file(shim.repo_file(Faker.fake::<String>()))?;
        assert!(detector.change_detected());

        // when
        controller.change_repo(shim.repo_root())?;
        mk_file(shim.repo_file(Faker.fake::<String>()))?;
        assert!(detector.change_detected());
        mk_file(shim.new_repo_file(Faker.fake::<String>()))?;

        // then
        assert!(detector.no_change_detected());

        Ok(())
    }
}
