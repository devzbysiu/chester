use crate::result::BusErr;

use std::fmt::Debug;
use std::sync::Arc;

pub type EventBus = Arc<dyn Bus>;
pub type EventSubscriber = Arc<dyn Subscriber>;
pub type EventPublisher = Arc<dyn Publisher>;

pub trait Bus: Send + Sync + Debug {
    fn publisher(&self) -> EventPublisher;

    fn subscriber(&self) -> EventSubscriber;
}

pub trait Publisher: Send {
    fn send(&self, event: BusEvent) -> Result<(), BusErr>;
}

pub trait Subscriber: Sync + Send {
    fn recv(&self) -> Result<BusEvent, BusErr>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BusEvent {
    ChangeDetected,
}
