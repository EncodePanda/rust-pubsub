use serde::{Deserialize, Serialize};

pub const TOPIC_APPLICATIONS: &str = "loan-applications";

pub const SUB_APPLICATIONS: &str = "loan-applications-sub";

#[derive(Debug, Serialize, Deserialize)]
pub struct LoanApplication {
    pub application_id: String,
    pub user_id: String,
    pub amount: u64,
    pub currency: String,
    pub submitted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationStatus {
    pub application_id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interest_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_term_months: Option<u32>,
}
