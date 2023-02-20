use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct IgnoredPath {
    path: PathBuf,
}

impl IgnoredPath {
    #[allow(unused)]
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().to_path_buf();
        Self { path }
    }
}

impl AsRef<Path> for IgnoredPath {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}
