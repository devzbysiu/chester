use crate::entities::repo_root::RepoRoot;
use crate::result::WatcherErr;

pub type ChangeWatcher = Box<dyn Watcher>;

pub trait Watcher: Send {
    fn next_change(&self, path: RepoRoot) -> Result<Change, WatcherErr>;
}

#[derive(Debug)]
pub enum Change {
    Any,
    No,
}
