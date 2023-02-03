use crate::use_cases::{bus::EventBus, test_runner::TestRunner};

pub struct TestRunnerShell {
    #[allow(unused)]
    bus: EventBus,
}

impl TestRunnerShell {
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    #[allow(clippy::unused_self)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn run(&self, _test_runner: TestRunner) {
        todo!()
    }
}
