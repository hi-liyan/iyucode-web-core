///! 多数据库连接池管理模块
///!
///! 提供多数据库连接池管理，支持命名数据库的隔离访问

use sqlx::{MySql, Pool};
use std::collections::HashMap;

use crate::config::{DatabaseConfig, DatabaseConnection};
use crate::error::{AppError, Result};

/// 多数据库连接池管理器
///
/// 管理多个命名的数据库连接池，支持：
/// - 单数据库模式（向后兼容）
/// - 多数据库模式（命名数据库）
///
/// # 示例
///
/// 单数据库模式：
/// ```rust
/// use iyucode_core::{database::DatabasePools, config::Settings};
///
/// #[tokio::main]
/// async fn main() {
///     let settings = Settings::new().unwrap();
///     let pools = DatabasePools::new(&settings.database).await.unwrap();
///     
///     // 获取默认数据库
///     let pool = pools.get("default").unwrap();
/// }
/// ```
///
/// 多数据库模式：
/// ```rust
/// use iyucode_core::{database::DatabasePools, config::Settings};
///
/// #[tokio::main]
/// async fn main() {
///     let settings = Settings::new().unwrap();
///     let pools = DatabasePools::new(&settings.database).await.unwrap();
///     
///     // 获取平台数据库
///     let platform_pool = pools.get("platform").unwrap();
///     
///     // 获取租户数据库
///     let tenant_pool = pools.get("tenant").unwrap();
/// }
/// ```
#[derive(Clone)]
pub struct DatabasePools {
    /// 命名的数据库连接池集合
    pools: HashMap<String, Pool<MySql>>,
}

impl DatabasePools {
    /// 创建多数据库连接池
    ///
    /// 根据配置创建所有数据库的连接池
    ///
    /// # 参数
    /// - `config`: 数据库配置（支持单数据库或多数据库）
    ///
    /// # 错误
    /// - 数据库连接失败
    /// - 配置参数无效
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        tracing::info!("Creating database connection pools...");
        
        let mut pools = HashMap::new();
        
        // 遍历所有数据库配置并创建连接池
        for (name, conn_config) in config.all() {
            tracing::info!("Creating connection pool for database: {}", name);
            
            let pool = Self::create_pool(name, conn_config).await?;
            pools.insert(name.to_string(), pool);
            
            tracing::info!("Database '{}' connection pool created successfully", name);
        }
        
        tracing::info!(
            "All database connection pools created successfully (total: {})",
            pools.len()
        );
        
        Ok(Self { pools })
    }
    
    /// 创建单个数据库连接池
    async fn create_pool(name: &str, config: &DatabaseConnection) -> Result<Pool<MySql>> {
        tracing::debug!(
            "Database '{}' config: max_connections={}, min_connections={}, connect_timeout={}s, idle_timeout={}s",
            name,
            config.max_connections,
            config.min_connections,
            config.connect_timeout,
            config.idle_timeout
        );
        
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.connect_timeout))
            .idle_timeout(std::time::Duration::from_secs(config.idle_timeout))
            .connect(&config.url)
            .await
            .map_err(|e| {
                tracing::error!("Failed to connect to database '{}': {}", name, e);
                AppError::Database(format!("Failed to connect to database '{}': {}", name, e))
            })?;
        
        Ok(pool)
    }
    
    /// 获取指定名称的数据库连接池
    ///
    /// # 参数
    /// - `name`: 数据库名称（如 "platform", "tenant", "default"）
    ///
    /// # 返回
    /// - Some(&Pool<MySql>): 如果数据库存在
    /// - None: 如果数据库不存在
    pub fn get(&self, name: &str) -> Option<&Pool<MySql>> {
        self.pools.get(name)
    }
    
    /// 执行所有数据库的健康检查
    ///
    /// 通过执行简单的查询来验证所有数据库连接是否正常
    ///
    /// # 错误
    /// 如果任何数据库连接失败或查询执行失败，返回错误
    pub async fn health_check(&self) -> Result<()> {
        tracing::debug!("Performing health check on all databases...");
        
        for (name, pool) in &self.pools {
            tracing::debug!("Checking database '{}'...", name);
            
            sqlx::query("SELECT 1")
                .execute(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Health check failed for database '{}': {}", name, e);
                    AppError::Database(format!("Health check failed for '{}': {}", name, e))
                })?;
            
            tracing::debug!("Database '{}' health check passed", name);
        }
        
        tracing::debug!("All databases health check passed");
        Ok(())
    }
    
    /// 优雅关闭所有数据库连接池
    ///
    /// 关闭所有数据库连接，释放资源
    pub async fn close_all(&self) {
        tracing::info!("Closing all database connection pools...");
        
        for (name, pool) in &self.pools {
            tracing::info!("Closing database '{}'...", name);
            pool.close().await;
            tracing::info!("Database '{}' closed", name);
        }
        
        tracing::info!("All database connection pools closed");
    }
    
    /// 获取所有数据库连接池的统计信息
    ///
    /// # 返回
    /// HashMap<数据库名称, (当前连接数, 空闲连接数)>
    pub fn stats(&self) -> HashMap<String, (u32, u32)> {
        self.pools
            .iter()
            .map(|(name, pool)| {
                let size = pool.size();
                let idle = pool.num_idle() as u32;
                (name.clone(), (size, idle))
            })
            .collect()
    }
    
    /// 获取数据库数量
    pub fn count(&self) -> usize {
        self.pools.len()
    }
    
    /// 检查指定数据库是否存在
    pub fn has(&self, name: &str) -> bool {
        self.pools.contains_key(name)
    }
    
    /// 获取所有数据库名称
    pub fn names(&self) -> Vec<String> {
        self.pools.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_database_pools_api() {
        // API 使用示例测试
        // 实际测试需要数据库连接
    }
}
