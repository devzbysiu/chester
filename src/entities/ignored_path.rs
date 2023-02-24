use crate::result::IgnoredPathErr;

use regex::Regex;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct IgnoredPath {
    re: Regex,
}

impl IgnoredPath {
    #[allow(unused)]
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, IgnoredPathErr> {
        let re = Regex::new(&to_string(&path))?;
        Ok(Self { re })
    }

    pub fn matched_by<P: AsRef<Path>>(&self, other: P) -> bool {
        self.re.is_match(&to_string(other))
    }
}

fn to_string<P: AsRef<Path>>(path: P) -> String {
    path.as_ref().to_string_lossy().to_string()
}
