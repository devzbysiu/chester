use crate::result::IndexErr;
use crate::use_cases::bus::{BusEvent, EventBus};
use crate::use_cases::state::State;
use crate::use_cases::tests_index::{IndexStatus, TestsIndex};

use std::thread;
use tracing::{debug, error, instrument, trace};

type Result<T> = std::result::Result<T, IndexErr>;

pub struct TestsIndexShell {
    bus: EventBus,
}

impl TestsIndexShell {
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    #[instrument(skip(self, index, st))]
    pub fn run(self, index: TestsIndex, st: State) {
        let sub = self.bus.subscriber();
        let publ = self.bus.publisher();
        thread::spawn(move || -> Result<()> {
            loop {
                if let Ok(BusEvent::TestsPassed) = sub.recv() {
                    debug!("checking if tests changed");
                    match index.refresh(st.reader().repo_root()?)? {
                        IndexStatus::TestsChanged => {
                            debug!("tests change detected");
                            publ.send(BusEvent::TestsChanged)?;
                        }
                        IndexStatus::TestsNotChanged => {
                            debug!("tests not changed");
                            publ.send(BusEvent::TestsNotChanged)?;
                        }
                        IndexStatus::Failure => error!("index refresh failed"),
                    }
                } else {
                    trace!("no change detected");
                }
            }
        });
    }
}

// TODO: Add Tests

// #[cfg(test)]
// mod test {
//     use super::*;

//     use crate::configuration::tracing::init_tracing;
//     use crate::testingtools::coverage_runner::{failing, tracked, working};
//     use crate::testingtools::state::noop;
//     use crate::testingtools::unit::create_test_shim;

//     use anyhow::Result;

//     #[test]
//     fn coverage_is_not_started_when_change_is_detected() -> Result<()> {
//         // given
//         init_tracing();
//         let (cov_runner_spy, cov_runner) = tracked(working(CoverageRunStatus::Success(20.0)));
//         let noop_state = noop();
//         let shim = create_test_shim()?;
//         CoverageShell::new(shim.bus()).run(cov_runner, noop_state.reader());

//         // when
//         shim.simulate_change()?;

//         // then
//         assert!(!cov_runner_spy.run_called());

//         Ok(())
//     }

//     #[test]
//     fn coverage_is_started_when_tests_passed() -> Result<()> {
//         // given
//         init_tracing();
//         let (cov_runner_spy, cov_runner) = tracked(working(CoverageRunStatus::Success(20.0)));
//         let noop_state = noop();
//         let shim = create_test_shim()?;
//         CoverageShell::new(shim.bus()).run(cov_runner, noop_state.reader());

//         // when
//         shim.simulate_tests_passed()?;

//         // then
//         assert!(cov_runner_spy.run_called());

//         Ok(())
//     }
//     #[test]
//     fn when_coverage_pass_there_is_corrent_event_on_the_bus() -> Result<()> {
//         // given
//         init_tracing();
//         let cov_runner = working(CoverageRunStatus::Success(20.0));
//         let noop_state = noop();
//         let shim = create_test_shim()?;
//         CoverageShell::new(shim.bus()).run(cov_runner, noop_state.reader());

//         // when
//         shim.simulate_tests_passed()?;
//         shim.ignore_event()?; // ignore BusEvent::ChangeDetected

//         // then
//         assert!(shim.event_on_bus(&BusEvent::GotCoverage(20.0))?);

//         Ok(())
//     }

//     #[test]
//     fn when_coverage_fail_there_is_correct_event_on_the_bus() -> Result<()> {
//         // given
//         init_tracing();
//         let coverage_runner = working(CoverageRunStatus::Failure);
//         let noop_state = noop();
//         let shim = create_test_shim()?;
//         CoverageShell::new(shim.bus()).run(coverage_runner, noop_state.reader());

//         // when
//         shim.simulate_tests_passed()?;
//         shim.ignore_event()?; // ignore BusEvent::ChangeDetected

//         // then
//         assert!(shim.event_on_bus(&BusEvent::CoverageFailed)?);

//         Ok(())
//     }

//     #[test]
//     fn when_coverage_runner_fail_there_is_correct_event_on_the_bus() -> Result<()> {
//         // given
//         init_tracing();
//         let coverage_runner = failing();
//         let noop_state = noop();
//         let shim = create_test_shim()?;
//         CoverageShell::new(shim.bus()).run(coverage_runner, noop_state.reader());

//         // when
//         shim.simulate_tests_passed()?;
//         shim.ignore_event()?; // ignore BusEvent::ChangeDetected

//         // then
//         assert!(shim.event_on_bus(&BusEvent::CoverageFailed)?);

//         Ok(())
//     }
// }
