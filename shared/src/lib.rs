use serde::{Deserialize, Serialize};

pub const PROJECT_ID: &str = "local-project";

pub const TOPIC_APPLICATIONS: &str = "loan-applications";
pub const TOPIC_DECISIONS: &str = "loan-decisions";

pub const SUB_APPLICATIONS: &str = "loan-applications-sub";
pub const SUB_DECISIONS: &str = "loan-decisions-sub";

#[derive(Debug, Serialize, Deserialize)]
pub struct LoanApplication {
    pub application_id: String,
    pub user_id: String,
    pub amount: u64,
    pub currency: String,
    pub submitted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DecisionStatus {
    Approved,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoanDecision {
    pub application_id: String,
    pub status: DecisionStatus,
    pub interest_rate: Option<f64>,
    pub max_term_months: Option<u32>,
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
