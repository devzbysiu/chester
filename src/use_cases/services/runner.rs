use crate::result::RunnerErr;
use crate::use_cases::bus::{BusEvent, EventBus};
use crate::use_cases::test_runner::{TestRunner, TestsStatus};

use std::thread;
use tracing::{debug, trace};

type Result<T> = std::result::Result<T, RunnerErr>;

pub struct TestRunnerShell {
    #[allow(unused)]
    bus: EventBus,
}

impl TestRunnerShell {
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    #[allow(clippy::unused_self)]
    pub fn run(self, test_runner: TestRunner) {
        let sub = self.bus.subscriber();
        let publ = self.bus.publisher();
        thread::spawn(move || -> Result<()> {
            loop {
                if let Ok(BusEvent::ChangeDetected) = sub.recv() {
                    if let Ok(TestsStatus::Success) = test_runner.run() {
                        debug!("tests passed");
                        publ.send(BusEvent::TestsPassed)?;
                    } else {
                        debug!("tests failed");
                        publ.send(BusEvent::TestsFailed)?;
                    }
                } else {
                    trace!("no change detected");
                }
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::tracing::init_tracing;
    use crate::testingtools::services::test_runner::{failing, tracked, working};
    use crate::testingtools::unit::create_test_shim;

    use anyhow::Result;

    #[test]
    fn tests_are_run_when_any_change_is_detected() -> Result<()> {
        // given
        init_tracing();
        let (test_runner_spy, test_runner) = tracked(working(TestsStatus::Success));
        let shim = create_test_shim()?;
        TestRunnerShell::new(shim.bus()).run(test_runner);

        // when
        shim.simulate_change()?;

        // then
        assert!(test_runner_spy.run_called());

        Ok(())
    }

    #[test]
    fn when_tests_pass_there_is_correct_event_on_the_bus() -> Result<()> {
        // given
        init_tracing();
        let test_runner = working(TestsStatus::Success);
        let shim = create_test_shim()?;
        TestRunnerShell::new(shim.bus()).run(test_runner);

        // when
        shim.simulate_change()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(shim.event_on_bus(&BusEvent::TestsPassed)?);

        Ok(())
    }

    #[test]
    fn when_tests_fail_there_is_correct_event_on_the_bus() -> Result<()> {
        // given
        init_tracing();
        let test_runner = working(TestsStatus::Failure);
        let shim = create_test_shim()?;
        TestRunnerShell::new(shim.bus()).run(test_runner);

        // when
        shim.simulate_change()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(shim.event_on_bus(&BusEvent::TestsFailed)?);

        Ok(())
    }

    #[test]
    fn when_test_runner_fails_correct_event_is_sent() -> Result<()> {
        // given
        init_tracing();
        let test_runner = failing();
        let shim = create_test_shim()?;
        TestRunnerShell::new(shim.bus()).run(test_runner);

        // when
        shim.simulate_change()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(shim.event_on_bus(&BusEvent::TestsFailed)?);

        Ok(())
    }
}
