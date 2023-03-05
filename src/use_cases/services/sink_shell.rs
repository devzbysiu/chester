use crate::entities::coverage::CoverageState;
use crate::entities::status::TestsState;
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
                    Ok(BusEvent::ChangeDetected) => {
                        st.tests(TestsState::Pending)?;
                        st.coverage(CoverageState::Pending)?;
                    }
                    Ok(BusEvent::TestsPassed) => st.tests(TestsState::Success)?,
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
    fn when_tests_succeed_success_is_writen_to_state() -> Result<()> {
        // given
        let shim = create_test_shim()?;
        let (state_spy, state) = tracked(&working());
        ResultsSinkShell::new(shim.bus()).run(state.writer());

        // when
        shim.simulate_tests_succeeded()?;
        shim.ignore_event()?;

        // then
        assert!(state_spy.tests_status_called_with_val(&TestsState::Success));

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
    fn when_tests_start_pending_is_written_to_state() -> Result<()> {
        // given
        let shim = create_test_shim()?;
        let (state_spy, state) = tracked(&working());
        ResultsSinkShell::new(shim.bus()).run(state.writer());

        // when
        shim.simulate_change()?;
        shim.ignore_event()?;

        // then
        assert!(state_spy.tests_status_called_with_val(&TestsState::Pending));

        Ok(())
    }
}
