//! 租户错误类型定义
//!
//! 定义了租户管理相关的错误类型

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TenantError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Tenant not found")]
    NotFound,

    #[error("Tenant already exists")]
    AlreadyExists,

    #[error("Key expired")]
    KeyExpired,

    #[error("Key does not exist")]
    KeyNotExist,

    #[error("Parse error: {0}")]
    ParseError(String),
}

impl From<sqlx::Error> for TenantError {
    fn from(err: sqlx::Error) -> Self {
        TenantError::DatabaseError(err.to_string())
    }
}
