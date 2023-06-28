use crate::entities::tests::TestsState;
use crate::result::RunnerErr;
use crate::use_cases::bus::{BusEvent, EventBus};
use crate::use_cases::state::State;
use crate::use_cases::test_runner::{TestRunner, TestsRunStatus};

use std::thread;
use tracing::{debug, instrument, trace};

type Result<T> = std::result::Result<T, RunnerErr>;

/// Runs tests and updates tests state when check passed.
///
/// `TestsShell` first waits for the event describing the result of the check stage.
/// If check stage failed, nothing happens.
/// If check stage succeeds, `TestsShell` sets the tests state as `TestsState::Pending`, then
/// runs the tests.
/// Tests state is updated accordingly to the result of the tests.
///
/// It publishes following events:
/// - `BusEvent::TestsPassed` - when check passed and tests passed as well
/// - `BusEvent::TestsFailed` - when check passed, but tests failed
pub struct TestsShell {
    bus: EventBus,
}

impl TestsShell {
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    #[instrument(skip(self, tr, st))]
    pub fn run(self, tr: TestRunner, st: State) {
        let sub = self.bus.subscriber();
        let publ = self.bus.publisher();
        thread::spawn(move || -> Result<()> {
            let sw = st.writer();
            loop {
                let Ok(BusEvent::CheckPassed) = sub.recv() else {
                    trace!("check failed, skipping tests");
                    continue;
                };

                debug!("running tests");
                sw.tests(TestsState::Pending)?;
                let Ok(TestsRunStatus::Success) = tr.run(st.reader().repo_root()?) else {
                    debug!("tests failed");
                    sw.tests(TestsState::Failure)?;
                    publ.send(BusEvent::TestsFailed)?;
                    continue;
                };

                debug!("tests passed");
                sw.tests(TestsState::Success)?;
                publ.send(BusEvent::TestsPassed)?;
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::tracing::init_tracing;
    use crate::testingtools::state;
    use crate::testingtools::test_runner::{failing, tracked, working};
    use crate::testingtools::unit::create_test_shim;

    use anyhow::Result;

    #[test]
    fn tests_are_started_when_check_passed() -> Result<()> {
        // given
        init_tracing();
        let (test_runner_spy, test_runner) = tracked(working(TestsRunStatus::Success));
        let noop_state = state::noop();
        let shim = create_test_shim()?;
        TestsShell::new(shim.bus()).run(test_runner, noop_state);

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
        let noop_state = state::noop();
        let shim = create_test_shim()?;
        TestsShell::new(shim.bus()).run(test_runner, noop_state);

        // when
        shim.simulate_check_passed()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(shim.event_on_bus(&BusEvent::TestsPassed)?);

        Ok(())
    }

    #[test]
    fn when_tests_pass_state_is_set_to_pending_then_success() -> Result<()> {
        // given
        init_tracing();
        let test_runner = working(TestsRunStatus::Success);
        let (spy, state) = state::tracked(&state::noop());
        let shim = create_test_shim()?;
        TestsShell::new(shim.bus()).run(test_runner, state);

        // when
        shim.simulate_check_passed()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(spy.tests_state_called_with_val(&TestsState::Pending));
        assert!(spy.tests_state_called_with_val(&TestsState::Success));

        Ok(())
    }

    #[test]
    fn when_tests_fail_there_is_correct_event_on_the_bus() -> Result<()> {
        // given
        init_tracing();
        let test_runner = working(TestsRunStatus::Failure);
        let noop_state = state::noop();
        let shim = create_test_shim()?;
        TestsShell::new(shim.bus()).run(test_runner, noop_state);

        // when
        shim.simulate_check_passed()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(shim.event_on_bus(&BusEvent::TestsFailed)?);

        Ok(())
    }

    #[test]
    fn when_tests_fail_state_is_set_to_pending_then_failure() -> Result<()> {
        // given
        init_tracing();
        let test_runner = working(TestsRunStatus::Failure);
        let (spy, state) = state::tracked(&state::noop());
        let shim = create_test_shim()?;
        TestsShell::new(shim.bus()).run(test_runner, state);

        // when
        shim.simulate_check_passed()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(spy.tests_state_called_with_val(&TestsState::Pending));
        assert!(spy.tests_state_called_with_val(&TestsState::Failure));

        Ok(())
    }

    #[test]
    fn when_test_runner_fails_correct_event_is_sent() -> Result<()> {
        // given
        init_tracing();
        let test_runner = failing();
        let noop_state = state::noop();
        let shim = create_test_shim()?;
        TestsShell::new(shim.bus()).run(test_runner, noop_state);

        // when
        shim.simulate_check_passed()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(shim.event_on_bus(&BusEvent::TestsFailed)?);

        Ok(())
    }

    #[test]
    fn when_test_runner_fails_state_is_set_to_pending_then_failure() -> Result<()> {
        // given
        init_tracing();
        let test_runner = failing();
        let (spy, state) = state::tracked(&state::noop());
        let shim = create_test_shim()?;
        TestsShell::new(shim.bus()).run(test_runner, state);

        // when
        shim.simulate_check_passed()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(spy.tests_state_called_with_val(&TestsState::Pending));
        assert!(spy.tests_state_called_with_val(&TestsState::Failure));

        Ok(())
    }
}
