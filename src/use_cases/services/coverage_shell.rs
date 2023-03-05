use crate::result::CoverageErr;
use crate::use_cases::bus::{BusEvent, EventBus};
use crate::use_cases::coverage_runner::{CoverageRunStatus, CoverageRunner};
use crate::use_cases::state::StateReader;

use std::thread;
use tracing::{debug, instrument, trace};

type Result<T> = std::result::Result<T, CoverageErr>;

pub struct CoverageShell {
    bus: EventBus,
}

impl CoverageShell {
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    #[instrument(skip(self, cov_runner))]
    pub fn run(self, cov_runner: CoverageRunner, st: StateReader) {
        let sub = self.bus.subscriber();
        let publ = self.bus.publisher();
        thread::spawn(move || -> Result<()> {
            loop {
                if let Ok(BusEvent::ChangeDetected) = sub.recv() {
                    debug!("running coverage");
                    if let Ok(CoverageRunStatus::Success(val)) = cov_runner.run(st.repo_root()?) {
                        debug!("coverage calculated");
                        publ.send(BusEvent::GotCoverage(val))?;
                    } else {
                        debug!("coverage failed");
                        publ.send(BusEvent::CoverageFailed)?;
                    }
                } else {
                    trace!("no change detected");
                }
            }
        });
    }
}

// TODO: Add tests
