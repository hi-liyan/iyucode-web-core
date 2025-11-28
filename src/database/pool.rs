///! 数据库连接池模块
///!
///! 提供 MySQL 数据库连接池管理

use sqlx::{MySql, Pool};

use crate::error::{AppError, Result};

/// 数据库连接池包装器
///
/// 封装 SQLx 的连接池，提供类型安全的数据库访问
///
/// # 示例
/// ```rust
/// use iyucode_core::{database::DatabasePool, config::DatabaseConfig};
///
/// #[tokio::main]
/// async fn main() {
///     let config = DatabaseConfig {
///         url: "mysql://root:password@localhost:3306/mydb".to_string(),
///         max_connections: 10,
///         min_connections: 2,
///         connect_timeout: 30,
///         idle_timeout: 600,
///     };
///     
///     let db_pool = DatabasePool::new(&config).await.unwrap();
///     
///     // 使用连接池
///     let pool = db_pool.pool();
///     sqlx::query!("SELECT 1").fetch_one(pool).await.unwrap();
/// }
/// ```
#[derive(Clone)]
pub struct DatabasePool {
    /// SQLx MySQL 连接池
    pool: Pool<MySql>,
}

impl DatabasePool {
    /// 创建数据库连接池
    ///
    /// 根据配置创建并初始化数据库连接池
    ///
    /// # 参数
    /// - `config`: 数据库连接配置
    ///
    /// # 错误
    /// - 数据库连接失败
    /// - 配置参数无效
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::{database::DatabasePool, config::DatabaseConnection};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = DatabaseConnection {
    ///         url: "mysql://root:password@localhost:3306/mydb".to_string(),
    ///         max_connections: 10,
    ///         min_connections: 2,
    ///         connect_timeout: 30,
    ///         idle_timeout: 600,
    ///     };
    ///     
    ///     match DatabasePool::new(&config).await {
    ///         Ok(pool) => println!("Database connected"),
    ///         Err(e) => eprintln!("Failed to connect: {}", e),
    ///     }
    /// }
    /// ```
    pub async fn new(config: &crate::config::DatabaseConnection) -> Result<Self> {
        tracing::info!("Creating database connection pool...");
        tracing::debug!(
            "Database config: max_connections={}, min_connections={}, connect_timeout={}s, idle_timeout={}s",
            config.max_connections,
            config.min_connections,
            config.connect_timeout,
            config.idle_timeout
        );

        // 创建连接池
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            // 设置最大连接数
            .max_connections(config.max_connections)
            // 设置最小连接数
            .min_connections(config.min_connections)
            // 设置连接超时时间
            .acquire_timeout(std::time::Duration::from_secs(config.connect_timeout))
            // 设置空闲连接超时时间
            .idle_timeout(std::time::Duration::from_secs(config.idle_timeout))
            // 连接到数据库
            .connect(&config.url)
            .await
            .map_err(|e| {
                tracing::error!("Failed to connect to database: {}", e);
                AppError::Database(format!("Failed to connect to database: {}", e))
            })?;

        tracing::info!("Database connection pool created successfully");

        Ok(Self { pool })
    }

    /// 获取连接池引用
    ///
    /// 返回底层 SQLx 连接池的引用，用于执行数据库操作
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::database::DatabasePool;
    ///
    /// async fn query_users(db: &DatabasePool) {
    ///     let pool = db.pool();
    ///     let users = sqlx::query!("SELECT * FROM users")
    ///         .fetch_all(pool)
    ///         .await
    ///         .unwrap();
    /// }
    /// ```
    pub fn pool(&self) -> &Pool<MySql> {
        &self.pool
    }

    /// 执行健康检查
    ///
    /// 通过执行简单的查询来验证数据库连接是否正常
    ///
    /// # 错误
    /// 如果数据库连接失败或查询执行失败，返回错误
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::database::DatabasePool;
    ///
    /// async fn check_database(db: &DatabasePool) {
    ///     match db.health_check().await {
    ///         Ok(_) => println!("Database is healthy"),
    ///         Err(e) => eprintln!("Database health check failed: {}", e),
    ///     }
    /// }
    /// ```
    pub async fn health_check(&self) -> Result<()> {
        tracing::debug!("Performing database health check...");

        // 执行简单的查询
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("Database health check failed: {}", e);
                AppError::Database(format!("Health check failed: {}", e))
            })?;

        tracing::debug!("Database health check passed");
        Ok(())
    }

    /// 优雅关闭连接池
    ///
    /// 关闭所有数据库连接，释放资源
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::database::DatabasePool;
    ///
    /// async fn shutdown(db: DatabasePool) {
    ///     db.close().await;
    ///     println!("Database connections closed");
    /// }
    /// ```
    pub async fn close(&self) {
        tracing::info!("Closing database connection pool...");
        self.pool.close().await;
        tracing::info!("Database connection pool closed");
    }

    /// 获取连接池统计信息
    ///
    /// 返回当前连接池的状态信息
    ///
    /// # 返回
    /// - `size`: 当前连接数
    /// - `idle`: 空闲连接数
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::database::DatabasePool;
    ///
    /// async fn print_pool_stats(db: &DatabasePool) {
    ///     let (size, idle) = db.stats();
    ///     println!("Pool size: {}, Idle: {}", size, idle);
    /// }
    /// ```
    pub fn stats(&self) -> (u32, u32) {
        let size = self.pool.size();
        let idle = self.pool.num_idle() as u32;
        (size, idle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：这些测试需要实际的 MySQL 数据库
    // 在 CI/CD 环境中应该使用测试数据库或 mock

    #[tokio::test]
    #[ignore] // 需要实际数据库，默认忽略
    async fn test_database_pool_creation() {
        let config = crate::config::DatabaseConnection {
            url: "mysql://root:password@localhost:3306/test".to_string(),
            max_connections: 5,
            min_connections: 1,
            connect_timeout: 10,
            idle_timeout: 300,
        };

        let result = DatabasePool::new(&config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // 需要实际数据库，默认忽略
    async fn test_health_check() {
        let config = crate::config::DatabaseConnection {
            url: "mysql://root:password@localhost:3306/test".to_string(),
            max_connections: 5,
            min_connections: 1,
            connect_timeout: 10,
            idle_timeout: 300,
        };

        let pool = DatabasePool::new(&config).await.unwrap();
        let result = pool.health_check().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // 需要实际数据库，默认忽略
    async fn test_pool_stats() {
        let config = crate::config::DatabaseConnection {
            url: "mysql://root:password@localhost:3306/test".to_string(),
            max_connections: 5,
            min_connections: 2,
            connect_timeout: 10,
            idle_timeout: 300,
        };

        let pool = DatabasePool::new(&config).await.unwrap();
        let (size, _idle) = pool.stats();
        
        // 至少应该有 min_connections 个连接
        assert!(size >= 2);
    }
}
