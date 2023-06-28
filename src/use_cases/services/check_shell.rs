use crate::entities::check::CheckState;
use crate::result::CheckErr;
use crate::use_cases::bus::{BusEvent, EventBus};
use crate::use_cases::check_runner::{CheckRunStatus, CheckRunner};
use crate::use_cases::state::State;

use std::thread;
use tracing::{debug, instrument, trace};

type Result<T> = std::result::Result<T, CheckErr>;

/// When change is detected, it runs check, updates check state and publishes result of the check.
///
/// `CheckShell` first waits for the event meaning the change of files was detected.
/// If change was not detected, nothing happens.
/// If change was detected, `CheckShell` sets the check state as `CheckState::Pending`, then
/// runs the check.
/// Check state is updated accordingly to the result of the tests.
///
/// It publishes following events:
/// - `BusEvent::CheckPassed` - when change was detected and check passed as well
/// - `BusEvent::CheckFailed` - when change is detected, but check failed
pub struct CheckShell {
    bus: EventBus,
}

impl CheckShell {
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    #[instrument(skip(self, cr, st))]
    pub fn run(self, cr: CheckRunner, st: State) {
        let sub = self.bus.subscriber();
        let publ = self.bus.publisher();
        thread::spawn(move || -> Result<()> {
            let sw = st.writer();
            loop {
                let Ok(BusEvent::ChangeDetected) = sub.recv() else {
                    trace!("no change detected");
                    continue;
                };

                debug!("running check");
                sw.check(CheckState::Pending)?;
                let Ok(CheckRunStatus::Success) = cr.run(st.reader().repo_root()?) else {
                    debug!("check failed");
                    sw.check(CheckState::Failure)?;
                    publ.send(BusEvent::CheckFailed)?;
                    continue;
                };

                debug!("check passed");
                sw.check(CheckState::Success)?;
                publ.send(BusEvent::CheckPassed)?;
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::tracing::init_tracing;
    use crate::testingtools::check_runner::{failing, tracked, working};
    use crate::testingtools::state;
    use crate::testingtools::unit::create_test_shim;

    use anyhow::Result;

    #[test]
    fn check_is_run_when_any_change_is_detected() -> Result<()> {
        // given
        init_tracing();
        let (check_runner_spy, check_runner) = tracked(working(CheckRunStatus::Success));
        let noop_state = state::noop();
        let shim = create_test_shim()?;
        CheckShell::new(shim.bus()).run(check_runner, noop_state);

        // when
        shim.simulate_change()?;

        // then
        assert!(check_runner_spy.run_called());

        Ok(())
    }

    #[test]
    fn when_check_pass_there_is_correct_event_on_the_bus() -> Result<()> {
        // given
        init_tracing();
        let check_runner = working(CheckRunStatus::Success);
        let noop_state = state::noop();
        let shim = create_test_shim()?;
        CheckShell::new(shim.bus()).run(check_runner, noop_state);

        // when
        shim.simulate_change()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(shim.event_on_bus(&BusEvent::CheckPassed)?);

        Ok(())
    }

    #[test]
    fn when_check_succeeds_state_is_set_to_pending_then_success() -> Result<()> {
        // given
        init_tracing();
        let check_runner = working(CheckRunStatus::Success);
        let (spy, state) = state::tracked(&state::noop());
        let shim = create_test_shim()?;
        CheckShell::new(shim.bus()).run(check_runner, state);

        // when
        shim.simulate_change()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(spy.check_state_called_with_val(&CheckState::Pending));
        assert!(spy.check_state_called_with_val(&CheckState::Success));

        Ok(())
    }

    #[test]
    fn when_check_fail_there_is_correct_event_on_the_bus() -> Result<()> {
        // given
        init_tracing();
        let check_runner = working(CheckRunStatus::Failure);
        let noop_state = state::noop();
        let shim = create_test_shim()?;
        CheckShell::new(shim.bus()).run(check_runner, noop_state);

        // when
        shim.simulate_change()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(shim.event_on_bus(&BusEvent::CheckFailed)?);

        Ok(())
    }

    #[test]
    fn when_check_fail_state_is_set_to_pending_then_failure() -> Result<()> {
        // given
        init_tracing();
        let check_runner = working(CheckRunStatus::Failure);
        let (spy, state) = state::tracked(&state::noop());
        let shim = create_test_shim()?;
        CheckShell::new(shim.bus()).run(check_runner, state);

        // when
        shim.simulate_change()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(spy.check_state_called_with_val(&CheckState::Pending));
        assert!(spy.check_state_called_with_val(&CheckState::Failure));

        Ok(())
    }

    #[test]
    fn when_check_runner_fails_correct_event_is_sent() -> Result<()> {
        // given
        init_tracing();
        let check_runner = failing();
        let noop_state = state::noop();
        let shim = create_test_shim()?;
        CheckShell::new(shim.bus()).run(check_runner, noop_state);

        // when
        shim.simulate_change()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(shim.event_on_bus(&BusEvent::CheckFailed)?);

        Ok(())
    }

    #[test]
    fn when_check_runner_fails_status_is_set_to_pending_then_failure() -> Result<()> {
        // given
        init_tracing();
        let check_runner = failing();
        let (spy, state) = state::tracked(&state::noop());
        let shim = create_test_shim()?;
        CheckShell::new(shim.bus()).run(check_runner, state);

        // when
        shim.simulate_change()?;
        shim.ignore_event()?; // ignore BusEvent::ChangeDetected

        // then
        assert!(spy.check_state_called_with_val(&CheckState::Pending));
        assert!(spy.check_state_called_with_val(&CheckState::Failure));

        Ok(())
    }
}
