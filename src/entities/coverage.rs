use std::fmt::Display;

use serde::Serialize;

#[derive(Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum CoverageState {
    Pending,
    Failure,
    Success(u8),
}

impl Display for CoverageState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CoverageState::Pending => "pending".to_string(),
                CoverageState::Failure => "failure".to_string(),
                CoverageState::Success(val) => format!("{val}"),
            }
        )
    }
}

impl Default for CoverageState {
    fn default() -> Self {
        Self::Pending
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::tracing::init_tracing;

    #[test]
    fn default_coverage_status_is_pending() {
        // given
        init_tracing();

        // when
        let status = CoverageState::default();

        // then
        assert_eq!(status, CoverageState::Pending);
    }

    #[test]
    fn tests_status_has_display_trait_implemented() {
        // given
        init_tracing();

        // then
        assert_eq!(CoverageState::Pending.to_string(), "pending");
        assert_eq!(CoverageState::Failure.to_string(), "failure");
        assert_eq!(CoverageState::Success(10).to_string(), "10");
    }
}
