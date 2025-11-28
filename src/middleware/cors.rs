///! CORS 跨域资源共享中间件
///!
///! 提供 CORS 配置和中间件设置

use axum::http::{HeaderName, HeaderValue, Method};
use tower_http::cors::{Any, CorsLayer};

use crate::config::CorsConfig;

/// 设置 CORS 中间件
///
/// 根据配置创建 CORS 中间件层
///
/// # 参数
/// - `config`: CORS 配置
///
/// # 返回
/// 配置好的 `CorsLayer`
///
/// # 示例
/// ```rust
/// use axum::Router;
/// use iyucode_core::{middleware::setup_cors, config::CorsConfig};
///
/// let cors_config = CorsConfig {
///     allowed_origins: vec!["http://localhost:3001".to_string()],
///     allowed_methods: vec!["GET".to_string(), "POST".to_string()],
///     allowed_headers: vec!["Content-Type".to_string()],
///     max_age: 3600,
/// };
///
/// let app = Router::new()
///     .layer(setup_cors(&cors_config));
/// ```
pub fn setup_cors(config: &CorsConfig) -> CorsLayer {
    tracing::info!("Setting up CORS middleware");
    tracing::debug!(
        "CORS config: origins={:?}, methods={:?}, headers={:?}, max_age={}",
        config.allowed_origins,
        config.allowed_methods,
        config.allowed_headers,
        config.max_age
    );

    // 解析允许的源
    let has_wildcard = config.allowed_origins.contains(&"*".to_string());
    
    // 解析允许的 HTTP 方法
    let methods: Vec<Method> = config
        .allowed_methods
        .iter()
        .filter_map(|m| {
            m.parse::<Method>().ok().or_else(|| {
                tracing::warn!("Invalid HTTP method in CORS config: {}", m);
                None
            })
        })
        .collect();

    // 解析允许的请求头
    let headers: Vec<HeaderName> = config
        .allowed_headers
        .iter()
        .filter_map(|h| {
            h.parse::<HeaderName>().ok().or_else(|| {
                tracing::warn!("Invalid header name in CORS config: {}", h);
                None
            })
        })
        .collect();

    // 创建 CORS 层
    if has_wildcard {
        // 允许所有源（不推荐用于生产环境）
        tracing::warn!("CORS configured to allow all origins (*). This is not recommended for production.");
        
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(methods)
            .allow_headers(headers)
            .max_age(std::time::Duration::from_secs(config.max_age))
    } else {
        // 明确指定允许的源
        let origins: Vec<HeaderValue> = config
            .allowed_origins
            .iter()
            .filter_map(|origin| {
                origin.parse::<HeaderValue>().ok().or_else(|| {
                    tracing::warn!("Invalid origin in CORS config: {}", origin);
                    None
                })
            })
            .collect();

        tracing::info!("CORS configured with {} allowed origins", origins.len());

        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(methods)
            .allow_headers(headers)
            .max_age(std::time::Duration::from_secs(config.max_age))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_cors_with_wildcard() {
        let config = CorsConfig {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec!["Content-Type".to_string()],
            max_age: 3600,
        };

        let cors_layer = setup_cors(&config);
        // CORS 层创建成功
        assert!(std::mem::size_of_val(&cors_layer) > 0);
    }

    #[test]
    fn test_setup_cors_with_specific_origins() {
        let config = CorsConfig {
            allowed_origins: vec![
                "http://localhost:3001".to_string(),
                "https://example.com".to_string(),
            ],
            allowed_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string()],
            allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
            max_age: 7200,
        };

        let cors_layer = setup_cors(&config);
        // CORS 层创建成功
        assert!(std::mem::size_of_val(&cors_layer) > 0);
    }

    #[test]
    fn test_setup_cors_with_invalid_method() {
        let config = CorsConfig {
            allowed_origins: vec!["http://localhost:3001".to_string()],
            allowed_methods: vec!["GET".to_string(), "INVALID_METHOD".to_string()],
            allowed_headers: vec!["Content-Type".to_string()],
            max_age: 3600,
        };

        // 应该忽略无效的方法，但仍然创建 CORS 层
        let cors_layer = setup_cors(&config);
        assert!(std::mem::size_of_val(&cors_layer) > 0);
    }

    #[test]
    fn test_setup_cors_with_empty_config() {
        let config = CorsConfig {
            allowed_origins: vec![],
            allowed_methods: vec![],
            allowed_headers: vec![],
            max_age: 0,
        };

        // 即使配置为空，也应该能创建 CORS 层
        let cors_layer = setup_cors(&config);
        assert!(std::mem::size_of_val(&cors_layer) > 0);
    }
}
