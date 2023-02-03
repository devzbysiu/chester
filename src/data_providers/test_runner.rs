use crate::use_cases::test_runner::{Runner, TestRunner};

pub struct DefaultTestRunner;

impl DefaultTestRunner {
    pub fn make() -> TestRunner {
        Box::new(Self)
    }
}

impl Runner for DefaultTestRunner {
    fn run(&self) {
        todo!()
    }
}
