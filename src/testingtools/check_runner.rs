use crate::entities::repo_root::RepoRoot;
use crate::result::{BusErr, CheckErr};
use crate::testingtools::{pipe, MutexExt, Spy, Tx};
use crate::use_cases::check_runner::{CRunner, CheckRunStatus, CheckRunner};

use anyhow::anyhow;

pub fn tracked(runner: CheckRunner) -> (CheckRunnerSpy, CheckRunner) {
    TrackedCheckRunner::wrap(runner)
}

pub struct TrackedCheckRunner {
    runner: CheckRunner,
    tx: Tx,
}

impl TrackedCheckRunner {
    fn wrap(runner: CheckRunner) -> (CheckRunnerSpy, CheckRunner) {
        let (tx, spy) = pipe();

        (CheckRunnerSpy::new(spy), Box::new(Self { runner, tx }))
    }
}

impl CRunner for TrackedCheckRunner {
    fn run(&self, repo_root: RepoRoot) -> Result<CheckRunStatus, CheckErr> {
        let res = self.runner.run(repo_root);
        self.tx.signal(());
        res
    }
}

pub struct CheckRunnerSpy {
    spy: Spy,
}

impl CheckRunnerSpy {
    fn new(spy: Spy) -> Self {
        Self { spy }
    }

    pub fn run_called(&self) -> bool {
        self.spy.method_called()
    }
}

pub fn working(result: CheckRunStatus) -> CheckRunner {
    WorkingTestRunner::make(result)
}

pub struct WorkingTestRunner {
    result: CheckRunStatus,
}

impl WorkingTestRunner {
    fn make(result: CheckRunStatus) -> CheckRunner {
        Box::new(Self { result })
    }
}

impl CRunner for WorkingTestRunner {
    fn run(&self, _repo_root: RepoRoot) -> Result<CheckRunStatus, CheckErr> {
        Ok(self.result.clone())
    }
}

pub fn failing() -> CheckRunner {
    FailingTestRunner::make()
}

pub struct FailingTestRunner;

impl FailingTestRunner {
    fn make() -> CheckRunner {
        Box::new(Self)
    }
}

impl CRunner for FailingTestRunner {
    fn run(&self, _repo_root: RepoRoot) -> Result<CheckRunStatus, CheckErr> {
        Err(CheckErr::Bus(BusErr::Generic(anyhow!("Failure"))))
    }
}
