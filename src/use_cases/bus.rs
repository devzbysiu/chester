use crate::result::BusErr;

use std::fmt::Debug;
use std::sync::Arc;

pub type EventBus = Arc<dyn Bus>;
pub type EventSubscriber = Box<dyn Subscriber>;
pub type EventPublisher = Box<dyn Publisher>;

pub trait Bus: Send + Sync + Debug {
    fn publisher(&self) -> EventPublisher;

    fn subscriber(&self) -> EventSubscriber;
}

pub trait Publisher: Send {
    fn send(&mut self, event: BusEvent) -> Result<(), BusErr>;
}

pub trait Subscriber: Send {
    fn recv(&self) -> Result<BusEvent, BusErr>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BusEvent {}
