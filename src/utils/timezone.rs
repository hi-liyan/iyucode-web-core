//! 时区转换工具
//!
//! 提供 UTC 和各时区之间的转换功能
//!
//! # 示例
//! ```rust
//! use iyucode_core::utils::TimezoneConverter;
//! use chrono::Utc;
//!
//! // 获取当前时间（指定时区）
//! let now = TimezoneConverter::now(8); // UTC+8
//!
//! // UTC 转本地时区
//! let utc_time = Utc::now();
//! let local_time = TimezoneConverter::utc_to_offset(utc_time, 8);
//! ```

use chrono::{DateTime, FixedOffset, Utc};

/// 时区转换器
///
/// 提供 UTC 和固定偏移时区之间的转换
///
/// # 常用时区偏移
/// - 中国（上海/北京）: +8
/// - 日本（东京）: +9
/// - 美国东部（纽约）: -5 (标准时间) / -4 (夏令时)
/// - 美国西部（洛杉矶）: -8 (标准时间) / -7 (夏令时)
/// - 英国（伦敦）: 0 (标准时间) / +1 (夏令时)
pub struct TimezoneConverter;

impl TimezoneConverter {
    /// 创建固定偏移时区
    ///
    /// # 参数
    /// * `offset_hours` - 相对 UTC 的小时偏移（可以是负数）
    ///
    /// # 返回
    /// * `FixedOffset` - 固定偏移时区
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::TimezoneConverter;
    ///
    /// let shanghai = TimezoneConverter::timezone(8);   // UTC+8
    /// let tokyo = TimezoneConverter::timezone(9);      // UTC+9
    /// let new_york = TimezoneConverter::timezone(-5);  // UTC-5
    /// ```
    pub fn timezone(offset_hours: i32) -> FixedOffset {
        FixedOffset::east_opt(offset_hours * 3600)
            .unwrap_or_else(|| FixedOffset::east_opt(0).unwrap())
    }

    /// 将 UTC 时间转换为指定时区
    ///
    /// # 参数
    /// * `utc_time` - UTC 时间
    /// * `offset_hours` - 目标时区的小时偏移
    ///
    /// # 返回
    /// * 指定时区的时间
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::TimezoneConverter;
    /// use chrono::Utc;
    ///
    /// let utc_time = Utc::now();
    /// let shanghai_time = TimezoneConverter::utc_to_offset(utc_time, 8);
    /// let tokyo_time = TimezoneConverter::utc_to_offset(utc_time, 9);
    /// ```
    pub fn utc_to_offset(utc_time: DateTime<Utc>, offset_hours: i32) -> DateTime<FixedOffset> {
        utc_time.with_timezone(&Self::timezone(offset_hours))
    }

    /// 将指定时区的时间转换为 UTC
    ///
    /// # 参数
    /// * `local_time` - 本地时区的时间
    ///
    /// # 返回
    /// * UTC 时间
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::TimezoneConverter;
    /// use chrono::Utc;
    ///
    /// let shanghai_time = TimezoneConverter::now(8);
    /// let utc_time = TimezoneConverter::to_utc(shanghai_time);
    /// ```
    pub fn to_utc(local_time: DateTime<FixedOffset>) -> DateTime<Utc> {
        local_time.with_timezone(&Utc)
    }

    /// 获取当前时间（指定时区）
    ///
    /// # 参数
    /// * `offset_hours` - 时区的小时偏移
    ///
    /// # 返回
    /// * 当前时间（指定时区）
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::TimezoneConverter;
    ///
    /// let now_shanghai = TimezoneConverter::now(8);
    /// let now_tokyo = TimezoneConverter::now(9);
    /// let now_utc = TimezoneConverter::now(0);
    /// ```
    pub fn now(offset_hours: i32) -> DateTime<FixedOffset> {
        Utc::now().with_timezone(&Self::timezone(offset_hours))
    }

    /// 格式化为指定时区的字符串（RFC 3339 格式）
    ///
    /// # 参数
    /// * `utc_time` - UTC 时间
    /// * `offset_hours` - 目标时区的小时偏移
    ///
    /// # 返回
    /// * 格式化的字符串，例如 "2024-01-01T12:00:00+08:00"
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::TimezoneConverter;
    /// use chrono::Utc;
    ///
    /// let utc_time = Utc::now();
    /// let formatted = TimezoneConverter::format(utc_time, 8);
    /// println!("上海时间: {}", formatted);
    /// ```
    pub fn format(utc_time: DateTime<Utc>, offset_hours: i32) -> String {
        Self::utc_to_offset(utc_time, offset_hours).to_rfc3339()
    }

    /// 在两个时区之间转换
    ///
    /// # 参数
    /// * `time` - 源时区的时间
    /// * `target_offset_hours` - 目标时区的小时偏移
    ///
    /// # 返回
    /// * 目标时区的时间
    ///
    /// # 示例
    /// ```rust
    /// use iyucode_core::utils::TimezoneConverter;
    ///
    /// let shanghai_time = TimezoneConverter::now(8);
    /// let tokyo_time = TimezoneConverter::convert(shanghai_time, 9);
    /// ```
    pub fn convert(time: DateTime<FixedOffset>, target_offset_hours: i32) -> DateTime<FixedOffset> {
        time.with_timezone(&Self::timezone(target_offset_hours))
    }
}

/// 常用时区常量
pub mod timezones {
    /// 中国标准时间（上海/北京）UTC+8
    pub const CHINA: i32 = 8;
    
    /// 日本标准时间（东京）UTC+9
    pub const JAPAN: i32 = 9;
    
    /// 韩国标准时间（首尔）UTC+9
    pub const KOREA: i32 = 9;
    
    /// 新加坡标准时间 UTC+8
    pub const SINGAPORE: i32 = 8;
    
    /// 印度标准时间 UTC+5:30 (注意：需要特殊处理半小时偏移)
    pub const INDIA: i32 = 5; // 简化为 +5，实际应该是 +5:30
    
    /// 英国标准时间（伦敦）UTC+0
    pub const UK: i32 = 0;
    
    /// 美国东部标准时间（纽约）UTC-5
    pub const US_EAST: i32 = -5;
    
    /// 美国西部标准时间（洛杉矶）UTC-8
    pub const US_WEST: i32 = -8;
    
    /// 澳大利亚东部标准时间（悉尼）UTC+10
    pub const AUSTRALIA_EAST: i32 = 10;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    #[test]
    fn test_utc_to_offset() {
        // 2024-01-01 00:00:00 UTC
        let utc_time = Utc.from_utc_datetime(
            &NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
        );
        
        let shanghai_time = TimezoneConverter::utc_to_offset(utc_time, 8);
        
        // 应该是 2024-01-01 08:00:00 +08:00
        assert_eq!(shanghai_time.hour(), 8);
        assert_eq!(shanghai_time.offset().local_minus_utc(), 8 * 3600);
    }

    #[test]
    fn test_to_utc() {
        let shanghai_tz = TimezoneConverter::timezone(8);
        
        // 2024-01-01 08:00:00 +08:00
        let shanghai_time = shanghai_tz.from_local_datetime(
            &NaiveDateTime::parse_from_str("2024-01-01 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
        ).unwrap();
        
        let utc_time = TimezoneConverter::to_utc(shanghai_time);
        
        // 应该是 2024-01-01 00:00:00 UTC
        assert_eq!(utc_time.hour(), 0);
    }

    #[test]
    fn test_now() {
        let now = TimezoneConverter::now(8);
        assert_eq!(now.offset().local_minus_utc(), 8 * 3600);
    }

    #[test]
    fn test_convert() {
        let shanghai_time = TimezoneConverter::now(8);
        let tokyo_time = TimezoneConverter::convert(shanghai_time, 9);
        
        // 东京时间应该比上海时间早 1 小时
        assert_eq!(tokyo_time.hour(), (shanghai_time.hour() + 1) % 24);
    }

    #[test]
    fn test_negative_offset() {
        let utc_time = Utc.from_utc_datetime(
            &NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
        );
        
        let ny_time = TimezoneConverter::utc_to_offset(utc_time, -5);
        
        // 应该是 2024-01-01 07:00:00 -05:00
        assert_eq!(ny_time.hour(), 7);
        assert_eq!(ny_time.offset().local_minus_utc(), -5 * 3600);
    }
}
