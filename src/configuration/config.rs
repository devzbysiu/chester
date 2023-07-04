use crate::data_providers::command::Cmd;
use crate::entities::ignored_path::IgnoredPath;
use crate::result::{CfgErr, CoverageParseErr};

use derive_builder::Builder;

pub fn cfg() -> Result<Config, CfgErr> {
    Ok(ConfigBuilder::default()
        .tests_cmd(tests_cmd())
        .list_tests_cmd(list_tests_cmd())
        .check_cmd(check_cmd())
        .coverage_cmd(coverage_cmd())
        .ignored_paths(vec![IgnoredPath::new("target")?, IgnoredPath::new(".git")?])
        .build()?)
}

fn tests_cmd() -> Cmd {
    Cmd::new("cargo", &["test"])
}

fn list_tests_cmd() -> Cmd {
    Cmd::new("cargo", &["-q", "test", "--", "--list", "--format=terse"])
}

fn check_cmd() -> Cmd {
    Cmd::new("cargo", &["check"])
}

fn coverage_cmd() -> Cmd<f32, CoverageParseErr> {
    Cmd::new(
        "cargo",
        &[
            "tarpaulin",
            "--skip-clean",
            "--target-dir",
            "./tarpaulin-target",
        ],
    )
}

#[derive(Debug, Default, Clone, Builder)]
#[builder(default)]
pub struct Config {
    pub ignored_paths: Vec<IgnoredPath>,
    pub tests_cmd: Cmd,
    pub list_tests_cmd: Cmd,
    pub check_cmd: Cmd,
    pub coverage_cmd: Cmd<f32, CoverageParseErr>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cfg_works() {
        assert!(cfg().is_ok());
    }
}
