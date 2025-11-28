# IyuCode Core

[![Crates.io](https://img.shields.io/crates/v/iyucode-core.svg)](https://crates.io/crates/iyucode-core)
[![Documentation](https://docs.rs/iyucode-core/badge.svg)](https://docs.rs/iyucode-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

生产级别的 Rust Web API 基础设施库，基于 Axum 0.8 框架构建。提供开箱即用的配置管理、数据库连接、API 响应格式化、错误处理、中间件等核心功能。

## 版本要求

- Rust 1.75+
- Axum 0.8.6
- Tokio 1.48.0

## 特性

- 🚀 **开箱即用** - 提供完整的 Web API 基础设施
- 🔧 **多环境配置** - 支持开发、测试、生产等多环境配置
- 💾 **数据库连接池** - 内置 MySQL 连接池管理
- ✅ **请求验证** - 自动验证 API 请求参数
- 📦 **统一响应格式** - 标准化的 JSON 响应结构
- 🔐 **认证中间件** - JWT 认证支持
- 🌐 **CORS 支持** - 跨域资源共享配置
- 🎯 **错误处理** - 统一的错误类型和响应
- 📝 **结构化日志** - 基于 tracing 的日志系统
- 🧩 **模块化设计** - 各功能模块独立，按需使用
- 🛠️ **通用工具** - 密码加密、频率限制、时区转换、输入验证

## 安装

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
iyucode-core = "0.1.0"
axum = "0.8.6"
tokio = { version = "1.48.0", features = ["full"] }
```

## 快速开始

### 1. 基本使用

```rust
use axum::{routing::get, Router};
use iyucode_core::{ApiResponse, AppError, Result};

#[tokio::main]
async fn main() {
    // 创建路由
    let app = Router::new()
        .route("/api/hello", get(hello_handler));

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    axum::serve(listener, app).await.unwrap();
}

// 处理器函数
async fn hello_handler() -> Result<ApiResponse<String>> {
    Ok(ApiResponse::success("Hello, World!".to_string()))
}
```

### 2. 配置管理

创建配置文件 `config/development.toml`：

```toml
[server]
host = "0.0.0.0"
port = 3000

[database]
url = "mysql://user:password@localhost:3306/mydb"
max_connections = 10
min_connections = 2
connect_timeout = 30

[cors]
allowed_origins = ["http://localhost:3001"]
allowed_methods = ["GET", "POST", "PUT", "DELETE"]
allowed_headers = ["Content-Type", "Authorization"]

[jwt]
secret = "your-secret-key"
expiration = 86400  # 24 hours in seconds

[logging]
level = "debug"
format = "json"
```

加载配置：

```rust
use iyucode_core::config::Settings;

#[tokio::main]
async fn main() {
    // 加载配置（默认使用 development 环境）
    let settings = Settings::new().expect("Failed to load configuration");
    
    println!("Server running on {}:{}", settings.server.host, settings.server.port);
}
```

### 3. 数据库连接

```rust
use iyucode_core::database::DatabasePool;
use iyucode_core::config::Settings;

#[tokio::main]
async fn main() {
    let settings = Settings::new().unwrap();
    
    // 创建数据库连接池
    let db_pool = DatabasePool::new(&settings.database)
        .await
        .expect("Failed to create database pool");
    
    // 使用连接池
    let pool = db_pool.pool();
    
    // 执行查询
    let result = sqlx::query!("SELECT 1 as value")
        .fetch_one(pool)
        .await
        .unwrap();
    
    println!("Query result: {}", result.value);
}
```

### 4. API 响应格式化

```rust
use axum::{routing::get, Router, Json};
use iyucode_core::{ApiResponse, Result};
use serde::Serialize;

#[derive(Serialize)]
struct User {
    id: i32,
    name: String,
    email: String,
}

async fn get_user() -> Result<ApiResponse<User>> {
    let user = User {
        id: 1,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };
    
    Ok(ApiResponse::success(user))
}

// 响应格式（小驼峰命名）：
// {
//   "code": 200,
//   "message": "Success",
//   "data": {
//     "id": 1,
//     "name": "John Doe",
//     "email": "john@example.com"
//   }
// }
```

### 5. 请求验证

```rust
use axum::{routing::post, Router};
use iyucode_core::{validation::ValidatedJson, ApiResponse, Result};
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
struct CreateUserRequest {
    #[validate(length(min = 3, max = 50))]
    name: String,
    
    #[validate(email)]
    email: String,
    
    #[validate(length(min = 8))]
    password: String,
}

async fn create_user(
    ValidatedJson(payload): ValidatedJson<CreateUserRequest>
) -> Result<ApiResponse<String>> {
    // 参数已自动验证，这里直接使用
    println!("Creating user: {:?}", payload);
    
    Ok(ApiResponse::success("User created successfully".to_string()))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/users", post(create_user));
    
    // 启动服务器...
}
```

### 6. 错误处理

```rust
use iyucode_core::{AppError, Result, ApiResponse};

async fn get_user_by_id(id: i32) -> Result<ApiResponse<User>> {
    if id <= 0 {
        return Err(AppError::Validation("Invalid user ID".to_string()));
    }
    
    // 查询数据库
    let user = fetch_user_from_db(id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    match user {
        Some(u) => Ok(ApiResponse::success(u)),
        None => Err(AppError::NotFound(format!("User {} not found", id))),
    }
}

// 错误会自动转换为标准响应（小驼峰命名）：
// {
//   "code": 404,
//   "message": "User 123 not found",
//   "data": null
// }
```

### 7. CORS 中间件

```rust
use axum::Router;
use iyucode_core::{middleware::setup_cors, config::Settings};

#[tokio::main]
async fn main() {
    let settings = Settings::new().unwrap();
    
    let app = Router::new()
        .route("/api/hello", get(hello_handler))
        .layer(setup_cors(&settings.cors));
    
    // 启动服务器...
}
```

### 8. JWT 认证中间件

```rust
use axum::{routing::get, Router, middleware};
use iyucode_core::{
    middleware::AuthMiddleware,
    config::Settings,
    ApiResponse,
    Result,
};

async fn protected_handler() -> Result<ApiResponse<String>> {
    Ok(ApiResponse::success("This is a protected route".to_string()))
}

#[tokio::main]
async fn main() {
    let settings = Settings::new().unwrap();
    let auth_middleware = AuthMiddleware::new(settings.jwt.secret.clone());
    
    let app = Router::new()
        .route("/api/protected", get(protected_handler))
        .layer(middleware::from_fn(move |req, next| {
            auth_middleware.clone().authenticate(req, next)
        }));
    
    // 启动服务器...
}
```

### 9. 工具模块

#### 密码加密

```rust
use iyucode_core::PasswordHasher;

// 加密密码
let hash = PasswordHasher::hash("my_password")?;

// 验证密码
let is_valid = PasswordHasher::verify("my_password", &hash)?;
assert!(is_valid);
```

#### 频率限制

```rust
use iyucode_core::RateLimiter;

// 创建限制器：每分钟最多 60 次请求
let limiter = RateLimiter::new(60, 60);

// 检查是否允许请求
if limiter.check("user_id_or_ip").await {
    // 处理请求
} else {
    // 返回 429 Too Many Requests
}

// 查询剩余配额
let remaining = limiter.remaining("user_id").await;
```

#### 时区转换

```rust
use iyucode_core::TimezoneConverter;
use chrono::Utc;

// 获取当前时间（指定时区）
let now_shanghai = TimezoneConverter::now(8);  // UTC+8

// UTC 转本地时区
let utc_time = Utc::now();
let local_time = TimezoneConverter::utc_to_offset(utc_time, 8);

// 格式化为字符串
let formatted = TimezoneConverter::format(utc_time, 8);
```

#### 输入验证

```rust
use iyucode_core::Validator;

// 验证邮箱
Validator::email("test@example.com")?;

// 验证密码（默认：8-128字符，包含字母和数字）
Validator::password("MyPass123")?;

// 自定义密码策略
Validator::password_with_policy(
    "MyP@ss123456",
    12,    // 最小长度
    128,   // 最大长度
    true,  // 要求字母
    true,  // 要求数字
    true,  // 要求特殊字符
)?;

// 验证用户名
Validator::username("john_doe")?;

// 验证长度
Validator::length("name", "John", 2, 50)?;

// 验证范围
Validator::range("age", 25, 1, 120)?;

// 验证 URL
Validator::url("https://example.com")?;
```

### 10. 完整示例

```rust
use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use iyucode_core::{
    config::Settings,
    database::DatabasePool,
    middleware::{setup_cors, AuthMiddleware},
    ApiResponse,
    Result,
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载配置
    let settings = Settings::new().expect("Failed to load configuration");

    // 创建数据库连接池
    let db_pool = DatabasePool::new(&settings.database)
        .await
        .expect("Failed to create database pool");

    // 创建认证中间件
    let auth_middleware = AuthMiddleware::new(settings.jwt.secret.clone());

    // 公开路由
    let public_routes = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/login", post(login));

    // 受保护路由
    let protected_routes = Router::new()
        .route("/api/users", get(get_users))
        .layer(middleware::from_fn(move |req, next| {
            auth_middleware.clone().authenticate(req, next)
        }));

    // 组合路由
    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(setup_cors(&settings.cors))
        .layer(TraceLayer::new_for_http())
        .with_state(db_pool);

    // 启动服务器
    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    tracing::info!("Server running on {}", addr);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

async fn health_check() -> Result<ApiResponse<String>> {
    Ok(ApiResponse::success("OK".to_string()))
}

async fn login() -> Result<ApiResponse<String>> {
    // 登录逻辑...
    Ok(ApiResponse::success("token".to_string()))
}

async fn get_users() -> Result<ApiResponse<Vec<String>>> {
    // 获取用户列表...
    Ok(ApiResponse::success(vec!["user1".to_string(), "user2".to_string()]))
}
```

## 架构设计

### 模块结构

```
iyucode-core/
├── src/
│   ├── lib.rs              # 库入口，导出公共 API
│   ├── config/             # 配置管理模块
│   │   └── settings.rs     # 配置结构定义和加载
│   ├── database/           # 数据库模块
│   │   └── pool.rs         # 连接池管理
│   ├── error.rs            # 错误类型定义
│   ├── response.rs         # API 响应格式
│   ├── validation.rs       # 请求验证
│   ├── logging.rs          # 日志初始化
│   ├── middleware/         # 中间件模块
│   │   ├── auth.rs         # JWT 认证中间件
│   │   └── cors.rs         # CORS 中间件
│   └── utils/              # 工具模块
│       ├── crypto.rs       # 密码加密（Argon2）
│       ├── rate_limit.rs   # 频率限制器
│       ├── timezone.rs     # 时区转换
│       └── validator.rs    # 输入验证
└── Cargo.toml
```

### 设计原则

1. **分层架构**
   - 配置层：管理应用配置
   - 数据层：数据库连接和操作
   - 业务层：由使用者实现
   - 表现层：API 路由和响应

2. **依赖注入**
   - 通过 Axum 的 State 机制注入依赖
   - 支持共享状态（如数据库连接池）

3. **错误处理**
   - 统一的错误类型 `AppError`
   - 自动转换为标准 HTTP 响应
   - 区分开发和生产环境的错误详情

4. **类型安全**
   - 充分利用 Rust 类型系统
   - 编译时检查，减少运行时错误

5. **可扩展性**
   - 模块化设计，按需使用
   - 提供清晰的扩展点

## 环境变量

支持通过环境变量覆盖配置：

```bash
# 指定环境
export APP_ENV=production

# 覆盖特定配置
export APP_SERVER__PORT=8080
export APP_DATABASE__URL=mysql://user:pass@localhost/db
export APP_JWT__SECRET=your-secret-key
```

## 配置文件优先级

1. 环境变量（最高优先级）
2. `config/{environment}.toml`
3. `config/default.toml`
4. 代码中的默认值（最低优先级）

## 错误类型

| 错误类型 | HTTP 状态码 | 说明 |
|---------|-----------|------|
| `AppError::Database` | 500 | 数据库操作错误 |
| `AppError::Validation` | 400 | 请求参数验证失败 |
| `AppError::Authentication` | 401 | 认证失败 |
| `AppError::Authorization` | 403 | 权限不足 |
| `AppError::NotFound` | 404 | 资源不存在 |
| `AppError::Internal` | 500 | 内部服务器错误 |

## API 响应格式

### 成功响应

```json
{
  "code": 200,
  "message": "Success",
  "data": {
    "id": 1,
    "name": "Example"
  }
}
```

### 错误响应

```json
{
  "code": 400,
  "message": "Validation error: email is invalid",
  "data": null
}
```

### 分页响应

```json
{
  "code": 200,
  "message": "Success",
  "data": {
    "items": [...],
    "total": 100,
    "page": 1,
    "page_size": 20
  }
}
```

## 最佳实践

### 1. 项目结构建议

```
your-project/
├── src/
│   ├── main.rs             # 应用入口
│   ├── routes/             # 路由定义
│   │   ├── mod.rs
│   │   ├── users.rs
│   │   └── posts.rs
│   ├── handlers/           # 请求处理器
│   │   ├── mod.rs
│   │   ├── user_handler.rs
│   │   └── post_handler.rs
│   ├── services/           # 业务逻辑
│   │   ├── mod.rs
│   │   ├── user_service.rs
│   │   └── post_service.rs
│   ├── models/             # 数据模型
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   └── post.rs
│   └── repositories/       # 数据访问层
│       ├── mod.rs
│       ├── user_repo.rs
│       └── post_repo.rs
├── config/
│   ├── default.toml
│   ├── development.toml
│   ├── testing.toml
│   └── production.toml
└── Cargo.toml
```

### 2. 使用 State 共享依赖

```rust
use axum::{extract::State, Router};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db: DatabasePool,
    config: Arc<Settings>,
}

async fn handler(State(state): State<AppState>) {
    // 使用 state.db 和 state.config
}

let state = AppState {
    db: db_pool,
    config: Arc::new(settings),
};

let app = Router::new()
    .route("/api/endpoint", get(handler))
    .with_state(state);
```

### 3. 自定义错误类型

```rust
use iyucode_core::AppError;

// 扩展错误类型
impl From<MyCustomError> for AppError {
    fn from(err: MyCustomError) -> Self {
        AppError::Internal(err.to_string())
    }
}
```

## 测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let app = Router::new()
            .route("/health", get(health_check));

        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
```

## 性能优化建议

1. **数据库连接池**：根据负载调整 `max_connections`
2. **异步处理**：充分利用 Tokio 的异步特性
3. **缓存**：对频繁访问的数据使用缓存
4. **日志级别**：生产环境使用 `info` 或 `warn` 级别

## 常见问题

### Q: 如何在其他项目中使用？

A: 直接复制 `iyucode-core` 目录到新项目，或者在 `Cargo.toml` 中引用：

```toml
[dependencies]
iyucode-core = { path = "../iyucode-core" }
```

### Q: 如何自定义响应格式？

A: 可以创建自己的响应类型，或者扩展 `ApiResponse`：

```rust
impl<T: Serialize> ApiResponse<T> {
    pub fn custom(code: u16, message: String, data: T) -> Self {
        Self { code, message, data: Some(data) }
    }
}
```

### Q: 支持其他数据库吗？

A: 当前支持 MySQL，可以通过修改 `sqlx` features 支持 PostgreSQL、SQLite 等。

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT License

## 文档

- [工具模块使用指南](docs/UTILS_GUIDE.md) - 详细的工具模块文档和示例

## 更新日志

### v0.1.1 (2024-11-15)

- ✨ 新增工具模块（utils）
  - 密码加密工具（Argon2id）
  - 频率限制器（滑动窗口算法）
  - 时区转换工具（支持任意时区）
  - 输入验证器（邮箱、密码、用户名等）
- 📝 完善文档和示例
- 🔧 优化错误处理

### v0.1.0 (2024-11-14)

- 🎉 初始版本发布
- 支持多环境配置
- MySQL 数据库连接池
- API 响应格式化
- 请求验证
- JWT 认证中间件
- CORS 支持
- 统一错误处理
- 结构化日志
