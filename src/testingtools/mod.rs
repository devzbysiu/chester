use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use std::time::Duration;
use tracing::error;

pub mod services;
pub mod unit;

pub fn pipe<T>() -> (Tx<T>, Spy<T>)
where
    T: Eq,
{
    let (tx, rx) = channel();
    (Mutex::new(tx), Spy::new(rx))
}

pub type Tx<T = ()> = Mutex<Sender<T>>;

pub struct Spy<T = ()>
where
    T: Eq,
{
    rx: Receiver<T>,
}

impl<T> Spy<T>
where
    T: Eq,
{
    pub fn new(rx: Receiver<T>) -> Self {
        Self { rx }
    }

    pub fn method_called(&self) -> bool {
        self.rx.recv_timeout(Duration::from_secs(30)).is_ok()
    }

    pub fn method_called_with_val(&self, val: &T) -> bool {
        match self.rx.recv_timeout(Duration::from_secs(30)) {
            Ok(res) => res == *val,
            _ => false,
        }
    }
}

pub trait MutexExt<T = ()> {
    fn signal(&self, val: T);
}

impl<T> MutexExt<T> for Tx<T> {
    fn signal(&self, val: T) {
        let tx = self.lock().expect("poisoned mutex");
        // NOTE: We can't `unwrap` or `expect` (etc.) here because during testing, the other end of
        // the channel gets dropped while this end is still used in thread. The result is that
        // `send` returns error and `unwrap` or `expect` panics which triigers abort and stop the
        // test binary.
        // This `signal` fn is used only for testing and it's acceptable to ignore this error.
        // Ultimately, if the other end is dropped it means that the test finished and all its
        // requirements are fullfilled.
        if let Err(e) = tx.send(val) {
            error!("failed to send signal: {:?}", e);
        }
    }
}
