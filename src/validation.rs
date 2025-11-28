//! 请求验证模块
//!
//! 提供自动请求参数验证功能
//!
//! # 功能
//! - 自动 JSON 解析
//! - 自动参数验证
//! - 详细的验证错误信息
//! - 支持嵌套结构验证
//!
//! # 示例
//! ```rust
//! use axum::routing::post;
//! use iyucode_core::{validation::ValidatedJson, ApiResponse, Result};
//! use serde::Deserialize;
//! use validator::Validate;
//!
//! #[derive(Debug, Deserialize, Validate)]
//! struct CreateUserRequest {
//!     #[validate(length(min = 3, max = 50))]
//!     name: String,
//!     #[validate(email)]
//!     email: String,
//! }
//!
//! async fn create_user(
//!     ValidatedJson(payload): ValidatedJson<CreateUserRequest>
//! ) -> Result<ApiResponse<String>> {
//!     // 参数已自动验证，可以安全使用
//!     Ok(ApiResponse::success("User created".to_string()))
//! }
//! ```

use axum::{
    extract::{FromRequest, Request},
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::AppError;

/// 验证的 JSON 提取器
///
/// 自动提取和验证 JSON 请求体
///
/// # 类型参数
/// - `T`: 必须实现 `Deserialize` 和 `Validate` trait
///
/// # 验证流程
/// 1. 从请求体提取 JSON
/// 2. 反序列化为目标类型
/// 3. 执行验证规则
/// 4. 如果验证失败，返回 400 错误和详细的验证错误信息
///
/// # 示例
/// ```rust,ignore
/// use iyucode_core::validation::ValidatedJson;
/// use serde::Deserialize;
/// use validator::Validate;
///
/// #[derive(Deserialize, Validate)]
/// struct LoginRequest {
///     #[validate(email)]
///     email: String,
///     #[validate(length(min = 8))]
///     password: String,
/// }
///
/// async fn login(
///     ValidatedJson(req): ValidatedJson<LoginRequest>
/// ) {
///     // req.email 和 req.password 已经通过验证
/// }
/// ```
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    /// 从请求中提取并验证 JSON 数据
    ///
    /// # 错误
    /// - JSON 解析失败：返回 `AppError::Validation`
    /// - 验证失败：返回 `AppError::Validation` 包含所有验证错误
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // 1. 提取 JSON 数据
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|err| {
                // JSON 解析失败
                tracing::warn!("JSON parsing failed: {}", err);
                AppError::Validation(format!("Invalid JSON: {}", err))
            })?;

        // 2. 执行验证
        value.validate().map_err(|err| {
            // 收集所有验证错误
            let errors = err
                .field_errors()
                .iter()
                .map(|(field, errors)| {
                    // 提取每个字段的所有错误消息
                    let messages: Vec<String> = errors
                        .iter()
                        .filter_map(|e| e.message.as_ref().map(|m| m.to_string()))
                        .collect();
                    
                    // 格式化为 "字段名: 错误1, 错误2"
                    format!("{}: {}", field, messages.join(", "))
                })
                .collect::<Vec<_>>()
                .join("; ");

            tracing::warn!("Validation failed: {}", errors);
            AppError::Validation(errors)
        })?;

        // 3. 返回验证通过的数据
        Ok(ValidatedJson(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use validator::Validate;

    #[derive(Debug, Deserialize, Validate)]
    struct TestRequest {
        #[validate(length(min = 3, max = 10))]
        name: String,
        #[validate(range(min = 1, max = 100))]
        age: i32,
    }

    #[test]
    fn test_validation_success() {
        let req = TestRequest {
            name: "John".to_string(),
            age: 25,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_validation_name_too_short() {
        let req = TestRequest {
            name: "Jo".to_string(),
            age: 25,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_validation_age_out_of_range() {
        let req = TestRequest {
            name: "John".to_string(),
            age: 150,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_validation_multiple_errors() {
        let req = TestRequest {
            name: "J".to_string(),
            age: 0,
        };
        let result = req.validate();
        assert!(result.is_err());
        
        if let Err(errors) = result {
            // 应该有两个字段的错误
            assert_eq!(errors.field_errors().len(), 2);
        }
    }
}

