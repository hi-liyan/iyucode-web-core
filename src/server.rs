//! # 服务器启动器模块
//!
//! 提供统一的服务器启动和配置管理功能

use crate::{
    config::Settings,
    database::DatabasePools,
    init_logging,
    setup_cors,
    error::Result,
};
use axum::Router;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

/// 服务器构建器
///
/// 用于配置和启动 Axum 服务器
pub struct ServerBuilder {
    settings: Settings,
    db_pools: Option<DatabasePools>,
    cors: Option<CorsLayer>,
}

impl ServerBuilder {
    /// 创建新的服务器构建器
    ///
    /// 从默认配置目录 `config/` 加载配置
    ///
    /// # 错误
    ///
    /// 如果配置加载失败，返回错误
    pub fn new() -> Result<Self> {
        let settings = Settings::new()?;
        Ok(Self {
            settings,
            db_pools: None,
            cors: None,
        })
    }

    /// 从指定配置目录创建服务器构建器
    ///
    /// # 参数
    ///
    /// * `config_dir` - 配置文件目录路径（如 "server/config"）
    ///
    /// # 错误
    ///
    /// 如果配置加载失败，返回错误
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use iyucode_core::ServerBuilder;
    ///
    /// let builder = ServerBuilder::from_config_dir("server/config")
    ///     .expect("加载配置失败");
    /// ```
    pub fn from_config_dir(config_dir: &str) -> Result<Self> {
        let settings = Settings::from_dir(config_dir)?;
        Ok(Self {
            settings,
            db_pools: None,
            cors: None,
        })
    }

    /// 从指定的配置创建服务器构建器
    pub fn with_settings(settings: Settings) -> Self {
        Self {
            settings,
            db_pools: None,
            cors: None,
        }
    }

    /// 初始化日志系统
    ///
    /// # 错误
    ///
    /// 如果日志系统初始化失败，返回错误
    pub fn init_logging(self) -> Result<Self> {
        init_logging(&self.settings.logging)?;
        Ok(self)
    }

    /// 初始化数据库连接池
    ///
    /// # 错误
    ///
    /// 如果数据库连接池创建失败，返回错误
    pub async fn init_database(mut self) -> Result<Self> {
        let db_pools = DatabasePools::new(&self.settings.database).await?;
        
        tracing::info!("数据库连接池创建成功 (数量: {})", db_pools.count());
        tracing::info!("数据库列表: {:?}", db_pools.names());
        
        // 执行健康检查
        db_pools.health_check().await?;
        
        self.db_pools = Some(db_pools);
        Ok(self)
    }

    /// 配置 CORS
    pub fn setup_cors(mut self) -> Self {
        let cors = setup_cors(&self.settings.cors);
        self.cors = Some(cors);
        self
    }

    /// 获取配置的引用
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// 获取数据库连接池的引用
    pub fn db_pools(&self) -> Option<&DatabasePools> {
        self.db_pools.as_ref()
    }

    /// 消费构建器并返回配置和数据库连接池
    pub fn build(self) -> (Settings, Option<DatabasePools>, Option<CorsLayer>) {
        (self.settings, self.db_pools, self.cors)
    }

    /// 启动服务器
    ///
    /// # 参数
    ///
    /// * `app` - Axum 路由应用
    ///
    /// # 错误
    ///
    /// 如果服务器启动失败，返回错误
    pub async fn serve(self, app: Router) -> Result<()> {
        let addr = SocketAddr::from((
            self.settings.server.host
                .parse::<std::net::IpAddr>()
                .unwrap_or([0, 0, 0, 0].into()),
            self.settings.server.port,
        ));
        
        tracing::info!("服务器监听地址: {}", addr);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| crate::error::AppError::Internal(format!("绑定地址失败: {}", e)))?;
        
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>()
        )
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("服务器运行失败: {}", e)))?;

        Ok(())
    }
}

/// 快速启动服务器的辅助函数
///
/// 从默认配置目录 `config/` 加载配置
///
/// # 参数
///
/// * `router_fn` - 接收配置、数据库连接池和 CORS 层，返回配置好的路由
///
/// # 示例
///
/// ```rust,no_run
/// use axum::{Router, routing::get};
/// use iyucode_core::server::run_server;
///
/// #[tokio::main]
/// async fn main() {
///     run_server(|settings, db_pools, cors| {
///         let mut app = Router::new()
///             .route("/health", get(|| async { "OK" }));
///         
///         if let Some(pools) = db_pools {
///             app = app.layer(axum::Extension(pools));
///         }
///         
///         if let Some(cors_layer) = cors {
///             app = app.layer(cors_layer);
///         }
///         
///         app
///     })
///     .await
///     .expect("服务器启动失败");
/// }
/// ```
pub async fn run_server<F>(router_fn: F) -> Result<()>
where
    F: FnOnce(&Settings, Option<DatabasePools>, Option<CorsLayer>) -> Router,
{
    run_server_with_config("config", router_fn).await
}

/// 从指定配置目录快速启动服务器
///
/// # 参数
///
/// * `config_dir` - 配置文件目录路径
/// * `router_fn` - 接收配置、数据库连接池和 CORS 层，返回配置好的路由
///
/// # 示例
///
/// ```rust,no_run
/// use axum::{Router, routing::get};
/// use iyucode_core::server::run_server_with_config;
///
/// #[tokio::main]
/// async fn main() {
///     run_server_with_config("server/config", |settings, db_pools, cors| {
///         let mut app = Router::new()
///             .route("/health", get(|| async { "OK" }));
///         
///         if let Some(pools) = db_pools {
///             app = app.layer(axum::Extension(pools));
///         }
///         
///         if let Some(cors_layer) = cors {
///             app = app.layer(cors_layer);
///         }
///         
///         app
///     })
///     .await
///     .expect("服务器启动失败");
/// }
/// ```
pub async fn run_server_with_config<F>(config_dir: &str, router_fn: F) -> Result<()>
where
    F: FnOnce(&Settings, Option<DatabasePools>, Option<CorsLayer>) -> Router,
{
    let builder = ServerBuilder::from_config_dir(config_dir)?
        .init_logging()?
        .init_database()
        .await?
        .setup_cors();

    let settings = builder.settings().clone();
    let db_pools = builder.db_pools().cloned();
    let cors = builder.cors.clone();

    let app = router_fn(&settings, db_pools, cors);

    builder.serve(app).await
}
