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

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::tracing::init_tracing;

    #[test]
    fn default_status_is_pending() {
        // given
        init_tracing();
        let status = TestsStatus::default();

        // then
        assert_eq!(status, TestsStatus::Pending);
    }
}
