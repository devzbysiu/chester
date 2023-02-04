use crate::configuration::factories::event_bus;
use crate::use_cases::bus::{BusEvent, EventBus, EventPublisher, EventSubscriber};
use crate::use_cases::change_watcher::Change;

use anyhow::Result;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

pub fn create_test_shim() -> Result<TestShim> {
    let (tx, rx) = channel();
    let rx = Some(rx);
    let bus = event_bus()?;
    let sub = bus.subscriber();
    let publ = bus.publisher();
    Ok(TestShim {
        rx,
        tx,
        bus,
        sub,
        publ,
    })
}

pub struct TestShim {
    rx: Option<Receiver<Change>>,
    tx: Sender<Change>,
    bus: EventBus,
    sub: EventSubscriber,
    publ: EventPublisher,
}

impl TestShim {
    pub fn trigger_watcher(&self) -> Result<()> {
        self.tx.send(Change::Any)?;
        Ok(())
    }

    pub fn simulate_change(&self) -> Result<()> {
        self.publ.send(BusEvent::ChangeDetected)?;
        Ok(())
    }

    pub fn bus(&self) -> EventBus {
        self.bus.clone()
    }

    pub fn rx(&mut self) -> Receiver<Change> {
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
}
