use crate::entities::repo_root::RepoRoot;
use crate::result::{BusErr, CoverageErr};
use crate::testingtools::{pipe, MutexExt, Spy, Tx};
use crate::use_cases::coverage_runner::{CovRunner, CoverageRunStatus, CoverageRunner};

use anyhow::anyhow;

pub fn tracked(cov_runner: CoverageRunner) -> (CoverageRunnerSpy, CoverageRunner) {
    TrackedCoverageRunner::wrap(cov_runner)
}

pub struct TrackedCoverageRunner {
    cov_runner: CoverageRunner,
    tx: Tx,
}

impl TrackedCoverageRunner {
    fn wrap(cov_runner: CoverageRunner) -> (CoverageRunnerSpy, CoverageRunner) {
        let (tx, spy) = pipe();

        (
            CoverageRunnerSpy::new(spy),
            Box::new(Self { cov_runner, tx }),
        )
    }
}

impl CovRunner for TrackedCoverageRunner {
    fn run(&self, repo_root: RepoRoot) -> Result<CoverageRunStatus, CoverageErr> {
        let res = self.cov_runner.run(repo_root);
        self.tx.signal(());
        res
    }
}

pub struct CoverageRunnerSpy {
    spy: Spy,
}

impl CoverageRunnerSpy {
    fn new(spy: Spy) -> Self {
        Self { spy }
    }

    pub fn run_called(&self) -> bool {
        self.spy.method_called()
    }
}

pub fn working(result: CoverageRunStatus) -> CoverageRunner {
    WorkingCoverageRunner::make(result)
}

pub struct WorkingCoverageRunner {
    result: CoverageRunStatus,
}

impl WorkingCoverageRunner {
    fn make(result: CoverageRunStatus) -> CoverageRunner {
        Box::new(Self { result })
    }
}

impl CovRunner for WorkingCoverageRunner {
    fn run(&self, _repo_root: RepoRoot) -> Result<CoverageRunStatus, CoverageErr> {
        Ok(self.result.clone())
    }
}

pub fn failing() -> CoverageRunner {
    FailingCoverageRunner::make()
}

pub struct FailingCoverageRunner;

impl FailingCoverageRunner {
    fn make() -> CoverageRunner {
        Box::new(Self)
    }
}

impl CovRunner for FailingCoverageRunner {
    fn run(&self, _repo_root: RepoRoot) -> Result<CoverageRunStatus, CoverageErr> {
        Err(CoverageErr::Bus(BusErr::Generic(anyhow!("Failure"))))
    }
}
