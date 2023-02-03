pub type ChangeWatcher = Box<dyn Watcher>;

pub trait Watcher {
    fn run(&self);
}
