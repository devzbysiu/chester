use std::fmt::Display;

use serde::Serialize;

#[derive(Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TestsStatus {
    Pending,
    Failure,
    Success,
}

impl Display for TestsStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TestsStatus::Pending => "pending",
                TestsStatus::Failure => "failure",
                TestsStatus::Success => "success",
            }
        )
    }
}

impl Default for TestsStatus {
    fn default() -> Self {
        Self::Pending
    }
}
