# 工具模块使用指南

iyucode-core 提供了一套通用的工具模块，适用于大多数 Web 应用开发场景。

## 模块概览

- **crypto** - 密码加密和哈希
- **rate_limit** - 请求频率限制
- **timezone** - 时区转换
- **validator** - 输入验证

---

## 1. 密码加密 (crypto)

基于 Argon2id 算法的密码加密工具。

### 基本使用

```rust
use iyucode_core::PasswordHasher;

// 加密密码
let hash = PasswordHasher::hash("my_password")?;

// 验证密码
let is_valid = PasswordHasher::verify("my_password", &hash)?;
assert!(is_valid);
```

### 特性

- ✅ 使用 Argon2id 算法（最安全的密码哈希算法）
- ✅ 自动生成随机盐
- ✅ 抗暴力破解
- ✅ 抗彩虹表攻击
- ✅ 抗时序攻击

### 错误处理

```rust
use iyucode_core::utils::crypto::CryptoError;

match PasswordHasher::hash("password") {
    Ok(hash) => println!("Hash: {}", hash),
    Err(CryptoError::HashError(msg)) => eprintln!("加密失败: {}", msg),
    Err(CryptoError::VerifyError(msg)) => eprintln!("验证失败: {}", msg),
}
```

---

## 2. 频率限制 (rate_limit)

基于滑动窗口的频率限制器。

### 基本使用

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
```

### 高级功能

```rust
// 查询剩余配额
let remaining = limiter.remaining("user_123").await;
println!("剩余配额: {}", remaining);

// 查询重置时间
if let Some(duration) = limiter.reset_after("user_123").await {
    println!("{}秒后重置", duration.as_secs());
}

// 手动重置
limiter.reset("user_123").await;

// 清理过期记录
limiter.cleanup().await;
```

### 在 Axum 中使用

```rust
use axum::{extract::State, http::StatusCode};
use iyucode_core::{RateLimiter, ApiResponse};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    rate_limiter: Arc<RateLimiter>,
}

async fn handler(
    State(state): State<AppState>,
    // 获取客户端 IP
) -> Result<ApiResponse<String>, StatusCode> {
    if !state.rate_limiter.check("client_ip").await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    Ok(ApiResponse::success("Success".to_string()))
}
```

### 常见配置

```rust
// 登录限制：每分钟 5 次
let login_limiter = RateLimiter::new(5, 60);

// 注册限制：每 5 分钟 3 次
let register_limiter = RateLimiter::new(3, 300);

// API 调用限制：每小时 1000 次
let api_limiter = RateLimiter::new(1000, 3600);

// 严格限制：每天 10 次
let strict_limiter = RateLimiter::new(10, 86400);
```

---

## 3. 时区转换 (timezone)

UTC 和各时区之间的转换工具。

### 基本使用

```rust
use iyucode_core::TimezoneConverter;
use chrono::Utc;

// 获取当前时间（指定时区）
let now_shanghai = TimezoneConverter::now(8);  // UTC+8
let now_tokyo = TimezoneConverter::now(9);     // UTC+9

// UTC 转本地时区
let utc_time = Utc::now();
let local_time = TimezoneConverter::utc_to_offset(utc_time, 8);

// 本地时区转 UTC
let utc_time = TimezoneConverter::to_utc(local_time);

// 格式化为字符串
let formatted = TimezoneConverter::format(utc_time, 8);
println!("上海时间: {}", formatted);
```

### 时区之间转换

```rust
// 上海时间转东京时间
let shanghai_time = TimezoneConverter::now(8);
let tokyo_time = TimezoneConverter::convert(shanghai_time, 9);
```

### 使用预定义时区常量

```rust
use iyucode_core::utils::timezone::timezones;

let china_time = TimezoneConverter::now(timezones::CHINA);
let japan_time = TimezoneConverter::now(timezones::JAPAN);
let us_east_time = TimezoneConverter::now(timezones::US_EAST);
```

### 在 API 响应中使用

```rust
use iyucode_core::{ApiResponse, TimezoneConverter};
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UserResponse {
    id: i64,
    email: String,
    created_at_utc: DateTime<Utc>,
    created_at_local: String,  // 本地时区字符串
}

fn to_response(user: User) -> UserResponse {
    UserResponse {
        id: user.id,
        email: user.email,
        created_at_utc: user.created_at,
        created_at_local: TimezoneConverter::format(user.created_at, 8),
    }
}
```

---

## 4. 输入验证 (validator)

通用的输入验证工具。

### 邮箱验证

```rust
use iyucode_core::Validator;

match Validator::email("test@example.com") {
    Ok(_) => println!("邮箱有效"),
    Err(e) => println!("验证失败: {}", e),
}
```

### 密码验证

```rust
// 默认策略：8-128字符，包含字母和数字
Validator::password("MyPass123")?;

// 自定义策略：12字符，包含字母、数字和特殊字符
Validator::password_with_policy(
    "MyP@ss123456",
    12,    // 最小长度
    128,   // 最大长度
    true,  // 要求字母
    true,  // 要求数字
    true,  // 要求特殊字符
)?;
```

### 用户名验证

```rust
// 默认：3-20字符，只能包含字母、数字和下划线
Validator::username("john_doe")?;

// 自定义长度
Validator::username_with_length("john", 2, 50)?;
```

### 长度验证

```rust
Validator::length("name", "John Doe", 2, 50)?;
```

### 范围验证

```rust
Validator::range("age", 25, 1, 120)?;
Validator::range("price", 99.99, 0.0, 1000.0)?;
```

### URL 验证

```rust
Validator::url("https://example.com")?;
```

### 必填验证

```rust
Validator::required("name", "John")?;
```

### 在 Axum 中使用

```rust
use axum::Json;
use iyucode_core::{ApiResponse, Validator, AppError};
use serde::Deserialize;

#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
    username: String,
}

async fn register(
    Json(payload): Json<RegisterRequest>
) -> Result<ApiResponse<String>, AppError> {
    // 验证输入
    Validator::email(&payload.email)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    
    Validator::password(&payload.password)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    
    Validator::username(&payload.username)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    
    // 处理注册逻辑...
    
    Ok(ApiResponse::success("注册成功".to_string()))
}
```

### 批量验证

```rust
use iyucode_core::utils::validator::ValidationError;

fn validate_user(email: &str, password: &str, age: i32) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    if let Err(e) = Validator::email(email) {
        errors.push(e);
    }
    
    if let Err(e) = Validator::password(password) {
        errors.push(e);
    }
    
    if let Err(e) = Validator::range("age", age, 1, 120) {
        errors.push(e);
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

---

## 组合使用示例

### 完整的用户注册流程

```rust
use iyucode_core::{
    ApiResponse, Result, AppError,
    PasswordHasher, RateLimiter, Validator,
};
use axum::{extract::State, Json};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    rate_limiter: Arc<RateLimiter>,
}

#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<ApiResponse<String>> {
    // 1. 频率限制
    if !state.rate_limiter.check("register").await {
        return Err(AppError::BadRequest("请求过于频繁".to_string()));
    }
    
    // 2. 验证邮箱
    Validator::email(&payload.email)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    
    // 3. 验证密码
    Validator::password(&payload.password)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    
    // 4. 加密密码
    let password_hash = PasswordHasher::hash(&payload.password)
        .map_err(|_| AppError::Internal("密码加密失败".to_string()))?;
    
    // 5. 保存到数据库...
    
    Ok(ApiResponse::success("注册成功".to_string()))
}
```

---

## 最佳实践

### 1. 频率限制

- 为不同的操作使用不同的限制器
- 定期调用 `cleanup()` 清理过期记录
- 在响应头中返回剩余配额信息

```rust
// 在响应头中添加限流信息
let remaining = limiter.remaining(key).await;
let reset_after = limiter.reset_after(key).await;

// 设置响应头
// X-RateLimit-Limit: 60
// X-RateLimit-Remaining: 45
// X-RateLimit-Reset: 1234567890
```

### 2. 密码安全

- 永远不要记录明文密码
- 使用 HTTPS 传输密码
- 考虑添加密码强度提示
- 定期提醒用户更换密码

### 3. 时区处理

- 数据库统一使用 UTC 存储
- API 返回 UTC 时间或同时返回 UTC 和本地时间
- 前端根据用户时区显示
- 避免在数据库层面设置时区

### 4. 输入验证

- 在多个层面验证（前端 + 后端）
- 提供清晰的错误消息
- 使用白名单而非黑名单
- 验证所有用户输入

---

## 性能考虑

### 频率限制器

- 使用 `Arc` 共享限制器实例
- 定期清理过期记录避免内存泄漏
- 考虑使用 Redis 实现分布式限流

### 密码哈希

- Argon2 计算密集，考虑异步处理
- 不要在循环中哈希密码
- 考虑使用专门的密码服务

### 验证

- 正则表达式使用 `Lazy` 避免重复编译
- 批量验证时收集所有错误一次返回
- 考虑使用 `validator` crate 的派生宏

---

## 迁移指南

如果你的项目之前使用自定义工具，迁移到 iyucode-core 很简单：

### 从自定义密码哈希迁移

```rust
// 之前
use crate::utils::PasswordHasher;
let hash = PasswordHasher::hash(password)?;

// 现在
use iyucode_core::PasswordHasher;
let hash = PasswordHasher::hash(password)
    .map_err(|_| YourError::InternalError)?;
```

### 从自定义验证迁移

```rust
// 之前
use crate::utils::Validator;
Validator::validate_email(email)?;

// 现在
use iyucode_core::Validator;
Validator::email(email)
    .map_err(|e| YourError::Validation(e.to_string()))?;
```

---

## 总结

iyucode-core 的工具模块提供了：

- ✅ 开箱即用的常用功能
- ✅ 类型安全和错误处理
- ✅ 详细的文档和示例
- ✅ 生产级别的性能和安全性
- ✅ 易于扩展和自定义

这些工具可以显著减少样板代码，让你专注于业务逻辑开发。
