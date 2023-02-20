use crate::entities::ignored_path::IgnoredPath;

#[derive(Debug, Default, Clone)]
pub struct Config {
    pub ignored_paths: Vec<IgnoredPath>,
}
