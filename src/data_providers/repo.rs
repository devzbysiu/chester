use crate::use_cases::repo::{Repo, RepoRead, RepoWrite, Repository};

pub struct DefaultRepo;

impl DefaultRepo {
    pub fn make() -> Repo {
        Box::new(Self)
    }
}

impl Repository for DefaultRepo {
    fn read(&self) -> RepoRead {
        todo!()
    }

    fn write(&self) -> RepoWrite {
        todo!()
    }
}
