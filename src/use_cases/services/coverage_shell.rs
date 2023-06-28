use crate::entities::coverage::CoverageState;
use crate::result::CoverageErr;
use crate::use_cases::bus::{BusEvent, EventBus};
use crate::use_cases::coverage_runner::{CoverageRunStatus, CoverageRunner};
use crate::use_cases::state::State;

use std::thread;
use tracing::{debug, instrument, trace};

type Result<T> = std::result::Result<T, CoverageErr>;

/// When tests set changed, it runs code coverage, updates coverage state and publishes result of
/// the coverage.
///
/// `CoverageShell` first waits for the event describing the tests set.
/// If tests set did not change, nothing happens.
/// If tests set changed, `CoverageShell` sets the coverage state as `CoverageState::Pending`, then
/// runs the tests coverage.
/// Coverage state is updated accordingly to the result of the coverage.
///
/// It's the end of the pipeline, no event is published.
pub struct CoverageShell {
    bus: EventBus,
}

impl CoverageShell {
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    #[instrument(skip(self, cr, st))]
    pub fn run(self, cr: CoverageRunner, st: State) {
        let sub = self.bus.subscriber();
        thread::spawn(move || -> Result<()> {
            let sw = st.writer();
            loop {
                let Ok(BusEvent::TestsSetChanged) = sub.recv() else {
                    trace!("no change detected");
                    continue;
                };

                debug!("running coverage");
                sw.coverage(CoverageState::Pending)?;
                let Ok(CoverageRunStatus::Success(val)) = cr.run(st.reader().repo_root()?) else {
                    debug!("coverage failed");
                    sw.coverage(CoverageState::Failure)?;
                    continue;
                };

                debug!("coverage calculated: {val}");
                sw.coverage(CoverageState::Success(val))?;
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::tracing::init_tracing;
    use crate::testingtools::coverage_runner::{failing, tracked, working};
    use crate::testingtools::state;
    use crate::testingtools::unit::create_test_shim;

    use anyhow::Result;

    #[test]
    fn coverage_is_not_started_when_change_is_detected() -> Result<()> {
        // given
        init_tracing();
        let (cov_runner_spy, cov_runner) = tracked(working(CoverageRunStatus::Success(20.0)));
        let noop_state = state::noop();
        let shim = create_test_shim()?;
        CoverageShell::new(shim.bus()).run(cov_runner, noop_state);

        // when
        shim.simulate_change()?;

        // then
        assert!(!cov_runner_spy.run_called());

        Ok(())
    }

    #[test]
    fn coverage_is_started_when_tests_changed() -> Result<()> {
        // given
        init_tracing();
        let (cov_runner_spy, cov_runner) = tracked(working(CoverageRunStatus::Success(20.0)));
        let noop_state = state::noop();
        let shim = create_test_shim()?;
        CoverageShell::new(shim.bus()).run(cov_runner, noop_state);

        // when
        shim.simulate_tests_changed()?;

        // then
        assert!(cov_runner_spy.run_called());

        Ok(())
    }

    #[test]
    fn when_coverage_pass_state_is_set_to_pending_then_failure() -> Result<()> {
        // given
        init_tracing();
        let cov_runner = working(CoverageRunStatus::Success(20.0));
        let (spy, state) = state::tracked(&state::noop());
        let shim = create_test_shim()?;
        CoverageShell::new(shim.bus()).run(cov_runner, state);

        // when
        shim.simulate_tests_changed()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(spy.coverage_state_called_with_val(&CoverageState::Pending));
        assert!(spy.coverage_state_called_with_val(&CoverageState::Success(20.0)));

        Ok(())
    }

    #[test]
    fn when_coverage_fail_state_is_set_to_pending_then_failure() -> Result<()> {
        // given
        init_tracing();
        let coverage_runner = working(CoverageRunStatus::Failure);
        let (spy, state) = state::tracked(&state::noop());
        let shim = create_test_shim()?;
        CoverageShell::new(shim.bus()).run(coverage_runner, state);

        // when
        shim.simulate_tests_changed()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(spy.coverage_state_called_with_val(&CoverageState::Pending));
        assert!(spy.coverage_state_called_with_val(&CoverageState::Failure));

        Ok(())
    }

    #[test]
    fn when_coverage_runner_fail_state_is_set_to_pending_then_failure() -> Result<()> {
        // given
        init_tracing();
        let coverage_runner = failing();
        let (spy, state) = state::tracked(&state::noop());
        let shim = create_test_shim()?;
        CoverageShell::new(shim.bus()).run(coverage_runner, state);

        // when
        shim.simulate_tests_changed()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(spy.coverage_state_called_with_val(&CoverageState::Pending));
        assert!(spy.coverage_state_called_with_val(&CoverageState::Failure));

        Ok(())
    }
}
