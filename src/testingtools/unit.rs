use crate::configuration::factories::event_bus;
use crate::entities::repo_root::RepoRoot;
use crate::use_cases::bus::{BusEvent, EventBus, EventPublisher, EventSubscriber};
use crate::use_cases::change_watcher::ChangeWatcher;

use anyhow::Result;
use fake::{Fake, Faker};
use std::fs::create_dir;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{fs, thread};
use tempfile::{tempdir, TempDir};

pub fn create_test_shim() -> Result<TestShim> {
    let (tx, rx) = channel();
    let rx = Some(rx);
    let bus = event_bus()?;
    let sub = bus.subscriber();
    let publ = bus.publisher();
    let repo_dir = tempdir()?;
    let new_repo_dir = tempdir()?;
    let next_dir = PathBuf::from(Faker.fake::<String>());
    create_dir(repo_dir.path().join(&next_dir))?;
    Ok(TestShim {
        rx,
        tx,
        bus,
        sub,
        publ,
        repo_dir,
        new_repo_dir,
        next_dir,
    })
}

pub struct TestShim {
    rx: Option<Receiver<()>>,
    tx: Sender<()>,
    bus: EventBus,
    sub: EventSubscriber,
    publ: EventPublisher,
    repo_dir: TempDir,
    new_repo_dir: TempDir,
    next_dir: PathBuf,
}

impl TestShim {
    pub fn trigger_watcher(&self) -> Result<()> {
        self.tx.send(())?;
        Ok(())
    }

    pub fn simulate_change(&self) -> Result<()> {
        self.publ.send(BusEvent::ChangeDetected)?;
        Ok(())
    }

    pub fn simulate_check_passed(&self) -> Result<()> {
        self.publ.send(BusEvent::CheckPassed)?;
        Ok(())
    }

    pub fn simulate_tests_changed(&self) -> Result<()> {
        self.publ.send(BusEvent::TestsChanged)?;
        Ok(())
    }

    pub fn bus(&self) -> EventBus {
        self.bus.clone()
    }

    pub fn rx(&mut self) -> Receiver<()> {
        self.rx.take().unwrap()
    }

    pub fn event_on_bus(&self, event: &BusEvent) -> Result<bool> {
        let (tx, rx) = channel();
        let sub = self.sub.clone();
        let t = thread::spawn(move || -> Result<()> {
            tx.send(sub.recv()?)?;
            Ok(())
        });

        thread::sleep(Duration::from_millis(200));

        match rx.try_recv() {
            Ok(received) => Ok(*event == received),
            Err(TryRecvError::Empty) => {
                drop(rx);
                drop(t);
                // receiving event took more than 500 milliseconds
                Ok(false)
            }
            Err(TryRecvError::Disconnected) => unreachable!(),
        }
    }

    pub fn ignore_event(&self) -> Result<()> {
        let _event = self.sub.recv()?; // ignore message sent earliner
        Ok(())
    }

    pub fn repo_root(&self) -> RepoRoot {
        RepoRoot::new(&self.repo_dir)
    }

    pub fn new_repo_root(&self) -> RepoRoot {
        RepoRoot::new(&self.new_repo_dir)
    }

    pub fn repo_file<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.repo_dir.path().join(path)
    }

    pub fn new_repo_file<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.new_repo_dir.path().join(path)
    }

    pub fn dir_in_repo(&self) -> PathBuf {
        self.next_dir.clone()
    }
}

pub fn mk_file<P: AsRef<Path>>(path: P) -> Result<()> {
    fs::write(path, "some-content")?;
    Ok(())
}

pub struct ChangeDetector {
    change_rx: Receiver<()>,
}

impl ChangeDetector {
    pub fn new(change_rx: Receiver<()>) -> Self {
        Self { change_rx }
    }

    pub fn change_detected(&self) -> bool {
        self.change_rx.recv().is_ok()
    }

    pub fn no_change_detected(&self) -> bool {
        self.change_rx
            .recv_timeout(Duration::from_millis(1000))
            .is_err()
    }
}

pub fn run_watcher(watcher: ChangeWatcher, repo_root: RepoRoot) -> (Controller, ChangeDetector) {
    let (detector_tx, detector_rx) = channel();
    let (repo_root_tx, repo_root_rx) = channel();
    let handle = thread::spawn(move || -> Result<()> {
        loop {
            let repo_root = repo_root_rx.recv()?;
            watcher.wait_for_change(repo_root)?;
            detector_tx.send(())?;
        }
    });

    repo_root_tx.send(repo_root).unwrap();
    (
        Controller::new(handle, repo_root_tx),
        ChangeDetector::new(detector_rx),
    )
}

pub struct Controller {
    _handle: JoinHandle<Result<()>>,
    root_tx: Sender<RepoRoot>,
}

impl Controller {
    fn new(handle: JoinHandle<Result<()>>, root_tx: Sender<RepoRoot>) -> Self {
        Self {
            _handle: handle,
            root_tx,
        }
    }

    pub fn change_repo(&self, repo_root: RepoRoot) -> Result<()> {
        self.root_tx.send(repo_root)?;
        Ok(())
    }
}
