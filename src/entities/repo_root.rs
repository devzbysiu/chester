use std::fmt::Display;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize)]
pub struct RepoRoot {
    root: PathBuf,
}

impl RepoRoot {
    #[allow(unused)]
    pub fn new<P: AsRef<Path>>(repo_root: P) -> Self {
        let root = repo_root.as_ref().to_path_buf();
        Self { root }
    }

    #[allow(unused)]
    pub fn exists(&self) -> bool {
        self.root.exists()
    }
}

impl Display for RepoRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.root.to_string_lossy())
    }
}

impl AsRef<Path> for RepoRoot {
    fn as_ref(&self) -> &Path {
        self.root.as_path()
    }
}
