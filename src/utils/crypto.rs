//! 密码加密和哈希工具
//!
//! 提供基于 Argon2id 的密码加密功能
//!
//! # 示例
//! ```rust
//! use iyucode_core::utils::PasswordHasher;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // 加密密码
//! let hash = PasswordHasher::hash("my_password")?;
//!
//! // 验证密码
//! let is_valid = PasswordHasher::verify("my_password", &hash)?;
//! assert!(is_valid);
//! # Ok(())
//! # }
//! ```

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString},
    Argon2,
};

/// 密码加密错误
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("密码哈希失败: {0}")]
    HashError(String),
    
    #[error("密码验证失败: {0}")]
    VerifyError(String),
}

/// 密码哈希工具
///
/// 使用 Argon2id 算法进行密码加密，这是目前最安全的密码哈希算法之一
///
/// # 特性
/// - 使用随机盐
/// - 抗暴力破解
/// - 抗彩虹表攻击
/// - 抗时序攻击
pub struct PasswordHasher;

impl PasswordHasher {
    /// 使用 Argon2id 算法对密码进行哈希
    ///
    /// # 参数
    /// * `password` - 明文密码
    ///
    /// # 返回
    /// * `Result<String>` - 成功返回密码哈希字符串
    ///
    /// # 错误
    /// 如果哈希过程失败，返回 `CryptoError::HashError`
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::PasswordHasher;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let hash = PasswordHasher::hash("my_secure_password")?;
    /// println!("Password hash: {}", hash);
    /// # Ok(())
    /// # }
    /// ```
    pub fn hash(password: &str) -> Result<String, CryptoError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| {
                tracing::error!("密码哈希失败: {:?}", e);
                CryptoError::HashError(e.to_string())
            })?
            .to_string();

        Ok(password_hash)
    }

    /// 验证密码是否匹配哈希值
    ///
    /// # 参数
    /// * `password` - 明文密码
    /// * `password_hash` - 密码哈希字符串
    ///
    /// # 返回
    /// * `Result<bool>` - 成功返回是否匹配
    ///
    /// # 错误
    /// 如果哈希格式无效，返回 `CryptoError::VerifyError`
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::PasswordHasher;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let hash = PasswordHasher::hash("my_password")?;
    /// 
    /// // 正确的密码
    /// assert!(PasswordHasher::verify("my_password", &hash)?);
    /// 
    /// // 错误的密码
    /// assert!(!PasswordHasher::verify("wrong_password", &hash)?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn verify(password: &str, password_hash: &str) -> Result<bool, CryptoError> {
        let parsed_hash = PasswordHash::new(password_hash)
            .map_err(|e| {
                tracing::error!("解析密码哈希失败: {:?}", e);
                CryptoError::VerifyError(e.to_string())
            })?;

        let argon2 = Argon2::default();
        
        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "test_password123";
        let hash = PasswordHasher::hash(password).unwrap();
        
        assert!(PasswordHasher::verify(password, &hash).unwrap());
        assert!(!PasswordHasher::verify("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_different_hashes() {
        let password = "test_password123";
        let hash1 = PasswordHasher::hash(password).unwrap();
        let hash2 = PasswordHasher::hash(password).unwrap();
        
        // 由于使用了随机盐，相同密码的哈希应该不同
        assert_ne!(hash1, hash2);
        
        // 但都应该能验证成功
        assert!(PasswordHasher::verify(password, &hash1).unwrap());
        assert!(PasswordHasher::verify(password, &hash2).unwrap());
    }

    #[test]
    fn test_empty_password() {
        let hash = PasswordHasher::hash("").unwrap();
        assert!(PasswordHasher::verify("", &hash).unwrap());
    }

    #[test]
    fn test_unicode_password() {
        let password = "密码123🔐";
        let hash = PasswordHasher::hash(password).unwrap();
        assert!(PasswordHasher::verify(password, &hash).unwrap());
    }
}
