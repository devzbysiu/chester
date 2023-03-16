use crate::result::CheckErr;
use crate::use_cases::bus::{BusEvent, EventBus};
use crate::use_cases::check_runner::{CheckRunStatus, CheckRunner};
use crate::use_cases::state::StateReader;

use std::thread;
use tracing::{debug, instrument, trace};

type Result<T> = std::result::Result<T, CheckErr>;

pub struct CheckShell {
    bus: EventBus,
}

impl CheckShell {
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    #[instrument(skip(self, check_runner))]
    pub fn run(self, check_runner: CheckRunner, state: StateReader) {
        let sub = self.bus.subscriber();
        let publ = self.bus.publisher();
        thread::spawn(move || -> Result<()> {
            loop {
                if let Ok(BusEvent::ChangeDetected) = sub.recv() {
                    debug!("running check");
                    if let Ok(CheckRunStatus::Success) = check_runner.run(state.repo_root()?) {
                        debug!("check passed");
                        publ.send(BusEvent::CheckPassed)?;
                    } else {
                        debug!("check failed");
                        publ.send(BusEvent::CheckFailed)?;
                    }
                } else {
                    trace!("no change detected");
                }
            }
        });
    }
}

// TODO: Add tests

// #[cfg(test)]
// mod test {
//     use super::*;

//     use crate::configuration::tracing::init_tracing;
//     use crate::testingtools::state::noop;
//     use crate::testingtools::test_runner::{failing, tracked, working};
//     use crate::testingtools::unit::create_test_shim;

//     use anyhow::Result;

//     #[test]
//     fn tests_are_run_when_any_change_is_detected() -> Result<()> {
//         // given
//         init_tracing();
//         let (test_runner_spy, test_runner) = tracked(working(TestsRunStatus::Success));
//         let noop_state = noop();
//         let shim = create_test_shim()?;
//         TestsShell::new(shim.bus()).run(test_runner, noop_state.reader());

//         // when
//         shim.simulate_change()?;

//         // then
//         assert!(test_runner_spy.run_called());

//         Ok(())
//     }

//     #[test]
//     fn when_tests_pass_there_is_correct_event_on_the_bus() -> Result<()> {
//         // given
//         init_tracing();
//         let test_runner = working(TestsRunStatus::Success);
//         let noop_state = noop();
//         let shim = create_test_shim()?;
//         TestsShell::new(shim.bus()).run(test_runner, noop_state.reader());

//         // when
//         shim.simulate_change()?;
//         shim.ignore_event()?; // ignore BusEvent::ChangeDetected

//         // then
//         assert!(shim.event_on_bus(&BusEvent::TestsPassed)?);

//         Ok(())
//     }

//     #[test]
//     fn when_tests_fail_there_is_correct_event_on_the_bus() -> Result<()> {
//         // given
//         init_tracing();
//         let test_runner = working(TestsRunStatus::Failure);
//         let noop_state = noop();
//         let shim = create_test_shim()?;
//         TestsShell::new(shim.bus()).run(test_runner, noop_state.reader());

//         // when
//         shim.simulate_change()?;
//         shim.ignore_event()?; // ignore BusEvent::ChangeDetected

//         // then
//         assert!(shim.event_on_bus(&BusEvent::TestsFailed)?);

//         Ok(())
//     }

//     #[test]
//     fn when_test_runner_fails_correct_event_is_sent() -> Result<()> {
//         // given
//         init_tracing();
//         let test_runner = failing();
//         let noop_state = noop();
//         let shim = create_test_shim()?;
//         TestsShell::new(shim.bus()).run(test_runner, noop_state.reader());

//         // when
//         shim.simulate_change()?;
//         shim.ignore_event()?; // ignore BusEvent::ChangeDetected

//         // then
//         assert!(shim.event_on_bus(&BusEvent::TestsFailed)?);

//         Ok(())
//     }
// }
