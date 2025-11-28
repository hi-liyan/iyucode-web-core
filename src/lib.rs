//! # IyuCode Core Library
//!
//! 生产级别的 Rust Web API 基础设施库，基于 Axum 0.8 框架构建
//!
//! ## 功能特性
//!
//! - **配置管理** - 多环境配置，支持文件和环境变量
//! - **数据库连接** - MySQL 连接池管理
//! - **API 响应** - 统一的 JSON 响应格式（小驼峰命名）
//! - **错误处理** - 统一的错误类型和 HTTP 响应转换
//! - **请求验证** - 自动参数验证
//! - **CORS 支持** - 跨域资源共享配置
//! - **JWT 认证** - 基于 JWT 的认证中间件
//!
//! ## 快速开始
//!
//! ```rust,no_run
//! use axum::{routing::get, Router};
//! use iyucode_core::{ApiResponse, Result};
//!
//! async fn hello() -> Result<ApiResponse<String>> {
//!     Ok(ApiResponse::success("Hello, World!".to_string()))
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let app = Router::new().route("/", get(hello));
//!     
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
//!         .await
//!         .unwrap();
//!     
//!     axum::serve(listener, app).await.unwrap();
//! }
//! ```
//!
//! ## 模块说明
//!
//! - [`config`] - 配置管理，支持多环境和环境变量
//! - [`database`] - 数据库连接池管理
//! - [`error`] - 错误类型定义和处理
//! - [`response`] - API 响应格式化
//! - [`validation`] - 请求参数验证
//! - [`middleware`] - CORS 和 JWT 认证中间件
//!
//! ## 使用示例
//!
//! ### 配置管理
//!
//! ```rust
//! use iyucode_core::config::Settings;
//!
//! let settings = Settings::new().expect("Failed to load configuration");
//! println!("Server: {}:{}", settings.server.host, settings.server.port);
//! ```
//!
//! ### 数据库连接
//!
//! ```rust
//! use iyucode_core::{database::DatabasePool, config::Settings};
//!
//! # async fn example() {
//! let settings = Settings::new().unwrap();
//! let db = DatabasePool::new(&settings.database).await.unwrap();
//! # }
//! ```
//!
//! ### API 响应
//!
//! ```rust
//! use iyucode_core::{ApiResponse, Result};
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct User {
//!     id: i32,
//!     name: String,
//! }
//!
//! async fn get_user() -> Result<ApiResponse<User>> {
//!     let user = User { id: 1, name: "John".to_string() };
//!     Ok(ApiResponse::success(user))
//! }
//! ```
//!
//! ### 请求验证
//!
//! ```rust
//! use iyucode_core::{validation::ValidatedJson, ApiResponse, Result};
//! use serde::Deserialize;
//! use validator::Validate;
//!
//! #[derive(Deserialize, Validate)]
//! struct CreateUser {
//!     #[validate(length(min = 3))]
//!     name: String,
//!     #[validate(email)]
//!     email: String,
//! }
//!
//! async fn create_user(
//!     ValidatedJson(req): ValidatedJson<CreateUser>
//! ) -> Result<ApiResponse<String>> {
//!     Ok(ApiResponse::success("User created".to_string()))
//! }
//! ```

pub mod config;
pub mod database;
pub mod error;
pub mod logging;
pub mod middleware;
pub mod response;
pub mod validation;
pub mod utils;

// ============================================================================
// 公共 API 导出
// ============================================================================

// 错误处理
pub use error::{AppError, Result};

// API 响应
pub use response::{ApiResponse, PageData};

// 请求验证
pub use validation::ValidatedJson;

// 配置
pub use config::Settings;

// 数据库
pub use database::DatabasePool;

// 日志
pub use logging::{init as init_logging, init_default as init_logging_default};

// 中间件
pub use middleware::{setup_cors, AuthMiddleware, Claims, extract_claims};

// 工具
pub use utils::{PasswordHasher, RateLimiter, TimezoneConverter, Validator};
