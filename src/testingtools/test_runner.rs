use crate::entities::repo_root::RepoRoot;
use crate::result::{BusErr, RunnerErr};
use crate::testingtools::{pipe, MutexExt, Spy, Tx};
use crate::use_cases::test_runner::{TRunner, TestRunner, TestsRunStatus};

use anyhow::anyhow;

pub fn tracked(runner: TestRunner) -> (TestRunnerSpy, TestRunner) {
    TrackedTestRunner::wrap(runner)
}

pub struct TrackedTestRunner {
    runner: TestRunner,
    tx: Tx,
}

impl TrackedTestRunner {
    fn wrap(runner: TestRunner) -> (TestRunnerSpy, TestRunner) {
        let (tx, spy) = pipe();

        (TestRunnerSpy::new(spy), Box::new(Self { runner, tx }))
    }
}

impl TRunner for TrackedTestRunner {
    fn run(&self, repo_root: RepoRoot) -> Result<TestsRunStatus, RunnerErr> {
        let res = self.runner.run(repo_root);
        self.tx.signal(());
        res
    }
}

pub struct TestRunnerSpy {
    spy: Spy,
}

impl TestRunnerSpy {
    fn new(spy: Spy) -> Self {
        Self { spy }
    }

    pub fn run_called(&self) -> bool {
        self.spy.method_called()
    }
}

pub fn working(result: TestsRunStatus) -> TestRunner {
    WorkingTestRunner::make(result)
}

pub struct WorkingTestRunner {
    result: TestsRunStatus,
}

impl WorkingTestRunner {
    fn make(result: TestsRunStatus) -> TestRunner {
        Box::new(Self { result })
    }
}

impl TRunner for WorkingTestRunner {
    fn run(&self, _repo_root: RepoRoot) -> Result<TestsRunStatus, RunnerErr> {
        Ok(self.result.clone())
    }
}

pub fn failing() -> TestRunner {
    FailingTestRunner::make()
}

pub struct FailingTestRunner;

impl FailingTestRunner {
    fn make() -> TestRunner {
        Box::new(Self)
    }
}

impl TRunner for FailingTestRunner {
    fn run(&self, _repo_root: RepoRoot) -> Result<TestsRunStatus, RunnerErr> {
        Err(RunnerErr::Bus(BusErr::Generic(anyhow!("Failure"))))
    }
}
