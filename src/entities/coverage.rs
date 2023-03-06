use serde::{Serialize, Serializer};
use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub enum CoverageState {
    Pending,
    Failure,
    Success(f32),
}

impl Serialize for CoverageState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
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
        assert_eq!(CoverageState::Success(10.1).to_string(), "10.1");
    }
}
