use crate::result::RunnerErr;

pub type TestRunner = Box<dyn Runner>;

pub trait Runner: Send {
    fn run_all(&self) -> Result<TestsStatus, RunnerErr>;
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum TestsStatus {
    Success,
    Failure,
}
