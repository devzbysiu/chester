use crate::entities::coverage::CoverageState;
use crate::result::CoverageErr;
use crate::use_cases::bus::{BusEvent, EventBus};
use crate::use_cases::coverage_runner::{CoverageRunStatus, CoverageRunner};
use crate::use_cases::state::State;

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

    #[instrument(skip(self, cr, st))]
    pub fn run(self, cr: CoverageRunner, st: State) {
        let sub = self.bus.subscriber();
        let publ = self.bus.publisher();
        thread::spawn(move || -> Result<()> {
            loop {
                if let Ok(BusEvent::TestsChanged) = sub.recv() {
                    debug!("running coverage");
                    st.writer().coverage(CoverageState::Pending)?;
                    if let Ok(CoverageRunStatus::Success(val)) = cr.run(st.reader().repo_root()?) {
                        debug!("coverage calculated: {val}");
                        st.writer().coverage(CoverageState::Success(val))?;
                        publ.send(BusEvent::GotCoverage(val))?;
                    } else {
                        debug!("coverage failed");
                        st.writer().coverage(CoverageState::Failure)?;
                        publ.send(BusEvent::CoverageFailed)?;
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
    fn when_coverage_pass_there_is_correct_event_on_the_bus() -> Result<()> {
        // given
        init_tracing();
        let cov_runner = working(CoverageRunStatus::Success(20.0));
        let noop_state = state::noop();
        let shim = create_test_shim()?;
        CoverageShell::new(shim.bus()).run(cov_runner, noop_state);

        // when
        shim.simulate_tests_changed()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(shim.event_on_bus(&BusEvent::GotCoverage(20.0))?);

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
    fn when_coverage_fail_there_is_correct_event_on_the_bus() -> Result<()> {
        // given
        init_tracing();
        let coverage_runner = working(CoverageRunStatus::Failure);
        let noop_state = state::noop();
        let shim = create_test_shim()?;
        CoverageShell::new(shim.bus()).run(coverage_runner, noop_state);

        // when
        shim.simulate_tests_changed()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(shim.event_on_bus(&BusEvent::CoverageFailed)?);

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
    fn when_coverage_runner_fail_there_is_correct_event_on_the_bus() -> Result<()> {
        // given
        init_tracing();
        let coverage_runner = failing();
        let noop_state = state::noop();
        let shim = create_test_shim()?;
        CoverageShell::new(shim.bus()).run(coverage_runner, noop_state);

        // when
        shim.simulate_tests_changed()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(shim.event_on_bus(&BusEvent::CoverageFailed)?);

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
