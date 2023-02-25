use crate::entities::ignored_path::IgnoredPath;

#[derive(Debug, Default, Clone)]
pub struct Config {
    pub ignored_paths: Vec<IgnoredPath>,
    pub cmd: Cmd,
}

#[derive(Debug, Default, Clone)]
pub struct Cmd {
    pub tool: String,
    pub args: String,
}

impl Cmd {
    pub fn new<S: Into<String>>(tool: S, args: S) -> Self {
        let tool = tool.into();
        let args = args.into();
        Self { tool, args }
    }
}
