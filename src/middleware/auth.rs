///! JWT 认证中间件
///!
///! 提供基于 JWT 的认证功能

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// JWT Claims 结构
///
/// 包含 JWT token 中的标准声明和自定义字段
///
/// # 字段
/// - `sub`: Subject（用户 ID）
/// - `exp`: Expiration time（过期时间，Unix 时间戳）
/// - `iat`: Issued at（签发时间，Unix 时间戳）
/// - `role`: 用户角色（自定义字段）
///
/// # 示例
/// ```rust
/// use iyucode_core::middleware::Claims;
///
/// let claims = Claims {
///     sub: "user123".to_string(),
///     exp: 1234567890,
///     iat: 1234567800,
///     role: "admin".to_string(),
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject - 用户 ID
    pub sub: String,
    /// Expiration time - 过期时间（Unix 时间戳）
    pub exp: i64,
    /// Issued at - 签发时间（Unix 时间戳）
    pub iat: i64,
    /// User role - 用户角色
    pub role: String,
}

/// JWT 认证中间件
///
/// 验证请求中的 JWT token，并将用户信息注入到请求扩展中
///
/// # 示例
/// ```rust
/// use axum::{Router, routing::get, middleware};
/// use iyucode_core::middleware::AuthMiddleware;
///
/// let auth = AuthMiddleware::new("your-secret-key".to_string());
///
/// let app = Router::new()
///     .route("/protected", get(protected_handler))
///     .layer(middleware::from_fn(move |req, next| {
///         auth.clone().authenticate(req, next)
///     }));
/// ```
#[derive(Clone)]
pub struct AuthMiddleware {
    /// JWT 签名密钥
    secret: String,
}

impl AuthMiddleware {
    /// 创建认证中间件
    ///
    /// # 参数
    /// - `secret`: JWT 签名密钥（建议至少 256 位）
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::middleware::AuthMiddleware;
    ///
    /// let auth = AuthMiddleware::new("your-secret-key".to_string());
    /// ```
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    /// 认证处理函数
    ///
    /// 从请求头中提取 JWT token，验证并将 Claims 注入到请求扩展中
    ///
    /// # 参数
    /// - `req`: HTTP 请求
    /// - `next`: 下一个中间件或处理器
    ///
    /// # 错误
    /// - Token 缺失：返回 401
    /// - Token 无效：返回 401
    /// - Token 过期：返回 401
    ///
    /// # 示例
    /// ```rust
    /// use axum::middleware;
    /// use iyucode_core::middleware::AuthMiddleware;
    ///
    /// let auth = AuthMiddleware::new("secret".to_string());
    ///
    /// // 在路由中使用
    /// let layer = middleware::from_fn(move |req, next| {
    ///     auth.clone().authenticate(req, next)
    /// });
    /// ```
    pub async fn authenticate(
        &self,
        mut req: Request,
        next: Next,
    ) -> Result<Response, AppError> {
        // 1. 从 Authorization header 提取 token
        let token = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| {
                tracing::warn!("Missing or invalid Authorization header");
                AppError::Authentication("Missing or invalid token".to_string())
            })?;

        // 2. 验证 token
        let claims = self.verify_token(token)?;

        tracing::debug!("User authenticated: {} (role: {})", claims.sub, claims.role);

        // 3. 将 claims 注入到请求扩展中
        req.extensions_mut().insert(claims);

        // 4. 继续处理请求
        Ok(next.run(req).await)
    }

    /// 验证 JWT token
    ///
    /// # 参数
    /// - `token`: JWT token 字符串
    ///
    /// # 返回
    /// 解析后的 Claims
    ///
    /// # 错误
    /// - Token 格式无效
    /// - 签名验证失败
    /// - Token 已过期
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::middleware::AuthMiddleware;
    ///
    /// let auth = AuthMiddleware::new("secret".to_string());
    /// match auth.verify_token("eyJ...") {
    ///     Ok(claims) => println!("User: {}", claims.sub),
    ///     Err(e) => eprintln!("Invalid token: {}", e),
    /// }
    /// ```
    fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        // 创建解码密钥
        let decoding_key = DecodingKey::from_secret(self.secret.as_bytes());
        
        // 创建验证配置
        let mut validation = Validation::new(Algorithm::HS256);
        // 验证过期时间
        validation.validate_exp = true;

        // 解码并验证 token
        decode::<Claims>(token, &decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| {
                tracing::warn!("Token verification failed: {}", e);
                AppError::Authentication(format!("Invalid token: {}", e))
            })
    }

    /// 生成 JWT token
    ///
    /// # 参数
    /// - `user_id`: 用户 ID
    /// - `role`: 用户角色
    /// - `expiration`: 过期时间（秒）
    ///
    /// # 返回
    /// JWT token 字符串
    ///
    /// # 错误
    /// Token 生成失败
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::middleware::AuthMiddleware;
    ///
    /// let auth = AuthMiddleware::new("secret".to_string());
    /// let token = auth.generate_token("user123", "admin", 86400).unwrap();
    /// println!("Token: {}", token);
    /// ```
    pub fn generate_token(
        &self,
        user_id: &str,
        role: &str,
        expiration: i64,
    ) -> Result<String, AppError> {
        // 获取当前时间戳
        let now = chrono::Utc::now().timestamp();

        // 创建 claims
        let claims = Claims {
            sub: user_id.to_string(),
            exp: now + expiration,
            iat: now,
            role: role.to_string(),
        };

        // 创建编码密钥
        let encoding_key = EncodingKey::from_secret(self.secret.as_bytes());

        // 生成 token
        encode(&Header::default(), &claims, &encoding_key).map_err(|e| {
            tracing::error!("Failed to generate token: {}", e);
            AppError::Internal(format!("Failed to generate token: {}", e))
        })
    }
}

/// 从请求中提取 Claims
///
/// 在认证中间件之后的处理器中使用，获取当前用户信息
///
/// # 参数
/// - `req`: HTTP 请求
///
/// # 返回
/// 当前用户的 Claims
///
/// # 错误
/// 如果请求中没有 Claims（未经过认证中间件），返回错误
///
/// # 示例
/// ```rust
/// use axum::extract::Request;
/// use iyucode_core::middleware::extract_claims;
///
/// async fn protected_handler(req: Request) {
///     match extract_claims(&req) {
///         Ok(claims) => println!("User: {}", claims.sub),
///         Err(e) => eprintln!("Not authenticated: {}", e),
///     }
/// }
/// ```
pub fn extract_claims(req: &Request) -> Result<Claims, AppError> {
    req.extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| {
            tracing::warn!("No authentication data found in request");
            AppError::Authentication("No authentication data found".to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "test-secret-key-at-least-32-characters-long";

    #[test]
    fn test_generate_and_verify_token() {
        let auth = AuthMiddleware::new(TEST_SECRET.to_string());
        
        // 生成 token
        let token = auth.generate_token("user123", "admin", 3600).unwrap();
        assert!(!token.is_empty());
        
        // 验证 token
        let claims = auth.verify_token(&token).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.role, "admin");
    }

    #[test]
    fn test_verify_invalid_token() {
        let auth = AuthMiddleware::new(TEST_SECRET.to_string());
        
        // 无效的 token
        let result = auth.verify_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    #[ignore] // 需要等待时间，在 CI 中可能不稳定
    fn test_verify_expired_token() {
        let auth = AuthMiddleware::new(TEST_SECRET.to_string());
        
        // 生成一个很快过期的 token（1 秒后过期）
        let token = auth.generate_token("user123", "admin", 1).unwrap();
        
        // 等待 token 过期
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        // 验证应该失败
        let result = auth.verify_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_token_with_wrong_secret() {
        let auth1 = AuthMiddleware::new(TEST_SECRET.to_string());
        let auth2 = AuthMiddleware::new("different-secret-key-32-chars-long".to_string());
        
        // 用 auth1 生成 token
        let token = auth1.generate_token("user123", "admin", 3600).unwrap();
        
        // 用 auth2 验证应该失败
        let result = auth2.verify_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_claims_serialization() {
        let claims = Claims {
            sub: "user123".to_string(),
            exp: 1234567890,
            iat: 1234567800,
            role: "admin".to_string(),
        };
        
        // 序列化
        let json = serde_json::to_string(&claims).unwrap();
        assert!(json.contains("user123"));
        assert!(json.contains("admin"));
        
        // 反序列化
        let deserialized: Claims = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.sub, claims.sub);
        assert_eq!(deserialized.role, claims.role);
    }
}
