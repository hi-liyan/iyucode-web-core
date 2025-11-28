//! 日志初始化模块
//!
//! 提供统一的日志系统初始化功能
//!
//! # 功能
//! - 根据配置自动初始化日志
//! - 支持多种日志格式（JSON、Pretty）
//! - 支持多种日志级别
//! - 自动使用本地时区
//!
//! # 示例
//! ```rust,no_run
//! use iyucode_core::{config::Settings, logging};
//!
//! let settings = Settings::new().expect("Failed to load configuration");
//! logging::init(&settings.logging).expect("Failed to initialize logging");
//! ```

use crate::config::LoggingConfig;
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// 日志初始化错误
#[derive(Debug, thiserror::Error)]
pub enum LoggingError {
    #[error("Invalid log level: {0}")]
    InvalidLevel(String),
    
    #[error("Invalid log format: {0}")]
    InvalidFormat(String),
    
    #[error("Failed to set global subscriber: {0}")]
    SetGlobalDefault(String),
}

/// 初始化日志系统
///
/// 根据配置初始化 tracing 日志系统
///
/// # 参数
/// - `config`: 日志配置
///
/// # 错误
/// - 如果日志级别或格式无效，返回错误
/// - 如果无法设置全局订阅者，返回错误
///
/// # 示例
/// ```rust,no_run
/// use iyucode_core::{config::Settings, logging};
///
/// let settings = Settings::new().unwrap();
/// logging::init(&settings.logging).unwrap();
///
/// tracing::info!("Application started");
/// ```
pub fn init(config: &LoggingConfig) -> Result<(), LoggingError> {
    // 构建环境过滤器
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    // 根据格式选择不同的日志层
    match config.format.as_str() {
        "json" => {
            // JSON 格式 - 适合生产环境和日志收集
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .json()
                        .with_timer(fmt::time::OffsetTime::local_rfc_3339()
                            .unwrap_or_else(|_| fmt::time::OffsetTime::new(
                                time::UtcOffset::from_hms(8, 0, 0).unwrap(),
                                time::format_description::well_known::Rfc3339,
                            )))
                )
                .try_init()
                .map_err(|e| LoggingError::SetGlobalDefault(e.to_string()))?;
        }
        "pretty" => {
            // Pretty 格式 - 适合开发环境
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .pretty()
                        .with_timer(fmt::time::OffsetTime::local_rfc_3339()
                            .unwrap_or_else(|_| fmt::time::OffsetTime::new(
                                time::UtcOffset::from_hms(8, 0, 0).unwrap(),
                                time::format_description::well_known::Rfc3339,
                            )))
                )
                .try_init()
                .map_err(|e| LoggingError::SetGlobalDefault(e.to_string()))?;
        }
        "compact" => {
            // Compact 格式 - 紧凑格式
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .compact()
                        .with_timer(fmt::time::OffsetTime::local_rfc_3339()
                            .unwrap_or_else(|_| fmt::time::OffsetTime::new(
                                time::UtcOffset::from_hms(8, 0, 0).unwrap(),
                                time::format_description::well_known::Rfc3339,
                            )))
                )
                .try_init()
                .map_err(|e| LoggingError::SetGlobalDefault(e.to_string()))?;
        }
        _ => {
            return Err(LoggingError::InvalidFormat(config.format.clone()));
        }
    }

    Ok(())
}

/// 使用默认配置初始化日志系统
///
/// 使用以下默认值：
/// - 级别: info
/// - 格式: pretty
///
/// # 示例
/// ```rust,no_run
/// use iyucode_core::logging;
///
/// logging::init_default();
/// tracing::info!("Application started");
/// ```
pub fn init_default() {
    let config = LoggingConfig {
        level: "info".to_string(),
        format: "pretty".to_string(),
    };
    
    init(&config).expect("Failed to initialize logging with default config");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_format() {
        let config = LoggingConfig {
            level: "info".to_string(),
            format: "invalid".to_string(),
        };
        
        let result = init(&config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LoggingError::InvalidFormat(_)));
    }
}
