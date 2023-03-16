use crate::result::RunnerErr;
use crate::use_cases::bus::{BusEvent, EventBus};
use crate::use_cases::state::StateReader;
use crate::use_cases::test_runner::{TestRunner, TestsRunStatus};

use std::thread;
use tracing::{debug, instrument, trace};

type Result<T> = std::result::Result<T, RunnerErr>;

pub struct TestsShell {
    bus: EventBus,
}

impl TestsShell {
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    #[instrument(skip(self, test_runner))]
    pub fn run(self, test_runner: TestRunner, state: StateReader) {
        let sub = self.bus.subscriber();
        let publ = self.bus.publisher();
        thread::spawn(move || -> Result<()> {
            loop {
                if let Ok(BusEvent::CheckPassed) = sub.recv() {
                    debug!("running tests");
                    if let Ok(TestsRunStatus::Success) = test_runner.run_all(state.repo_root()?) {
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
    use crate::testingtools::state::noop;
    use crate::testingtools::test_runner::{failing, tracked, working};
    use crate::testingtools::unit::create_test_shim;

    use anyhow::Result;

    #[test]
    fn tests_are_not_started_when_any_change_is_detected() -> Result<()> {
        // given
        init_tracing();
        let (test_runner_spy, test_runner) = tracked(working(TestsRunStatus::Success));
        let noop_state = noop();
        let shim = create_test_shim()?;
        TestsShell::new(shim.bus()).run(test_runner, noop_state.reader());

        // when
        shim.simulate_change()?;

        // then
        assert!(!test_runner_spy.run_called());

        Ok(())
    }

    #[test]
    fn tests_are_started_when_check_passed() -> Result<()> {
        // given
        init_tracing();
        let (test_runner_spy, test_runner) = tracked(working(TestsRunStatus::Success));
        let noop_state = noop();
        let shim = create_test_shim()?;
        TestsShell::new(shim.bus()).run(test_runner, noop_state.reader());

        // when
        shim.simulate_check_passed()?;

        // then
        assert!(test_runner_spy.run_called());

        Ok(())
    }

    #[test]
    fn when_tests_pass_there_is_correct_event_on_the_bus() -> Result<()> {
        // given
        init_tracing();
        let test_runner = working(TestsRunStatus::Success);
        let noop_state = noop();
        let shim = create_test_shim()?;
        TestsShell::new(shim.bus()).run(test_runner, noop_state.reader());

        // when
        shim.simulate_check_passed()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(shim.event_on_bus(&BusEvent::TestsPassed)?);

        Ok(())
    }

    #[test]
    fn when_tests_fail_there_is_correct_event_on_the_bus() -> Result<()> {
        // given
        init_tracing();
        let test_runner = working(TestsRunStatus::Failure);
        let noop_state = noop();
        let shim = create_test_shim()?;
        TestsShell::new(shim.bus()).run(test_runner, noop_state.reader());

        // when
        shim.simulate_check_passed()?;
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
        let noop_state = noop();
        let shim = create_test_shim()?;
        TestsShell::new(shim.bus()).run(test_runner, noop_state.reader());

        // when
        shim.simulate_check_passed()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(shim.event_on_bus(&BusEvent::TestsFailed)?);

        Ok(())
    }
}
