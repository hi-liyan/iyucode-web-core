//! 配置管理模块
//!
//! 提供多环境配置支持，支持从配置文件和环境变量加载配置
//!
//! # 功能
//! - 多环境配置（development, testing, production）
//! - 环境变量覆盖
//! - 配置验证
//! - 类型安全的配置结构
//!
//! # 示例
//! ```rust
//! use iyucode_core::config::Settings;
//!
//! let settings = Settings::new().expect("Failed to load configuration");
//! println!("Server: {}:{}", settings.server.host, settings.server.port);
//! ```

pub mod settings;

pub use settings::{
    CorsConfig, DatabaseConfig, DatabaseConnection, JwtConfig, LoggingConfig, ServerConfig,
    Settings, SmtpConfig,
};
