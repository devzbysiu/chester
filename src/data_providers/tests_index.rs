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
    tests: TestsSet,
    sr: StateReader,
}

impl DefaultTestsIndex {
    pub fn make(cfg: Config, sr: StateReader) -> TestsIndex {
        Box::new(Self {
            cfg,
            tests: RefCell::new(BTreeSet::new()),
            sr,
        })
    }
}

impl TIndex for DefaultTestsIndex {
    #[instrument(skip(self))]
    fn refresh(&self, repo_root: RepoRoot) -> Result<IndexStatus, IndexErr> {
        if self.sr.tests()? == TestsState::Failure {
            debug!("tests failed previously, they need to be rerun");
            return Ok(IndexStatus::TestsChanged);
        }

        let output = self.cfg.list_tests_cmd.stdout(repo_root)?;
        let tests: Vec<String> = output.lines().map(ToString::to_string).collect();
        let new_tests_set = BTreeSet::from_iter(tests);
        let mut current_tests = self.tests.borrow_mut();
        if current_tests.is_empty() || current_tests.difference(&new_tests_set).count() != 0 {
            debug!("tests changed, or initial set is empty");
            *current_tests = new_tests_set;
            return Ok(IndexStatus::TestsChanged);
        }

        Ok(IndexStatus::TestsNotChanged)
    }
}

// TODO: Add tests

// #[cfg(test)]
// mod test {
//     use super::*;

//     use crate::configuration::config::{Cmd, ConfigBuilder};
//     use crate::configuration::tracing::init_tracing;

//     use anyhow::Result;
//     use cmd_lib::run_cmd;
//     use tempfile::tempdir;

//     #[test]
//     fn when_tests_command_fail_then_failure_status_is_returned() -> Result<()> {
//         // given
//         init_tracing();
//         let cfg = ConfigBuilder::default()
//             .tests_cmd(Cmd::new("cargo", &["test"]))
//             .build()?;
//         let runner = DefaultTestRunner::make(cfg);
//         let invalid_repo_root = RepoRoot::new("/not/existing/path");

//         // when
//         let res = runner.run(invalid_repo_root)?;

//         // then
//         assert_eq!(res, TestsRunStatus::Failure);

//         Ok(())
//     }

//     #[test]
//     fn when_tests_fail_then_failure_status_is_returned() -> Result<()> {
//         // given
//         init_tracing();
//         let tmpdir = tempdir()?;
//         let tmpdir_path = tmpdir.path();
//         run_cmd!(
//             cd $tmpdir_path;
//             cargo new test_project;
//             echo "#[test]\nfn test() { assert!(false); }" >> $tmpdir_path/test_project/src/main.rs
//         )?;
//         let cfg = ConfigBuilder::default()
//             .tests_cmd(Cmd::new("cargo", &["test"]))
//             .build()?;
//         let runner = DefaultTestRunner::make(cfg);
//         let project_path = tmpdir_path.join("test_project");
//         let root = RepoRoot::new(project_path);

//         // when
//         let res = runner.run(root)?;

//         // then
//         assert_eq!(res, TestsRunStatus::Failure);

//         Ok(())
//     }

//     #[test]
//     fn when_tests_succeed_then_success_status_is_returned() -> Result<()> {
//         // given
//         init_tracing();
//         let tmpdir = tempdir()?;
//         let tmpdir_path = tmpdir.path();
//         run_cmd!(cd $tmpdir_path ; cargo new test_project)?;
//         let cfg = ConfigBuilder::default()
//             .tests_cmd(Cmd::new("cargo", &["test"]))
//             .build()?;
//         let runner = DefaultTestRunner::make(cfg);
//         let project_path = tmpdir_path.join("test_project");
//         let root = RepoRoot::new(project_path);

//         // when
//         let res = runner.run(root)?;

//         // then
//         assert_eq!(res, TestsRunStatus::Success);

//         Ok(())
//     }
// }
