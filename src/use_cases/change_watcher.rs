use crate::result::WatcherErr;

pub type ChangeWatcher = Box<dyn Watcher>;

pub trait Watcher: Send {
    fn next_change(&self) -> Result<Change, WatcherErr>;
}

#[derive(Debug)]
pub enum Change {
    #[allow(unused)]
    Any,
}
