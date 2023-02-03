use crate::use_cases::bus::EventBus;
use crate::use_cases::repo::RepoWrite;

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
    pub fn run(&self, _repo_write: RepoWrite) {
        todo!()
    }
}
