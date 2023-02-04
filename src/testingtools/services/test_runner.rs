use crate::result::{BusErr, RunnerErr};
use crate::testingtools::{pipe, MutexExt, Spy, Tx};
use crate::use_cases::test_runner::{Runner, TestRunner, TestsStatus};

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

impl Runner for TrackedTestRunner {
    fn run(&self) -> Result<TestsStatus, RunnerErr> {
        let res = self.runner.run();
        self.tx.signal();
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

pub fn working(result: TestsStatus) -> TestRunner {
    WorkingTestRunner::make(result)
}

pub struct WorkingTestRunner {
    result: TestsStatus,
}

impl WorkingTestRunner {
    fn make(result: TestsStatus) -> TestRunner {
        Box::new(Self { result })
    }
}

impl Runner for WorkingTestRunner {
    fn run(&self) -> Result<TestsStatus, RunnerErr> {
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

impl Runner for FailingTestRunner {
    fn run(&self) -> Result<TestsStatus, RunnerErr> {
        Err(RunnerErr::Bus(BusErr::Generic(anyhow!("Failure"))))
    }
}
