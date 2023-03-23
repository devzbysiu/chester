use crate::result::IndexErr;
use crate::use_cases::bus::{BusEvent, EventBus};
use crate::use_cases::state::State;
use crate::use_cases::tests_index::{IndexStatus, TestsIndex};

use std::thread;
use tracing::{debug, error, instrument, trace};

type Result<T> = std::result::Result<T, IndexErr>;

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
                if let Ok(BusEvent::TestsPassed) = sub.recv() {
                    debug!("checking if tests changed");
                    match index.refresh(st.reader().repo_root()?) {
                        Ok(IndexStatus::TestsChanged) => {
                            debug!("tests change detected");
                            publ.send(BusEvent::TestsChanged)?;
                        }
                        Ok(IndexStatus::TestsNotChanged) => {
                            debug!("tests not changed");
                            publ.send(BusEvent::TestsNotChanged)?;
                        }
                        Ok(IndexStatus::Failure) => error!("index refresh failed"),
                        Err(e) => error!("error while running index refresh: {e:?}"),
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
    use crate::testingtools::state;
    use crate::testingtools::tests_index::{failing, tracked, working};
    use crate::testingtools::unit::create_test_shim;

    use anyhow::Result;

    #[test]
    fn index_is_started_when_tests_passed() -> Result<()> {
        // given
        init_tracing();
        let (spy, index) = tracked(working(IndexStatus::TestsChanged));
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
        let index = working(IndexStatus::TestsChanged);
        let state = state::noop();
        let shim = create_test_shim()?;
        TestsIndexShell::new(shim.bus()).run(index, state);

        // when
        shim.simulate_tests_passed()?;
        shim.ignore_event()?;

        // then
        assert!(shim.event_on_bus(&BusEvent::TestsChanged)?);

        Ok(())
    }

    #[test]
    fn when_tests_did_not_change_there_is_correct_event_on_the_bus() -> Result<()> {
        // given
        init_tracing();
        let index = working(IndexStatus::TestsNotChanged);
        let state = state::noop();
        let shim = create_test_shim()?;
        TestsIndexShell::new(shim.bus()).run(index, state);

        // when
        shim.simulate_tests_passed()?;
        shim.ignore_event()?;

        // then
        assert!(shim.event_on_bus(&BusEvent::TestsNotChanged)?);

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
