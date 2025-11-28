//! 数据库连接模块
//!
//! 提供 MySQL 数据库连接池管理
//!
//! # 功能
//! - 连接池管理
//! - 自动重连
//! - 健康检查
//! - 连接池统计
//!
//! # 示例
//! ```rust
//! use iyucode_core::{database::DatabasePool, config::DatabaseConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = DatabaseConfig {
//!         url: "mysql://root:password@localhost:3306/mydb".to_string(),
//!         max_connections: 10,
//!         min_connections: 2,
//!         connect_timeout: 30,
//!         idle_timeout: 600,
//!     };
//!     
//!     let db_pool = DatabasePool::new(&config).await.unwrap();
//!     
//!     // 健康检查
//!     db_pool.health_check().await.unwrap();
//!     
//!     // 使用连接池
//!     let pool = db_pool.pool();
//!     // ... 执行数据库操作
//! }
//! ```

pub mod pool;
pub mod pools;

pub use pool::DatabasePool;
pub use pools::DatabasePools;
