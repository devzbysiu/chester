use serde::Serialize;

#[derive(Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TestsStatus {
    Pending,
    Failure,
    Success,
}

impl ToString for TestsStatus {
    fn to_string(&self) -> String {
        match self {
            TestsStatus::Pending => "pending".into(),
            TestsStatus::Failure => "failure".into(),
            TestsStatus::Success => "success".into(),
        }
    }
}
