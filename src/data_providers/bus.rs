use crate::result::BusErr;
use crate::use_cases::bus::{
    Bus, BusEvent, EventPublisher, EventSubscriber, Publisher, Subscriber,
};

use std::sync::{Arc, Mutex};

const BUS_CAPACITY: u64 = 1024; // TODO: take care of this `capacity`

pub struct LocalBus {
    eventador: eventador::Eventador,
}

impl LocalBus {
    pub fn new() -> Result<Self, BusErr> {
        Ok(Self {
            eventador: eventador::Eventador::new(BUS_CAPACITY)?,
        })
    }
}

impl Bus for LocalBus {
    fn subscriber(&self) -> EventSubscriber {
        Arc::new(LocalSubscriber::new(self.eventador.subscribe()))
    }

    fn publisher(&self) -> EventPublisher {
        Arc::new(LocalPublisher::new(self.eventador.publisher()))
    }
}

pub struct LocalSubscriber {
    sub: eventador::Subscriber<BusEvent>,
}

impl LocalSubscriber {
    fn new(sub: eventador::Subscriber<BusEvent>) -> Self {
        Self { sub }
    }
}

impl Subscriber for LocalSubscriber {
    fn recv(&self) -> Result<BusEvent, BusErr> {
        Ok(self.sub.recv().to_owned())
    }
}

pub struct LocalPublisher {
    publ: Arc<Mutex<eventador::Publisher>>,
}

impl LocalPublisher {
    fn new(publ: eventador::Publisher) -> Self {
        let publ = Arc::new(Mutex::new(publ));
        Self { publ }
    }
}

impl Publisher for LocalPublisher {
    fn send(&self, event: BusEvent) -> Result<(), BusErr> {
        let mut publ = self.publ.lock().expect("poisoned mutex");
        publ.send(event);
        Ok(())
    }
}
