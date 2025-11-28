//! API 响应格式化模块
//!
//! 提供统一的 API 响应格式（使用小驼峰命名）
//!
//! # 功能
//! - 统一的 JSON 响应格式
//! - 支持成功和错误响应
//! - 支持分页数据
//! - 自动转换为 HTTP 响应
//!
//! # 响应格式
//! ```json
//! {
//!   "code": 200,
//!   "message": "Success",
//!   "data": { ... }
//! }
//! ```
//!
//! # 示例
//! ```rust
//! use iyucode_core::{ApiResponse, Result};
//!
//! async fn handler() -> Result<ApiResponse<String>> {
//!     Ok(ApiResponse::success("Hello, World!".to_string()))
//! }
//! ```

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// 统一的 API 响应结构
///
/// 所有 API 端点都应该返回此格式的响应
///
/// # 字段
/// - `code`: HTTP 状态码
/// - `message`: 响应消息
/// - `data`: 响应数据（可选）
///
/// # JSON 输出
/// 使用小驼峰命名（camelCase）
///
/// # 示例
/// ```rust
/// use iyucode_core::ApiResponse;
///
/// // 成功响应
/// let response = ApiResponse::success("Hello");
///
/// // 带自定义消息的成功响应
/// let response = ApiResponse::success_with_message("Created successfully", user);
///
/// // 错误响应
/// let response = ApiResponse::<()>::error(404, "Not found");
/// ```
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    /// HTTP 状态码
    pub code: u16,
    /// 响应消息
    pub message: String,
    /// 响应数据（如果为 None 则不序列化此字段）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

/// 分页响应数据结构
///
/// 用于返回分页列表数据
///
/// # 字段
/// - `items`: 当前页的数据列表
/// - `total`: 总记录数
/// - `page`: 当前页码（从 1 开始）
/// - `pageSize`: 每页记录数
/// - `totalPages`: 总页数
///
/// # JSON 输出
/// 使用小驼峰命名（camelCase）
///
/// # 示例
/// ```rust
/// use iyucode_core::response::PageData;
///
/// let page_data = PageData::new(
///     vec!["item1", "item2"],
///     100,  // total
///     1,    // page
///     20,   // page_size
/// );
/// ```
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageData<T> {
    /// 当前页的数据列表
    pub items: Vec<T>,
    /// 总记录数
    pub total: i64,
    /// 当前页码（从 1 开始）
    pub page: i64,
    /// 每页记录数
    pub page_size: i64,
    /// 总页数
    pub total_pages: i64,
}

impl<T: Serialize> ApiResponse<T> {
    /// 创建成功响应
    ///
    /// 使用默认消息 "Success" 和 200 状态码
    ///
    /// # 参数
    /// - `data`: 响应数据
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::ApiResponse;
    ///
    /// let response = ApiResponse::success("Hello, World!");
    /// ```
    pub fn success(data: T) -> Self {
        Self {
            code: 200,
            message: "Success".to_string(),
            data: Some(data),
        }
    }

    /// 创建带自定义消息的成功响应
    ///
    /// 使用 200 状态码和自定义消息
    ///
    /// # 参数
    /// - `message`: 自定义消息
    /// - `data`: 响应数据
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::ApiResponse;
    ///
    /// let response = ApiResponse::success_with_message(
    ///     "User created successfully",
    ///     user_data
    /// );
    /// ```
    pub fn success_with_message(message: impl Into<String>, data: T) -> Self {
        Self {
            code: 200,
            message: message.into(),
            data: Some(data),
        }
    }

    /// 创建自定义响应
    ///
    /// 允许完全自定义状态码、消息和数据
    ///
    /// # 参数
    /// - `code`: HTTP 状态码
    /// - `message`: 响应消息
    /// - `data`: 响应数据（可选）
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::ApiResponse;
    ///
    /// let response = ApiResponse::new(201, "Created", Some(user_data));
    /// ```
    pub fn new(code: u16, message: impl Into<String>, data: Option<T>) -> Self {
        Self {
            code,
            message: message.into(),
            data,
        }
    }
}

impl ApiResponse<()> {
    /// 创建错误响应
    ///
    /// 不包含数据字段
    ///
    /// # 参数
    /// - `code`: HTTP 状态码
    /// - `message`: 错误消息
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::ApiResponse;
    ///
    /// let response = ApiResponse::<()>::error(404, "User not found");
    /// ```
    pub fn error(code: u16, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    /// 将 ApiResponse 转换为 HTTP 响应
    ///
    /// 自动设置正确的 HTTP 状态码和 JSON 响应体
    fn into_response(self) -> Response {
        // 从 code 字段获取 HTTP 状态码，如果无效则使用 500
        let status = StatusCode::from_u16(self.code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        
        // 返回 JSON 响应
        (status, Json(self)).into_response()
    }
}

impl<T> PageData<T> {
    /// 创建分页数据
    ///
    /// 自动计算总页数
    ///
    /// # 参数
    /// - `items`: 当前页的数据列表
    /// - `total`: 总记录数
    /// - `page`: 当前页码（从 1 开始）
    /// - `page_size`: 每页记录数
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::response::PageData;
    ///
    /// let users = vec![user1, user2, user3];
    /// let page_data = PageData::new(users, 100, 1, 20);
    /// assert_eq!(page_data.total_pages, 5);
    /// ```
    pub fn new(items: Vec<T>, total: i64, page: i64, page_size: i64) -> Self {
        // 计算总页数（向上取整）
        let total_pages = if page_size > 0 {
            (total as f64 / page_size as f64).ceil() as i64
        } else {
            0
        };

        Self {
            items,
            total,
            page,
            page_size,
            total_pages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_success_response() {
        let response = ApiResponse::success("test data");
        assert_eq!(response.code, 200);
        assert_eq!(response.message, "Success");
        assert_eq!(response.data, Some("test data"));
    }

    #[test]
    fn test_success_with_message() {
        let response = ApiResponse::success_with_message("Custom message", 42);
        assert_eq!(response.code, 200);
        assert_eq!(response.message, "Custom message");
        assert_eq!(response.data, Some(42));
    }

    #[test]
    fn test_error_response() {
        let response = ApiResponse::<()>::error(404, "Not found");
        assert_eq!(response.code, 404);
        assert_eq!(response.message, "Not found");
        assert_eq!(response.data, None);
    }

    #[test]
    fn test_response_serialization() {
        let response = ApiResponse::success("test");
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"code\":200"));
        assert!(json.contains("\"message\":\"Success\""));
        assert!(json.contains("\"data\":\"test\""));
    }

    #[test]
    fn test_camel_case_serialization() {
        #[derive(Serialize)]
        struct TestData {
            user_name: String,
        }
        
        let data = TestData {
            user_name: "John".to_string(),
        };
        let response = ApiResponse::success(data);
        let json = serde_json::to_string(&response).unwrap();
        
        // 验证响应字段使用小驼峰
        assert!(json.contains("\"code\""));
        assert!(json.contains("\"message\""));
        assert!(json.contains("\"data\""));
    }

    #[test]
    fn test_page_data_creation() {
        let items = vec![1, 2, 3];
        let page_data = PageData::new(items, 100, 1, 20);
        
        assert_eq!(page_data.total, 100);
        assert_eq!(page_data.page, 1);
        assert_eq!(page_data.page_size, 20);
        assert_eq!(page_data.total_pages, 5);
    }

    #[test]
    fn test_page_data_serialization() {
        let items = vec!["item1", "item2"];
        let page_data = PageData::new(items, 50, 2, 10);
        let json = serde_json::to_string(&page_data).unwrap();
        
        // 验证使用小驼峰命名
        assert!(json.contains("\"pageSize\":10"));
        assert!(json.contains("\"totalPages\":5"));
    }

    #[test]
    fn test_page_data_total_pages_calculation() {
        // 测试向上取整
        let page_data = PageData::new(vec![1], 25, 1, 10);
        assert_eq!(page_data.total_pages, 3);
        
        // 测试整除
        let page_data = PageData::new(vec![1], 30, 1, 10);
        assert_eq!(page_data.total_pages, 3);
        
        // 测试空数据
        let page_data = PageData::new(Vec::<i32>::new(), 0, 1, 10);
        assert_eq!(page_data.total_pages, 0);
    }
}
