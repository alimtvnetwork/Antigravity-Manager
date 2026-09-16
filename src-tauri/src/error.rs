use serde::Serialize;
use thiserror::Error;

/// Standard application error representing domain failures across the Rust backend.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Network error: {0}")]
    Network(String, Option<u16>),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("OAuth error: {0}")]
    OAuth(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Account error: {0}")]
    Account(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        let status = err.status().map(|s| s.as_u16());
        AppError::Network(err.to_string(), status)
    }
}

impl From<rquest::Error> for AppError {
    fn from(err: rquest::Error) -> Self {
        let status = err.status().map(|s| s.as_u16());
        AppError::Network(err.to_string(), status)
    }
}

/// Universal Response Envelope status block.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct StatusBlock {
    pub is_success: bool,
    pub is_failed: bool,
    pub code: u16,
    pub message: String,
}

/// Universal Response Envelope errors block.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ErrorsBlock {
    pub backend_message: String,
    pub backend: Vec<String>,
}

/// Universal Response Envelope attributes block.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AttributesBlock {
    pub requested_at: Option<String>,
    pub has_any_errors: bool,
}

/// Universal error payload serialized to frontend callers.
#[derive(Serialize, Clone, Debug)]
pub struct AppErrorPayload {
    #[serde(rename = "Status")]
    pub status: StatusBlock,
    #[serde(rename = "Errors")]
    pub errors: ErrorsBlock,
    #[serde(rename = "Attributes")]
    pub attributes: AttributesBlock,

    pub code: &'static str,
    pub level: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend_stack_trace: Option<String>,
}

impl AppError {
    /// Maps error variant to standard diagnostic error code.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Network(_, Some(429)) => "E1003",
            AppError::Network(_, Some(_)) => "E1002",
            AppError::Network(_, None) => "E1001",
            AppError::OAuth(_) => "E2001",
            AppError::Database(_) => "E3001",
            AppError::Io(_) => "E4001",
            AppError::Config(_) => "E5001",
            AppError::Account(_) => "E6001",
            AppError::Tauri(_) => "E8001",
            AppError::Unknown(_) => "E9001",
        }
    }

    /// Maps error variant to corresponding HTTP/system status code.
    pub fn status_code(&self) -> u16 {
        match self {
            AppError::Network(_, Some(status)) => *status,
            AppError::Network(_, None) => 502,
            AppError::OAuth(_) => 401,
            AppError::Config(_) | AppError::Account(_) => 400,
            AppError::Database(_) | AppError::Io(_) | AppError::Tauri(_) | AppError::Unknown(_) => {
                500
            }
        }
    }

    /// Severity level of the error ("error", "warn", "info").
    pub fn level(&self) -> &'static str {
        "error"
    }

    /// Constructs the structured error payload.
    pub fn to_payload(&self) -> AppErrorPayload {
        let code_str = self.code();
        let status_num = self.status_code();
        let msg = self.to_string();
        let now_iso = chrono::Utc::now().to_rfc3339();

        AppErrorPayload {
            status: StatusBlock {
                is_success: false,
                is_failed: true,
                code: status_num,
                message: msg.clone(),
            },
            errors: ErrorsBlock {
                backend_message: msg.clone(),
                backend: vec![format!("{} [{}]", msg, code_str)],
            },
            attributes: AttributesBlock {
                requested_at: None,
                has_any_errors: true,
            },
            code: code_str,
            level: self.level(),
            message: msg,
            details: None,
            timestamp: now_iso,
            backend_stack_trace: None,
        }
    }
}

// Implement Serialize so Tauri commands return structured JSON payload
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_payload().serialize(serializer)
    }
}

// Implement alias for Result to simplify usage
pub type AppResult<T> = Result<T, AppError>;
