//! 输入验证工具
//!
//! 提供常用的输入验证功能
//!
//! # 示例
//! ```rust
//! use iyucode_core::utils::Validator;
//!
//! // 验证邮箱
//! assert!(Validator::email("test@example.com").is_ok());
//!
//! // 验证密码
//! assert!(Validator::password("MyPass123").is_ok());
//!
//! // 验证用户名
//! assert!(Validator::username("john_doe").is_ok());
//! ```

use once_cell::sync::Lazy;
use regex::Regex;

/// 邮箱验证正则表达式
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
});

/// 用户名验证正则表达式（字母、数字、下划线）
static USERNAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9_]+$").unwrap()
});

/// URL 验证正则表达式
static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap()
});

/// 验证错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// 通用验证器
///
/// 提供常用的输入验证功能
pub struct Validator;

impl Validator {
    /// 验证邮箱格式
    ///
    /// # 参数
    /// * `email` - 邮箱地址
    ///
    /// # 返回
    /// * `Result<(), ValidationError>` - 验证成功返回 Ok，失败返回错误
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::Validator;
    ///
    /// assert!(Validator::email("test@example.com").is_ok());
    /// assert!(Validator::email("invalid").is_err());
    /// ```
    pub fn email(email: &str) -> Result<(), ValidationError> {
        if email.is_empty() {
            return Err(ValidationError::new("email", "邮箱不能为空"));
        }

        if email.len() > 255 {
            return Err(ValidationError::new("email", "邮箱长度不能超过255个字符"));
        }

        if !EMAIL_REGEX.is_match(email) {
            return Err(ValidationError::new("email", "邮箱格式无效"));
        }

        Ok(())
    }

    /// 验证密码强度
    ///
    /// 默认要求：
    /// - 最少 8 个字符
    /// - 最多 128 个字符
    /// - 至少包含一个字母
    /// - 至少包含一个数字
    ///
    /// # 参数
    /// * `password` - 密码
    ///
    /// # 返回
    /// * `Result<(), ValidationError>` - 验证成功返回 Ok，失败返回错误
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::Validator;
    ///
    /// assert!(Validator::password("MyPass123").is_ok());
    /// assert!(Validator::password("short").is_err());
    /// ```
    pub fn password(password: &str) -> Result<(), ValidationError> {
        Self::password_with_policy(password, 8, 128, true, true, false)
    }

    /// 使用自定义策略验证密码
    ///
    /// # 参数
    /// * `password` - 密码
    /// * `min_length` - 最小长度
    /// * `max_length` - 最大长度
    /// * `require_letter` - 是否要求字母
    /// * `require_number` - 是否要求数字
    /// * `require_special` - 是否要求特殊字符
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::Validator;
    ///
    /// // 要求 12 个字符，包含字母、数字和特殊字符
    /// let result = Validator::password_with_policy(
    ///     "MyP@ss123456",
    ///     12,
    ///     128,
    ///     true,
    ///     true,
    ///     true
    /// );
    /// assert!(result.is_ok());
    /// ```
    pub fn password_with_policy(
        password: &str,
        min_length: usize,
        max_length: usize,
        require_letter: bool,
        require_number: bool,
        require_special: bool,
    ) -> Result<(), ValidationError> {
        if password.len() < min_length {
            return Err(ValidationError::new(
                "password",
                format!("密码长度至少为{}个字符", min_length),
            ));
        }

        if password.len() > max_length {
            return Err(ValidationError::new(
                "password",
                format!("密码长度不能超过{}个字符", max_length),
            ));
        }

        if require_letter && !password.chars().any(|c| c.is_alphabetic()) {
            return Err(ValidationError::new("password", "密码必须包含至少一个字母"));
        }

        if require_number && !password.chars().any(|c| c.is_numeric()) {
            return Err(ValidationError::new("password", "密码必须包含至少一个数字"));
        }

        if require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err(ValidationError::new("password", "密码必须包含至少一个特殊字符"));
        }

        Ok(())
    }

    /// 验证用户名
    ///
    /// 要求：
    /// - 长度在 3-20 个字符之间
    /// - 只能包含字母、数字和下划线
    ///
    /// # 参数
    /// * `username` - 用户名
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::Validator;
    ///
    /// assert!(Validator::username("john_doe").is_ok());
    /// assert!(Validator::username("ab").is_err());
    /// ```
    pub fn username(username: &str) -> Result<(), ValidationError> {
        Self::username_with_length(username, 3, 20)
    }

    /// 使用自定义长度验证用户名
    ///
    /// # 参数
    /// * `username` - 用户名
    /// * `min_length` - 最小长度
    /// * `max_length` - 最大长度
    pub fn username_with_length(
        username: &str,
        min_length: usize,
        max_length: usize,
    ) -> Result<(), ValidationError> {
        if username.len() < min_length {
            return Err(ValidationError::new(
                "username",
                format!("用户名至少为{}个字符", min_length),
            ));
        }

        if username.len() > max_length {
            return Err(ValidationError::new(
                "username",
                format!("用户名不能超过{}个字符", max_length),
            ));
        }

        if !USERNAME_REGEX.is_match(username) {
            return Err(ValidationError::new(
                "username",
                "用户名只能包含字母、数字和下划线",
            ));
        }

        Ok(())
    }

    /// 验证字符串长度
    ///
    /// # 参数
    /// * `field` - 字段名
    /// * `value` - 值
    /// * `min` - 最小长度
    /// * `max` - 最大长度
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::Validator;
    ///
    /// assert!(Validator::length("name", "John", 2, 50).is_ok());
    /// assert!(Validator::length("name", "J", 2, 50).is_err());
    /// ```
    pub fn length(
        field: &str,
        value: &str,
        min: usize,
        max: usize,
    ) -> Result<(), ValidationError> {
        if value.len() < min {
            return Err(ValidationError::new(
                field,
                format!("长度至少为{}个字符", min),
            ));
        }

        if value.len() > max {
            return Err(ValidationError::new(
                field,
                format!("长度不能超过{}个字符", max),
            ));
        }

        Ok(())
    }

    /// 验证数值范围
    ///
    /// # 参数
    /// * `field` - 字段名
    /// * `value` - 值
    /// * `min` - 最小值
    /// * `max` - 最大值
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::Validator;
    ///
    /// assert!(Validator::range("age", 25, 1, 120).is_ok());
    /// assert!(Validator::range("age", 0, 1, 120).is_err());
    /// ```
    pub fn range<T: PartialOrd + std::fmt::Display>(
        field: &str,
        value: T,
        min: T,
        max: T,
    ) -> Result<(), ValidationError> {
        if value < min {
            return Err(ValidationError::new(
                field,
                format!("值不能小于{}", min),
            ));
        }

        if value > max {
            return Err(ValidationError::new(
                field,
                format!("值不能大于{}", max),
            ));
        }

        Ok(())
    }

    /// 验证 URL 格式
    ///
    /// # 参数
    /// * `url` - URL 字符串
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::Validator;
    ///
    /// assert!(Validator::url("https://example.com").is_ok());
    /// assert!(Validator::url("not-a-url").is_err());
    /// ```
    pub fn url(url: &str) -> Result<(), ValidationError> {
        if url.is_empty() {
            return Err(ValidationError::new("url", "URL不能为空"));
        }

        if !URL_REGEX.is_match(url) {
            return Err(ValidationError::new("url", "URL格式无效"));
        }

        Ok(())
    }

    /// 验证是否为空
    ///
    /// # 参数
    /// * `field` - 字段名
    /// * `value` - 值
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::Validator;
    ///
    /// assert!(Validator::required("name", "John").is_ok());
    /// assert!(Validator::required("name", "").is_err());
    /// ```
    pub fn required(field: &str, value: &str) -> Result<(), ValidationError> {
        if value.trim().is_empty() {
            return Err(ValidationError::new(field, "不能为空"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email() {
        assert!(Validator::email("test@example.com").is_ok());
        assert!(Validator::email("user.name+tag@example.co.uk").is_ok());
        assert!(Validator::email("invalid").is_err());
        assert!(Validator::email("@example.com").is_err());
        assert!(Validator::email("test@").is_err());
        assert!(Validator::email("").is_err());
    }

    #[test]
    fn test_password() {
        assert!(Validator::password("password123").is_ok());
        assert!(Validator::password("Test1234").is_ok());
        assert!(Validator::password("12345678a").is_ok());
        
        assert!(Validator::password("short1").is_err());
        assert!(Validator::password("12345678").is_err());
        assert!(Validator::password("abcdefgh").is_err());
    }

    #[test]
    fn test_password_with_policy() {
        // 要求特殊字符
        assert!(Validator::password_with_policy(
            "MyP@ss123",
            8,
            128,
            true,
            true,
            true
        ).is_ok());
        
        assert!(Validator::password_with_policy(
            "MyPass123",
            8,
            128,
            true,
            true,
            true
        ).is_err());
    }

    #[test]
    fn test_username() {
        assert!(Validator::username("john_doe").is_ok());
        assert!(Validator::username("user123").is_ok());
        
        assert!(Validator::username("ab").is_err());
        assert!(Validator::username("this_is_a_very_long_username").is_err());
        assert!(Validator::username("user@123").is_err());
    }

    #[test]
    fn test_length() {
        assert!(Validator::length("name", "John", 2, 50).is_ok());
        assert!(Validator::length("name", "J", 2, 50).is_err());
        assert!(Validator::length("name", "A".repeat(51).as_str(), 2, 50).is_err());
    }

    #[test]
    fn test_range() {
        assert!(Validator::range("age", 25, 1, 120).is_ok());
        assert!(Validator::range("age", 0, 1, 120).is_err());
        assert!(Validator::range("age", 150, 1, 120).is_err());
    }

    #[test]
    fn test_url() {
        assert!(Validator::url("https://example.com").is_ok());
        assert!(Validator::url("http://example.com/path").is_ok());
        assert!(Validator::url("not-a-url").is_err());
        assert!(Validator::url("").is_err());
    }

    #[test]
    fn test_required() {
        assert!(Validator::required("name", "John").is_ok());
        assert!(Validator::required("name", "").is_err());
        assert!(Validator::required("name", "   ").is_err());
    }
}
