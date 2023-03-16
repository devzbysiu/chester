use crate::entities::check::CheckState;
use crate::entities::coverage::CoverageState;
use crate::entities::tests::TestsState;
use crate::result::SinkErr;
use crate::use_cases::bus::{BusEvent, EventBus};
use crate::use_cases::state::StateWriter;

use std::thread;
use tracing::{debug, error, instrument};

type Result<T> = std::result::Result<T, SinkErr>;

pub struct ResultsSinkShell {
    bus: EventBus,
}

impl ResultsSinkShell {
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    #[instrument(skip(self, st))]
    pub fn run(&self, st: StateWriter) {
        let sub = self.bus.subscriber();
        thread::spawn(move || -> Result<()> {
            loop {
                let event = sub.recv();
                debug!("received: {event:?}");
                match event {
                    Ok(BusEvent::ChangeDetected) => st.check(CheckState::Pending)?,
                    Ok(BusEvent::CheckPassed) => {
                        st.check(CheckState::Success)?;
                        st.tests(TestsState::Pending)?;
                    }
                    Ok(BusEvent::CheckFailed) => st.check(CheckState::Failure)?,
                    Ok(BusEvent::TestsPassed) => {
                        st.tests(TestsState::Success)?;
                        st.coverage(CoverageState::Pending)?;
                    }
                    Ok(BusEvent::TestsFailed) => st.tests(TestsState::Failure)?,
                    Ok(BusEvent::GotCoverage(val)) => st.coverage(CoverageState::Success(val))?,
                    Ok(BusEvent::CoverageFailed) => st.coverage(CoverageState::Failure)?,
                    Err(_) => error!("failed to recv bus event"),
                }
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::testingtools::state::{tracked, working};
    use crate::testingtools::unit::create_test_shim;

    use anyhow::Result;

    #[test]
    fn when_change_is_detected_check_status_becomes_pending() -> Result<()> {
        // given
        let shim = create_test_shim()?;
        let (state_spy, state) = tracked(&working());
        ResultsSinkShell::new(shim.bus()).run(state.writer());

        // when
        shim.simulate_change()?;
        shim.ignore_event()?;

        // then
        assert!(state_spy.check_status_called_with_val(&CheckState::Pending));

        Ok(())
    }

    #[test]
    fn when_check_passes_success_is_written_to_state() -> Result<()> {
        // given
        let shim = create_test_shim()?;
        let (state_spy, state) = tracked(&working());
        ResultsSinkShell::new(shim.bus()).run(state.writer());

        // when
        shim.simulate_check_passed()?;
        shim.ignore_event()?;

        // then
        assert!(state_spy.check_status_called_with_val(&CheckState::Success));

        Ok(())
    }

    #[test]
    fn successful_check_sets_tests_status_to_pending() -> Result<()> {
        // given
        let shim = create_test_shim()?;
        let (state_spy, state) = tracked(&working());
        ResultsSinkShell::new(shim.bus()).run(state.writer());

        // when
        shim.simulate_check_passed()?;
        shim.ignore_event()?;

        // then
        assert!(state_spy.tests_status_called_with_val(&TestsState::Pending));

        Ok(())
    }

    #[test]
    fn when_check_fails_failure_is_written_to_state() -> Result<()> {
        // given
        let shim = create_test_shim()?;
        let (state_spy, state) = tracked(&working());
        ResultsSinkShell::new(shim.bus()).run(state.writer());

        // when
        shim.simulate_check_failed()?;
        shim.ignore_event()?;

        // then
        assert!(state_spy.check_status_called_with_val(&CheckState::Failure));

        Ok(())
    }

    #[test]
    fn when_tests_succeeds_success_is_writen_to_state() -> Result<()> {
        // given
        let shim = create_test_shim()?;
        let (state_spy, state) = tracked(&working());
        ResultsSinkShell::new(shim.bus()).run(state.writer());

        // when
        shim.simulate_tests_passed()?;
        shim.ignore_event()?;

        // then
        assert!(state_spy.tests_status_called_with_val(&TestsState::Success));

        Ok(())
    }

    #[test]
    fn when_tests_succeeds_coverage_status_becomes_pending() -> Result<()> {
        // given
        let shim = create_test_shim()?;
        let (state_spy, state) = tracked(&working());
        ResultsSinkShell::new(shim.bus()).run(state.writer());

        // when
        shim.simulate_tests_passed()?;
        shim.ignore_event()?;

        // then
        assert!(state_spy.coverage_status_called_with_val(&CoverageState::Pending));

        Ok(())
    }

    #[test]
    fn when_tests_fail_failure_is_written_to_state() -> Result<()> {
        // given
        let shim = create_test_shim()?;
        let (state_spy, state) = tracked(&working());
        ResultsSinkShell::new(shim.bus()).run(state.writer());

        // when
        shim.simulate_tests_failed()?;
        shim.ignore_event()?;

        // then
        assert!(state_spy.tests_status_called_with_val(&TestsState::Failure));

        Ok(())
    }

    #[test]
    fn when_coverage_succeeds_success_is_writen_to_state() -> Result<()> {
        // given
        let shim = create_test_shim()?;
        let (state_spy, state) = tracked(&working());
        ResultsSinkShell::new(shim.bus()).run(state.writer());

        // when
        shim.simulate_coverage_passed()?;
        shim.ignore_event()?;

        // then
        assert!(state_spy.coverage_status_called_with_val(&CoverageState::Success(90.0)));

        Ok(())
    }

    #[test]
    fn when_coverage_fails_failure_is_writen_to_state() -> Result<()> {
        // given
        let shim = create_test_shim()?;
        let (state_spy, state) = tracked(&working());
        ResultsSinkShell::new(shim.bus()).run(state.writer());

        // when
        shim.simulate_coverage_failed()?;
        shim.ignore_event()?;

        // then
        assert!(state_spy.coverage_status_called_with_val(&CoverageState::Failure));

        Ok(())
    }
}
