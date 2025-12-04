use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClobError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {message}")]
    Json { message: String },

    #[error("API error: {message}")]
    Api { message: String },

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Signing error: {message}")]
    Signing { message: String },

    #[error("Authentication required: {0}")]
    AuthRequired(String),
}

pub type Result<T> = std::result::Result<T, ClobError>;
