//! 错误处理模块
//!
//! 提供统一的错误类型和处理，自动转换为标准 HTTP 响应
//!
//! # 功能
//! - 统一的错误类型定义
//! - 自动转换为 HTTP 响应
//! - 区分开发和生产环境的错误详情
//! - 支持错误链追踪
//!
//! # 示例
//! ```rust
//! use iyucode_core::{AppError, Result};
//!
//! fn do_something() -> Result<String> {
//!     Err(AppError::NotFound("Resource not found".to_string()))
//! }
//! ```

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Result 类型别名，使用 AppError 作为错误类型
pub type Result<T> = std::result::Result<T, AppError>;

/// 应用错误类型
///
/// 定义了所有可能的错误场景，每种错误会映射到对应的 HTTP 状态码
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// 数据库错误 (500 Internal Server Error)
    ///
    /// 包括连接失败、查询错误、事务错误等
    #[error("Database error: {0}")]
    Database(String),

    /// 验证错误 (400 Bad Request)
    ///
    /// 请求参数不符合验证规则
    #[error("Validation error: {0}")]
    Validation(String),

    /// 认证错误 (401 Unauthorized)
    ///
    /// Token 缺失、无效或过期
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// 授权错误 (403 Forbidden)
    ///
    /// 用户无权访问资源
    #[error("Authorization error: {0}")]
    Authorization(String),

    /// 资源未找到 (404 Not Found)
    ///
    /// 请求的资源不存在
    #[error("Not found: {0}")]
    NotFound(String),

    /// 冲突错误 (409 Conflict)
    ///
    /// 资源状态冲突，如重复创建
    #[error("Conflict: {0}")]
    Conflict(String),

    /// 错误请求 (400 Bad Request)
    ///
    /// 请求格式错误或参数无效
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// 内部服务器错误 (500 Internal Server Error)
    ///
    /// 未预期的错误
    #[error("Internal server error: {0}")]
    Internal(String),

    /// 请求过多 (429 Too Many Requests)
    ///
    /// 超过速率限制
    #[error("Too many requests: {0}")]
    TooManyRequests(String),
}

impl IntoResponse for AppError {
    /// 将错误转换为 HTTP 响应
    ///
    /// 根据错误类型设置对应的 HTTP 状态码，并返回统一格式的 JSON 响应
    ///
    /// # 安全性
    /// - Database 和 Internal 错误会记录详细日志，但只返回通用错误消息
    /// - 其他错误返回具体的错误信息
    fn into_response(self) -> Response {
        // 根据错误类型确定 HTTP 状态码和错误消息
        let (status, error_message) = match self {
            // 数据库错误：记录详细日志，返回通用消息
            AppError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            // 验证错误：返回具体的验证失败信息
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            // 认证错误：返回认证失败信息
            AppError::Authentication(msg) => (StatusCode::UNAUTHORIZED, msg),
            // 授权错误：返回权限不足信息
            AppError::Authorization(msg) => (StatusCode::FORBIDDEN, msg),
            // 未找到错误：返回资源不存在信息
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            // 冲突错误：返回冲突详情
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            // 错误请求：返回请求错误信息
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            // 内部错误：记录详细日志，返回通用消息
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            // 请求过多错误：返回速率限制信息
            AppError::TooManyRequests(msg) => (StatusCode::TOO_MANY_REQUESTS, msg),
        };

        // 构建统一格式的 JSON 响应
        let body = Json(json!({
            "code": status.as_u16(),
            "message": error_message,
            "data": serde_json::Value::Null
        }));

        (status, body).into_response()
    }
}

// ============================================================================
// 外部错误类型转换
// ============================================================================

/// 从 SQLx 错误转换
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

/// 从配置错误转换
impl From<config::ConfigError> for AppError {
    fn from(err: config::ConfigError) -> Self {
        AppError::Internal(format!("Configuration error: {}", err))
    }
}

/// 从 JWT 错误转换
impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::Authentication(format!("JWT error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AppError::NotFound("User not found".to_string());
        assert_eq!(err.to_string(), "Not found: User not found");
    }

    #[test]
    fn test_validation_error() {
        let err = AppError::Validation("Email is invalid".to_string());
        assert!(err.to_string().contains("Validation error"));
    }

    #[test]
    fn test_authentication_error() {
        let err = AppError::Authentication("Token expired".to_string());
        assert!(err.to_string().contains("Authentication error"));
    }
}

