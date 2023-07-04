use crate::result::CmdErr;
use crate::use_cases::output_parser::Parser;

use debug_ignore::DebugIgnore;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

#[derive(Debug, Default, Clone)]
pub struct Cmd<T = (), E = ()> {
    tool: String,
    args: Vec<String>,
    output_parser: DebugIgnore<Option<Parser<T, E>>>,
}

impl<T, E> Cmd<T, E> {
    pub fn new<S: Into<String>>(tool: S, args: &[&str]) -> Self {
        let tool = tool.into();
        let args = args.iter().map(ToString::to_string).collect();
        Self {
            tool,
            args,
            output_parser: None.into(),
        }
    }

    #[allow(unused)] // TODO: Remove this
    pub fn with_parser<S: Into<String>>(tool: S, args: &[&str], parser: Parser<T, E>) -> Self {
        let tool = tool.into();
        let args = args.iter().map(ToString::to_string).collect();
        Self {
            tool,
            args,
            output_parser: Some(parser).into(),
        }
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

    pub fn parser(&self) -> Option<&Parser<T, E>> {
        self.output_parser.0.as_ref()
    }
}
