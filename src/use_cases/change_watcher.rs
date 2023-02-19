use crate::entities::repo_root::RepoRoot;
use crate::result::WatcherErr;

pub type ChangeWatcher = Box<dyn Watcher>;

pub trait Watcher: Send {
    fn wait_for_change(&self, path: RepoRoot) -> Result<(), WatcherErr>;
}
