use ntex::http::StatusCode;

use super::AppError;

#[derive(Debug)]
pub enum FirewallError {
    RuleExists,
    RuleNotFound,
    OpeningPort,
    ClosingPort,
}

impl std::error::Error for FirewallError {}

impl std::fmt::Display for FirewallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Firewall error: {}", self.error_message())
    }
}

impl FirewallError {
    pub const fn status_code(&self) -> StatusCode {
        match self {
            FirewallError::RuleExists => StatusCode::CONFLICT,
            FirewallError::RuleNotFound => StatusCode::NOT_FOUND,
            FirewallError::OpeningPort => StatusCode::INTERNAL_SERVER_ERROR,
            FirewallError::ClosingPort => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub const fn error_message(&self) -> &'static str {
        match self {
            FirewallError::RuleExists => "Rule already exists",
            FirewallError::RuleNotFound => "Rule not found",
            FirewallError::OpeningPort => "Failed to open port",
            FirewallError::ClosingPort => "Failed to close port",
        }
    }
}

impl From<FirewallError> for AppError {
    fn from(value: FirewallError) -> Self {
        AppError::Firewall(value)
    }
}
