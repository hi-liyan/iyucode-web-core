///! 应用配置模块
///!
///! 提供多环境配置支持，支持从配置文件和环境变量加载配置

use serde::Deserialize;

/// 应用配置主结构
///
/// 包含所有子模块的配置，支持从 TOML 文件和环境变量加载
///
/// # 配置优先级
/// 1. 环境变量（最高优先级）
/// 2. 环境特定配置文件 `config/{env}.toml`
/// 3. 默认配置文件 `config/default.toml`
///
/// # 示例
/// ```rust
/// use iyucode_core::config::Settings;
///
/// let settings = Settings::new().expect("Failed to load configuration");
/// println!("Server running on {}:{}", settings.server.host, settings.server.port);
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    /// 服务器配置
    pub server: ServerConfig,
    /// 数据库配置
    pub database: DatabaseConfig,
    /// CORS 跨域配置
    pub cors: CorsConfig,
    /// JWT 认证配置
    pub jwt: JwtConfig,
    /// 日志配置
    pub logging: LoggingConfig,
    /// SMTP 邮件配置
    pub smtp: SmtpConfig,
    /// 自定义配置（业务层可以添加任意配置项）
    #[serde(default)]
    pub custom: std::collections::HashMap<String, String>,
}

/// 服务器配置
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// 服务器监听地址，默认 "0.0.0.0"
    pub host: String,
    /// 服务器监听端口，默认 3000
    pub port: u16,
}

/// 数据库配置
///
/// 支持两种配置模式：
/// 1. 单数据库模式（向后兼容）：直接配置 url, max_connections 等
/// 2. 多数据库模式：使用 HashMap 配置多个命名数据库
///
/// # 示例
///
/// 单数据库模式：
/// ```toml
/// [database]
/// url = "mysql://user:pass@localhost/mydb"
/// max_connections = 10
/// min_connections = 2
/// connect_timeout = 30
/// idle_timeout = 600
/// ```
///
/// 多数据库模式：
/// ```toml
/// [database.primary]
/// url = "mysql://user:pass@localhost/db1"
/// max_connections = 20
/// min_connections = 5
/// connect_timeout = 30
/// idle_timeout = 600
///
/// [database.secondary]
/// url = "mysql://user:pass@localhost/db2"
/// max_connections = 10
/// min_connections = 2
/// connect_timeout = 30
/// idle_timeout = 600
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum DatabaseConfig {
    /// 单数据库配置（向后兼容）
    Single(DatabaseConnection),
    /// 多数据库配置（使用命名数据库）
    Multiple(std::collections::HashMap<String, DatabaseConnection>),
}

impl DatabaseConfig {
    /// 获取单数据库配置（向后兼容）
    ///
    /// 如果是单数据库模式，返回该配置
    /// 如果是多数据库模式，返回 None
    pub fn single(&self) -> Option<&DatabaseConnection> {
        match self {
            DatabaseConfig::Single(conn) => Some(conn),
            DatabaseConfig::Multiple(_) => None,
        }
    }

    /// 获取指定名称的数据库配置
    ///
    /// # 参数
    /// - `name`: 数据库名称（如 "primary", "secondary"）
    ///
    /// # 返回
    /// - 单数据库模式：忽略 name，返回唯一的配置
    /// - 多数据库模式：返回指定名称的配置，如果不存在返回 None
    pub fn get(&self, name: &str) -> Option<&DatabaseConnection> {
        match self {
            DatabaseConfig::Single(conn) => Some(conn),
            DatabaseConfig::Multiple(map) => map.get(name),
        }
    }

    /// 获取所有数据库配置
    ///
    /// # 返回
    /// - 单数据库模式：返回包含一个元素的 Vec [("default", config)]
    /// - 多数据库模式：返回所有命名数据库配置
    pub fn all(&self) -> Vec<(&str, &DatabaseConnection)> {
        match self {
            DatabaseConfig::Single(conn) => vec![("default", conn)],
            DatabaseConfig::Multiple(map) => {
                map.iter().map(|(k, v)| (k.as_str(), v)).collect()
            }
        }
    }

    /// 检查是否为多数据库模式
    pub fn is_multiple(&self) -> bool {
        matches!(self, DatabaseConfig::Multiple(_))
    }

    /// 获取数据库数量
    pub fn count(&self) -> usize {
        match self {
            DatabaseConfig::Single(_) => 1,
            DatabaseConfig::Multiple(map) => map.len(),
        }
    }
}

/// 单个数据库连接配置
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConnection {
    /// 数据库连接 URL
    ///
    /// 格式: `mysql://username:password@host:port/database`
    pub url: String,
    /// 连接池最大连接数，默认 10
    pub max_connections: u32,
    /// 连接池最小连接数，默认 2
    pub min_connections: u32,
    /// 连接超时时间（秒），默认 30
    pub connect_timeout: u64,
    /// 空闲连接超时时间（秒），默认 600（10分钟）
    pub idle_timeout: u64,
}

/// CORS 跨域资源共享配置
#[derive(Debug, Clone, Deserialize)]
pub struct CorsConfig {
    /// 允许的源列表
    ///
    /// 支持 "*" 表示允许所有源（不推荐用于生产环境）
    /// 示例: `["http://localhost:3001", "https://example.com"]`
    pub allowed_origins: Vec<String>,
    /// 允许的 HTTP 方法列表
    ///
    /// 示例: `["GET", "POST", "PUT", "DELETE", "OPTIONS"]`
    pub allowed_methods: Vec<String>,
    /// 允许的请求头列表
    ///
    /// 示例: `["Content-Type", "Authorization"]`
    pub allowed_headers: Vec<String>,
    /// 预检请求缓存时间（秒），默认 3600（1小时）
    pub max_age: u64,
}

/// JWT 认证配置
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    /// JWT 签名密钥
    ///
    /// **重要**: 生产环境必须使用强密钥（至少 256 位）
    pub secret: String,
    /// Token 过期时间（秒），默认 86400（24小时）
    pub expiration: i64,
}

/// 日志配置
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    ///
    /// 可选值: "trace", "debug", "info", "warn", "error"
    pub level: String,
    /// 日志输出格式
    ///
    /// 可选值: "json"（结构化JSON）, "pretty"（人类可读）
    pub format: String,
}

/// SMTP 邮件服务器配置
#[derive(Debug, Clone, Deserialize)]
pub struct SmtpConfig {
    /// SMTP 服务器地址
    ///
    /// 示例: "smtp.gmail.com", "smtp.qq.com"
    pub host: String,
    /// SMTP 服务器端口
    ///
    /// 常用端口: 25 (非加密), 465 (SSL), 587 (TLS)
    pub port: u16,
    /// SMTP 用户名（邮箱地址）
    ///
    /// 示例: "noreply@example.com"
    pub username: String,
    /// SMTP 密码或授权码
    pub password: String,
    /// 发件人邮箱地址
    ///
    /// 示例: "noreply@example.com"
    pub from: String,
    /// 发件人显示名称（可选）
    ///
    /// 示例: "牛牛账簿"
    #[serde(default)]
    pub from_name: Option<String>,
    /// 连接超时时间（秒），默认 30
    #[serde(default = "default_smtp_timeout")]
    pub timeout: u64,
}

fn default_smtp_timeout() -> u64 {
    30
}

impl Settings {
    /// 加载应用配置
    ///
    /// 从配置文件和环境变量加载配置，支持多环境配置
    ///
    /// # 环境变量
    /// - `APP_ENV`: 指定环境（development, testing, production），默认 "development"
    /// - `APP_CONFIG_DIR`: 配置文件目录，默认 "config"
    /// - `APP_*`: 覆盖配置项，使用 `__` 作为分隔符
    ///   - 示例: `APP_SERVER__PORT=8080` 覆盖 `server.port`
    ///   - 示例: `APP_DATABASE__URL=mysql://...` 覆盖 `database.url`
    ///
    /// # 配置文件
    /// - `config/default.toml`: 默认配置
    /// - `config/{env}.toml`: 环境特定配置
    ///
    /// # 错误
    /// 如果配置文件格式错误或必需字段缺失，返回 `ConfigError`
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::config::Settings;
    ///
    /// // 使用默认环境（development）
    /// let settings = Settings::new().expect("Failed to load configuration");
    ///
    /// // 使用环境变量指定环境
    /// std::env::set_var("APP_ENV", "production");
    /// let settings = Settings::new().expect("Failed to load configuration");
    /// ```
    pub fn new() -> Result<Self, config::ConfigError> {
        Self::from_dir("config")
    }

    /// 从指定目录加载配置
    ///
    /// # 参数
    /// - `config_dir`: 配置文件目录路径
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::config::Settings;
    ///
    /// // 从 server/config 目录加载配置
    /// let settings = Settings::from_dir("server/config").expect("Failed to load configuration");
    /// ```
    pub fn from_dir(config_dir: &str) -> Result<Self, config::ConfigError> {
        // 获取当前环境，默认为 development
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());

        // 构建配置
        let config = config::Config::builder()
            // 1. 加载默认配置文件（可选）
            .add_source(
                config::File::with_name(&format!("{}/default", config_dir)).required(false),
            )
            // 2. 加载环境特定配置文件（可选）
            .add_source(
                config::File::with_name(&format!("{}/{}", config_dir, env)).required(false),
            )
            // 3. 从环境变量加载配置（最高优先级）
            // 前缀: APP_
            // 分隔符: __ (双下划线)
            // 示例: APP_SERVER__PORT=8080 -> server.port = 8080
            .add_source(
                config::Environment::with_prefix("APP")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        // 反序列化为 Settings 结构体
        config.try_deserialize()
    }

    /// 验证配置的有效性
    ///
    /// 检查配置项是否符合要求，例如端口范围、URL 格式等
    ///
    /// # 错误
    /// 如果配置无效，返回描述性错误信息
    pub fn validate(&self) -> Result<(), String> {
        // 验证服务器端口范围
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }

        // 验证数据库连接数配置
        let validate_connection = |conn: &DatabaseConnection, name: &str| -> Result<(), String> {
            if conn.max_connections < conn.min_connections {
                return Err(format!(
                    "Database '{}': max_connections must be >= min_connections",
                    name
                ));
            }
            if conn.max_connections == 0 {
                return Err(format!(
                    "Database '{}': max_connections must be > 0",
                    name
                ));
            }
            Ok(())
        };

        // 验证所有数据库配置
        for (name, conn) in self.database.all() {
            validate_connection(conn, name)?;
        }

        // 验证 JWT 密钥长度（建议至少 32 字符）
        if self.jwt.secret.len() < 32 {
            return Err(
                "JWT secret should be at least 32 characters for security".to_string(),
            );
        }

        // 验证日志级别
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            return Err(format!(
                "Invalid logging level '{}'. Must be one of: {}",
                self.logging.level,
                valid_levels.join(", ")
            ));
        }

        // 验证日志格式
        let valid_formats = ["json", "pretty"];
        if !valid_formats.contains(&self.logging.format.as_str()) {
            return Err(format!(
                "Invalid logging format '{}'. Must be one of: {}",
                self.logging.format,
                valid_formats.join(", ")
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_server_port() {
        let mut settings = create_test_settings();
        settings.server.port = 0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_database_connections() {
        let mut settings = create_test_settings();
        if let DatabaseConfig::Single(ref mut conn) = settings.database {
            conn.max_connections = 5;
            conn.min_connections = 10;
        }
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_jwt_secret_length() {
        let mut settings = create_test_settings();
        settings.jwt.secret = "short".to_string();
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_logging_level() {
        let mut settings = create_test_settings();
        settings.logging.level = "invalid".to_string();
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_valid_settings() {
        let settings = create_test_settings();
        assert!(settings.validate().is_ok());
    }

    /// 创建测试用的配置
    fn create_test_settings() -> Settings {
        Settings {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
            },
            database: DatabaseConfig::Single(DatabaseConnection {
                url: "mysql://root:password@localhost:3306/test".to_string(),
                max_connections: 10,
                min_connections: 2,
                connect_timeout: 30,
                idle_timeout: 600,
            }),
            cors: CorsConfig {
                allowed_origins: vec!["*".to_string()],
                allowed_methods: vec!["GET".to_string(), "POST".to_string()],
                allowed_headers: vec!["Content-Type".to_string()],
                max_age: 3600,
            },
            jwt: JwtConfig {
                secret: "this-is-a-very-long-secret-key-for-testing-purposes".to_string(),
                expiration: 86400,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
            smtp: SmtpConfig {
                host: "smtp.example.com".to_string(),
                port: 587,
                username: "noreply@example.com".to_string(),
                password: "password".to_string(),
                from: "noreply@example.com".to_string(),
                from_name: Some("Test App".to_string()),
                timeout: 30,
            },
            custom: std::collections::HashMap::new(),
        }
    }
}
