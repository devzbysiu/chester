use crate::{entities::ignored_path::IgnoredPath, result::CmdErr};

use derive_builder::Builder;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

#[derive(Debug, Default, Clone, Builder)]
#[builder(default)]
pub struct Config {
    pub ignored_paths: Vec<IgnoredPath>,
    pub tests_cmd: Cmd,
    pub coverage_cmd: Cmd,
}

// TODO: Move this somewhere else?
#[derive(Debug, Default, Clone)]
pub struct Cmd {
    tool: String,
    args: Vec<String>,
}

impl Cmd {
    pub fn new<S: Into<String>>(tool: S, args: &[&str]) -> Self {
        let tool = tool.into();
        let args = args.iter().map(ToString::to_string).collect();
        Self { tool, args }
    }

    pub fn stdout<P: AsRef<Path>>(&self, working_dir: P) -> Result<String, CmdErr> {
        let output = Command::new(&self.tool)
            .args(&self.args)
            .current_dir(working_dir)
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn status<P: AsRef<Path>>(&self, working_dir: P) -> Result<ExitStatus, CmdErr> {
        Ok(Command::new(&self.tool)
            .args(&self.args)
            .current_dir(working_dir)
            .stderr(Stdio::null()) // TODO: Move it to separate log file?
            .status()?)
    }
}
