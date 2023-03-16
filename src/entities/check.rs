use serde::Serialize;
use std::fmt::Display;

#[derive(Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum CheckState {
    Pending,
    Failure,
    Success,
}

impl Display for CheckState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CheckState::Pending => "pending",
                CheckState::Failure => "failure",
                CheckState::Success => "success",
            }
        )
    }
}

impl Default for CheckState {
    fn default() -> Self {
        Self::Pending
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     use crate::configuration::tracing::init_tracing;

//     #[test]
//     fn default_status_is_pending() {
//         // given
//         init_tracing();

//         // when
//         let status = TestsState::default();

//         // then
//         assert_eq!(status, TestsState::Pending);
//     }

//     #[test]
//     fn tests_status_has_display_trait_implemented() {
//         // given
//         init_tracing();

//         // then
//         assert_eq!(TestsState::Pending.to_string(), "pending");
//         assert_eq!(TestsState::Failure.to_string(), "failure");
//         assert_eq!(TestsState::Success.to_string(), "success");
//     }
// }
