//! 中间件模块
//!
//! 提供 CORS、认证等中间件
//!
//! # 功能
//! - CORS 跨域资源共享
//! - JWT 认证
//! - 请求日志
//!
//! # 示例
//! ```rust
//! use axum::{Router, routing::get, middleware};
//! use iyucode_core::middleware::{setup_cors, AuthMiddleware};
//! use iyucode_core::config::{CorsConfig, JwtConfig};
//!
//! let cors_config = CorsConfig {
//!     allowed_origins: vec!["http://localhost:3001".to_string()],
//!     allowed_methods: vec!["GET".to_string(), "POST".to_string()],
//!     allowed_headers: vec!["Content-Type".to_string()],
//!     max_age: 3600,
//! };
//!
//! let auth = AuthMiddleware::new("your-secret-key".to_string());
//!
//! let app = Router::new()
//!     .route("/public", get(public_handler))
//!     .route("/protected", get(protected_handler))
//!     .layer(middleware::from_fn(move |req, next| {
//!         auth.clone().authenticate(req, next)
//!     }))
//!     .layer(setup_cors(&cors_config));
//! ```

pub mod auth;
pub mod cors;

pub use auth::{AuthMiddleware, Claims, extract_claims};
pub use cors::setup_cors;
