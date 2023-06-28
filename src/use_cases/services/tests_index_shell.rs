use crate::result::IndexErr;
use crate::use_cases::bus::{BusEvent, EventBus};
use crate::use_cases::state::State;
use crate::use_cases::tests_index::{IndexStatus, TestsIndex};

use std::thread;
use tracing::{debug, error, instrument, trace};

type Result<T> = std::result::Result<T, IndexErr>;

/// Triggers code coverage if tests set is changed.
///
/// When the tests are run, `TestsIndexShell` checks tests status.
/// If the tests failed, then nothing more happens.
/// If the tests finished with success, then it refreshes tests index.
///
/// It publishes following events:
/// - `BusEvent::TestsSetChanged` - when tests passed and tests set is changed
/// - `BusEvent::TestsSetNotChanged` - when tests passed, but tests set is not changed
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
                let Ok(BusEvent::TestsPassed) = sub.recv() else {
                    trace!("tests failed, skipping index refresh");
                    continue;
                };

                debug!("checking if tests changed");
                match index.refresh(st.reader().repo_root()?) {
                    Ok(IndexStatus::TestsSetChanged) => {
                        debug!("tests change detected");
                        publ.send(BusEvent::TestsSetChanged)?;
                    }
                    Ok(IndexStatus::TestsSetNotChanged) => {
                        debug!("tests not changed");
                        publ.send(BusEvent::TestsSetNotChanged)?;
                    }
                    Ok(IndexStatus::Failure) => error!("index refresh failed"),
                    Err(e) => error!("error while running index refresh: {e:?}"),
                }
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::tracing::init_tracing;
    use crate::testingtools::state;
    use crate::testingtools::tests_index::{failing, tracked, working};
    use crate::testingtools::unit::create_test_shim;

    use anyhow::Result;

    #[test]
    fn index_is_started_when_tests_passed() -> Result<()> {
        // given
        init_tracing();
        let (spy, index) = tracked(working(IndexStatus::TestsSetChanged));
        let state = state::noop();
        let shim = create_test_shim()?;
        TestsIndexShell::new(shim.bus()).run(index, state);

        // when
        shim.simulate_tests_passed()?;

        // then
        assert!(spy.refresh_called());

        Ok(())
    }

    #[test]
    fn when_tests_changed_there_is_correct_event_on_the_bus() -> Result<()> {
        // given
        init_tracing();
        let index = working(IndexStatus::TestsSetChanged);
        let state = state::noop();
        let shim = create_test_shim()?;
        TestsIndexShell::new(shim.bus()).run(index, state);

        // when
        shim.simulate_tests_passed()?;
        shim.ignore_event()?;

        // then
        assert!(shim.event_on_bus(&BusEvent::TestsSetChanged)?);

        Ok(())
    }

    #[test]
    fn when_tests_did_not_change_there_is_correct_event_on_the_bus() -> Result<()> {
        // given
        init_tracing();
        let index = working(IndexStatus::TestsSetNotChanged);
        let state = state::noop();
        let shim = create_test_shim()?;
        TestsIndexShell::new(shim.bus()).run(index, state);

        // when
        shim.simulate_tests_passed()?;
        shim.ignore_event()?;

        // then
        assert!(shim.event_on_bus(&BusEvent::TestsSetNotChanged)?);

        Ok(())
    }

    #[test]
    fn when_index_fails_to_refresh_nothing_is_on_the_bus() -> Result<()> {
        // given
        init_tracing();
        let index = working(IndexStatus::Failure);
        let state = state::noop();
        let shim = create_test_shim()?;
        TestsIndexShell::new(shim.bus()).run(index, state);

        // when
        shim.simulate_tests_passed()?;
        shim.ignore_event()?;

        // then
        assert!(shim.no_event_on_bus()?);

        Ok(())
    }

    #[test]
    fn when_indexing_command_fails_there_is_no_event_on_the_bus() -> Result<()> {
        // given
        init_tracing();
        let index = failing();
        let state = state::noop();
        let shim = create_test_shim()?;
        TestsIndexShell::new(shim.bus()).run(index, state);

        // when
        shim.simulate_tests_passed()?;
        shim.ignore_event()?;

        // then
        assert!(shim.no_event_on_bus()?);

        Ok(())
    }
}
