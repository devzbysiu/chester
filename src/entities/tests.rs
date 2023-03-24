use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TestsState {
    Pending,
    Failure,
    Success,
}

impl Display for TestsState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TestsState::Pending => "pending",
                TestsState::Failure => "failure",
                TestsState::Success => "success",
            }
        )
    }
}

impl Default for TestsState {
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

        // when
        let status = TestsState::default();

        // then
        assert_eq!(status, TestsState::Pending);
    }

    #[test]
    fn tests_status_has_display_trait_implemented() {
        // given
        init_tracing();

        // then
        assert_eq!(TestsState::Pending.to_string(), "pending");
        assert_eq!(TestsState::Failure.to_string(), "failure");
        assert_eq!(TestsState::Success.to_string(), "success");
    }
}
