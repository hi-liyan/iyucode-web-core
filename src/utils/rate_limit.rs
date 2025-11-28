//! 请求频率限制工具
//!
//! 提供基于滑动窗口的频率限制功能
//!
//! # 示例
//! ```rust
//! use iyucode_core::utils::RateLimiter;
//!
//! # async fn example() {
//! // 创建限制器：每分钟最多 10 次请求
//! let limiter = RateLimiter::new(10, 60);
//!
//! // 检查是否允许请求
//! if limiter.check("user_id_or_ip").await {
//!     println!("请求允许");
//! } else {
//!     println!("请求被限流");
//! }
//! # }
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 速率限制记录
#[derive(Debug, Clone)]
struct RateLimit {
    /// 最大请求数
    max_requests: u32,
    /// 时间窗口
    window: Duration,
    /// 当前计数
    current_count: u32,
    /// 窗口开始时间
    window_start: Instant,
}

impl RateLimit {
    fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            current_count: 0,
            window_start: Instant::now(),
        }
    }

    fn check_and_increment(&mut self) -> bool {
        let now = Instant::now();
        
        // 如果超过时间窗口，重置计数
        if now.duration_since(self.window_start) >= self.window {
            self.current_count = 0;
            self.window_start = now;
        }

        // 检查是否超过限制
        if self.current_count >= self.max_requests {
            return false;
        }

        self.current_count += 1;
        true
    }

    fn remaining(&self) -> u32 {
        self.max_requests.saturating_sub(self.current_count)
    }

    fn reset_at(&self) -> Instant {
        self.window_start + self.window
    }
}

/// 速率限制器
///
/// 基于滑动窗口算法的频率限制器，支持多个键（如 IP、用户 ID）的独立限制
///
/// # 特性
/// - 线程安全
/// - 自动清理过期记录
/// - 支持自定义时间窗口
/// - 支持查询剩余配额
#[derive(Clone)]
pub struct RateLimiter {
    /// 限制映射
    limits: Arc<RwLock<HashMap<String, RateLimit>>>,
    /// 默认配置
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    /// 创建新的速率限制器
    ///
    /// # 参数
    /// * `max_requests` - 时间窗口内最大请求数
    /// * `window_secs` - 时间窗口（秒）
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::RateLimiter;
    ///
    /// // 每分钟最多 60 次请求
    /// let limiter = RateLimiter::new(60, 60);
    ///
    /// // 每 5 分钟最多 3 次请求
    /// let strict_limiter = RateLimiter::new(3, 300);
    /// ```
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            limits: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    /// 检查键是否被限流
    ///
    /// # 参数
    /// * `key` - 限制键（如 IP 地址、用户 ID）
    ///
    /// # 返回
    /// * `bool` - true 表示允许请求，false 表示被限流
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::RateLimiter;
    ///
    /// # async fn example() {
    /// let limiter = RateLimiter::new(10, 60);
    ///
    /// if limiter.check("192.168.1.1").await {
    ///     // 处理请求
    /// } else {
    ///     // 返回 429 Too Many Requests
    /// }
    /// # }
    /// ```
    pub async fn check(&self, key: &str) -> bool {
        let mut limits = self.limits.write().await;
        
        let limit = limits
            .entry(key.to_string())
            .or_insert_with(|| RateLimit::new(self.max_requests, self.window));

        limit.check_and_increment()
    }

    /// 获取剩余配额
    ///
    /// # 参数
    /// * `key` - 限制键
    ///
    /// # 返回
    /// * `u32` - 剩余请求次数
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::RateLimiter;
    ///
    /// # async fn example() {
    /// let limiter = RateLimiter::new(10, 60);
    /// let remaining = limiter.remaining("user_123").await;
    /// println!("剩余配额: {}", remaining);
    /// # }
    /// ```
    pub async fn remaining(&self, key: &str) -> u32 {
        let limits = self.limits.read().await;
        
        limits
            .get(key)
            .map(|limit| limit.remaining())
            .unwrap_or(self.max_requests)
    }

    /// 获取限制重置时间
    ///
    /// # 参数
    /// * `key` - 限制键
    ///
    /// # 返回
    /// * `Option<Duration>` - 距离重置的剩余时间
    pub async fn reset_after(&self, key: &str) -> Option<Duration> {
        let limits = self.limits.read().await;
        
        limits.get(key).map(|limit| {
            let now = Instant::now();
            let reset_at = limit.reset_at();
            if reset_at > now {
                reset_at.duration_since(now)
            } else {
                Duration::from_secs(0)
            }
        })
    }

    /// 重置指定键的限制
    ///
    /// # 参数
    /// * `key` - 限制键
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::RateLimiter;
    ///
    /// # async fn example() {
    /// let limiter = RateLimiter::new(10, 60);
    /// 
    /// // 重置某个用户的限制
    /// limiter.reset("user_123").await;
    /// # }
    /// ```
    pub async fn reset(&self, key: &str) {
        let mut limits = self.limits.write().await;
        limits.remove(key);
    }

    /// 清理过期的限制记录
    ///
    /// 建议定期调用此方法以释放内存
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::RateLimiter;
    ///
    /// # async fn example() {
    /// let limiter = RateLimiter::new(10, 60);
    /// 
    /// // 定期清理
    /// limiter.cleanup().await;
    /// # }
    /// ```
    pub async fn cleanup(&self) {
        let mut limits = self.limits.write().await;
        let now = Instant::now();
        
        limits.retain(|_, limit| {
            // 保留最近 2 个窗口内的记录
            now.duration_since(limit.window_start) < limit.window * 2
        });
    }

    /// 获取当前限制的键数量
    pub async fn len(&self) -> usize {
        let limits = self.limits.read().await;
        limits.len()
    }

    /// 检查是否为空
    pub async fn is_empty(&self) -> bool {
        let limits = self.limits.read().await;
        limits.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(3, 1);
        let key = "test_key";

        // 前3次请求应该成功
        assert!(limiter.check(key).await);
        assert!(limiter.check(key).await);
        assert!(limiter.check(key).await);

        // 第4次请求应该被限流
        assert!(!limiter.check(key).await);

        // 等待时间窗口过期
        sleep(Duration::from_secs(2)).await;

        // 新窗口内的请求应该成功
        assert!(limiter.check(key).await);
    }

    #[tokio::test]
    async fn test_different_keys() {
        let limiter = RateLimiter::new(2, 1);

        assert!(limiter.check("key1").await);
        assert!(limiter.check("key1").await);
        assert!(!limiter.check("key1").await);

        // 不同键应该有独立的限制
        assert!(limiter.check("key2").await);
        assert!(limiter.check("key2").await);
    }

    #[tokio::test]
    async fn test_remaining() {
        let limiter = RateLimiter::new(5, 60);
        let key = "test_key";

        assert_eq!(limiter.remaining(key).await, 5);
        
        limiter.check(key).await;
        assert_eq!(limiter.remaining(key).await, 4);
        
        limiter.check(key).await;
        assert_eq!(limiter.remaining(key).await, 3);
    }

    #[tokio::test]
    async fn test_reset() {
        let limiter = RateLimiter::new(2, 60);
        let key = "test_key";

        limiter.check(key).await;
        limiter.check(key).await;
        assert!(!limiter.check(key).await);

        // 重置后应该可以再次请求
        limiter.reset(key).await;
        assert!(limiter.check(key).await);
    }

    #[tokio::test]
    async fn test_cleanup() {
        let limiter = RateLimiter::new(10, 1);

        limiter.check("key1").await;
        limiter.check("key2").await;
        limiter.check("key3").await;

        assert_eq!(limiter.len().await, 3);

        // 等待过期
        sleep(Duration::from_secs(3)).await;

        limiter.cleanup().await;
        assert_eq!(limiter.len().await, 0);
    }
}
