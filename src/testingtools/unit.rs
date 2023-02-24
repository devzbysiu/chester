use crate::configuration::factories::event_bus;
use crate::entities::repo_root::RepoRoot;
use crate::use_cases::bus::{BusEvent, EventBus, EventPublisher, EventSubscriber};

use anyhow::Result;
use fake::{Fake, Faker};
use std::fs::create_dir;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
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
    let next_dir = PathBuf::from(Faker.fake::<String>());
    create_dir(repo_dir.path().join(&next_dir))?;
    Ok(TestShim {
        rx,
        tx,
        bus,
        sub,
        publ,
        repo_dir,
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

    pub fn simulate_tests_succeeded(&self) -> Result<()> {
        self.publ.send(BusEvent::TestsPassed)?;
        Ok(())
    }

    pub fn simulate_tests_failed(&self) -> Result<()> {
        self.publ.send(BusEvent::TestsFailed)?;
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

    pub fn mk_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        fs::write(self.repo_dir.path().join(path), "some-content")?;
        Ok(())
    }

    pub fn dir_in_repo(&self) -> PathBuf {
        self.next_dir.clone()
    }
}

pub struct ChangeDetector(pub Receiver<()>);

impl ChangeDetector {
    pub fn change_detected(&self) -> bool {
        self.0.recv().is_ok()
    }

    pub fn no_change_detected(&self) -> bool {
        self.0.recv_timeout(Duration::from_millis(1000)).is_err()
    }
}
