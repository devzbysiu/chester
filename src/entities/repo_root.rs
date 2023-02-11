use std::{fmt::Display, path::PathBuf};

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct RepoRoot {
    root: PathBuf,
}

impl Display for RepoRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.root.to_string_lossy())
    }
}
