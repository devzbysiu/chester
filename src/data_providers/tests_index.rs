use std::cell::RefCell;
use std::collections::BTreeSet;

use crate::configuration::config::Config;
use crate::entities::repo_root::RepoRoot;
use crate::entities::tests::TestsState;
use crate::result::IndexErr;
use crate::use_cases::state::StateReader;
use crate::use_cases::tests_index::{IndexStatus, TIndex, TestsIndex};

use tracing::{debug, instrument};

type TestsSet = RefCell<BTreeSet<String>>;

#[derive(Debug)]
pub struct DefaultTestsIndex {
    cfg: Config,
    curr_tests: TestsSet,
    state: StateReader,
}

impl DefaultTestsIndex {
    pub fn make(cfg: Config, state: StateReader) -> TestsIndex {
        Box::new(Self {
            cfg,
            curr_tests: RefCell::new(BTreeSet::new()),
            state,
        })
    }
}

impl TIndex for DefaultTestsIndex {
    #[instrument(skip(self))]
    fn refresh(&self, repo_root: RepoRoot) -> Result<IndexStatus, IndexErr> {
        if self.state.tests()? == TestsState::Failure {
            debug!("tests failed previously, they need to be rerun");
            return Ok(IndexStatus::TestsSetChanged);
        }

        let Ok(list_of_tests) = self.cfg.list_tests_cmd.stdout(repo_root) else {
            debug!("listing tests command failed");
            return Ok(IndexStatus::Failure);
        };

        let new_tests: Vec<String> = list_of_tests.lines().map(ToString::to_string).collect();
        let new_tests = BTreeSet::from_iter(new_tests);
        let mut curr_tests = self.curr_tests.borrow_mut();
        let different_tests_num = curr_tests.symmetric_difference(&new_tests).count();
        debug!("diff: {}", different_tests_num);

        if curr_tests.is_empty() || different_tests_num != 0 {
            debug!("tests changed, or initial set is empty");
            *curr_tests = new_tests;
            return Ok(IndexStatus::TestsSetChanged);
        }

        Ok(IndexStatus::TestsSetNotChanged)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::config::ConfigBuilder;
    use crate::configuration::tracing::init_tracing;
    use crate::data_providers::command::Cmd;
    use crate::testingtools::state::{noop, working_with, StateValues};

    use anyhow::Result;
    use fake::{Fake, Faker};
    use tempfile::tempdir;

    #[test]
    fn when_tests_previously_failed_tests_changed_status_is_returned() -> Result<()> {
        // given
        init_tracing();
        let cfg = ConfigBuilder::default().build()?;
        let state = working_with(StateValues {
            tests_state: TestsState::Failure,
            ..StateValues::default()
        });
        let index = DefaultTestsIndex::make(cfg, state.reader());
        let repo_root = RepoRoot::new(Faker.fake::<String>());

        // when
        let res = index.refresh(repo_root)?;

        // then
        assert_eq!(res, IndexStatus::TestsSetChanged);

        Ok(())
    }

    #[test]
    fn when_list_tests_command_fail_then_failure_status_is_returned() -> Result<()> {
        // given
        init_tracing();
        let cfg = ConfigBuilder::default()
            .list_tests_cmd(Cmd::new("cargo", &["test", "--", "--list"]))
            .build()?;
        let state = noop();
        let index = DefaultTestsIndex::make(cfg, state.reader());
        let repo_root = RepoRoot::new(Faker.fake::<String>());

        // when
        let res = index.refresh(repo_root)?;

        // then
        assert_eq!(res, IndexStatus::Failure);

        Ok(())
    }

    #[test]
    fn when_tests_did_not_change_correct_status_is_returned() -> Result<()> {
        // given
        init_tracing();
        let cfg = ConfigBuilder::default()
            .list_tests_cmd(Cmd::new("echo", &["test1\ntest2\ntest3"]))
            .build()?;
        let tmpdir = tempdir()?;
        let state = noop();
        let index = DefaultTestsIndex {
            cfg,
            curr_tests: RefCell::new(BTreeSet::from([
                "test1".into(),
                "test2".into(),
                "test3".into(),
            ])),
            state: state.reader(),
        };
        let repo_root = RepoRoot::new(&tmpdir);

        // when
        let res = index.refresh(repo_root)?;

        // then
        assert_eq!(res, IndexStatus::TestsSetNotChanged);

        Ok(())
    }

    #[test]
    fn when_initial_tests_set_is_empty_correct_status_is_returned() -> Result<()> {
        // given
        init_tracing();
        let cfg = ConfigBuilder::default()
            .list_tests_cmd(Cmd::new("echo", &["test1\ntest2\ntest3\n"]))
            .build()?;
        let tmpdir = tempdir()?;
        let state = noop();
        let index = DefaultTestsIndex::make(cfg, state.reader());
        let repo_root = RepoRoot::new(&tmpdir);

        // when
        let res = index.refresh(repo_root)?;

        // then
        assert_eq!(res, IndexStatus::TestsSetChanged);

        Ok(())
    }

    #[test]
    fn when_tests_changed_correct_status_is_returned() -> Result<()> {
        // given
        init_tracing();
        let cfg = ConfigBuilder::default()
            .list_tests_cmd(Cmd::new("echo", &["some-test\n"]))
            .build()?;
        let tmpdir = tempdir()?;
        let state = noop();
        let index = DefaultTestsIndex {
            cfg,
            curr_tests: RefCell::new(BTreeSet::from(["different-test".into()])),
            state: state.reader(),
        };
        let repo_root = RepoRoot::new(&tmpdir);

        // when
        let res = index.refresh(repo_root)?;

        // then
        assert_eq!(res, IndexStatus::TestsSetChanged);

        Ok(())
    }
}
