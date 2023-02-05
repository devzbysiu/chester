use crate::entities::status::Status;
use crate::result::SinkErr;
use crate::use_cases::bus::{BusEvent, EventBus};
use crate::use_cases::repo::RepoWrite;

use std::thread;
use tracing::error;

type Result<T> = std::result::Result<T, SinkErr>;

pub struct ResultsSinkShell {
    #[allow(unused)]
    bus: EventBus,
}

impl ResultsSinkShell {
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    #[allow(clippy::unused_self)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn run(&self, repo_write: RepoWrite) {
        let sub = self.bus.subscriber();
        thread::spawn(move || -> Result<()> {
            loop {
                match sub.recv() {
                    Ok(BusEvent::TestsPassed) => repo_write.status(Status::Success)?,
                    Ok(BusEvent::TestsFailed) => repo_write.status(Status::Failure)?,
                    Ok(BusEvent::ChangeDetected) => repo_write.status(Status::Pending)?,
                    Err(_) => error!("failed to recv bus event"),
                }
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::testingtools::services::repo::{tracked, working};
    use crate::testingtools::unit::create_test_shim;

    use anyhow::Result;

    #[test]
    fn when_tests_succeed_success_is_writen_to_repo() -> Result<()> {
        // given
        let shim = create_test_shim()?;
        let (repo_spy, repo) = tracked(&working());
        ResultsSinkShell::new(shim.bus()).run(repo.write());

        // when
        shim.simulate_tests_succeeded()?;
        shim.ignore_event()?;

        // then
        assert!(repo_spy.write_called_with_val(&Status::Success));

        Ok(())
    }

    #[test]
    fn when_tests_fail_failure_is_written_to_repo() -> Result<()> {
        // given
        let shim = create_test_shim()?;
        let (repo_spy, repo) = tracked(&working());
        ResultsSinkShell::new(shim.bus()).run(repo.write());

        // when
        shim.simulate_tests_failed()?;
        shim.ignore_event()?;

        // then
        assert!(repo_spy.write_called_with_val(&Status::Failure));

        Ok(())
    }

    #[test]
    fn when_tests_start_pending_is_written_to_repo() -> Result<()> {
        // given
        let shim = create_test_shim()?;
        let (repo_spy, repo) = tracked(&working());
        ResultsSinkShell::new(shim.bus()).run(repo.write());

        // when
        shim.simulate_change()?;
        shim.ignore_event()?;

        // then
        assert!(repo_spy.write_called_with_val(&Status::Pending));

        Ok(())
    }
}
