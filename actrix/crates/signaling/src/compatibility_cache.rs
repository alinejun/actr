//! 全局兼容性缓存 - Demo版本
//!
//! 在信令服务器内部维护一个简单的内存缓存，存储兼容性检查结果

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};

/// 兼容性缓存条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityCacheEntry {
    /// 兼容性结果 ("compatible", "backward_compatible", "incompatible")
    pub result: String,
    /// 缓存时间
    pub cached_at: SystemTime,
    /// 过期时间
    pub expires_at: SystemTime,
    /// 上报者数量（简单统计）
    pub reporter_count: u32,
}

/// 兼容性上报请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    /// 源指纹
    pub from_fingerprint: String,
    /// 目标指纹
    pub to_fingerprint: String,
    /// 服务类型
    pub service_type: String,
    /// 兼容性结果
    pub result: String,
}

/// 兼容性缓存查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityCacheQuery {
    /// 缓存键
    pub cache_key: String,
}

/// 兼容性缓存响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityCacheResponse {
    /// 缓存键
    pub cache_key: String,
    /// 缓存值（如果存在）
    pub result: Option<String>,
    /// 是否命中缓存
    pub hit: bool,
}

/// 全局兼容性缓存管理器（Demo版本）
#[derive(Debug)]
pub struct GlobalCompatibilityCache {
    /// 内存缓存 (cache_key -> entry)
    cache: HashMap<String, CompatibilityCacheEntry>,
    /// 最大缓存条目数
    max_entries: usize,
    /// 默认TTL（24小时）
    default_ttl: Duration,
}

impl GlobalCompatibilityCache {
    /// 创建新的缓存管理器
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_entries: 10000,                          // demo版本限制1万条
            default_ttl: Duration::from_secs(24 * 3600), // 24小时
        }
    }

    /// 构建缓存键
    pub fn build_cache_key(
        service_type: &str,
        from_fingerprint: &str,
        to_fingerprint: &str,
    ) -> String {
        format!("{service_type}:{from_fingerprint}:{to_fingerprint}")
    }

    /// 查询兼容性缓存
    pub fn query(&self, cache_key: &str) -> CompatibilityCacheResponse {
        if let Some(entry) = self.cache.get(cache_key) {
            // 检查是否过期
            if SystemTime::now() <= entry.expires_at {
                debug!("兼容性缓存命中: {}", cache_key);
                return CompatibilityCacheResponse {
                    cache_key: cache_key.to_string(),
                    result: Some(entry.result.clone()),
                    hit: true,
                };
            } else {
                debug!("兼容性缓存过期: {}", cache_key);
            }
        }

        debug!("兼容性缓存未命中: {}", cache_key);
        CompatibilityCacheResponse {
            cache_key: cache_key.to_string(),
            result: None,
            hit: false,
        }
    }

    /// 上报兼容性结果
    pub fn report(&mut self, report: CompatibilityReport) {
        let cache_key = Self::build_cache_key(
            &report.service_type,
            &report.from_fingerprint,
            &report.to_fingerprint,
        );

        let now = SystemTime::now();
        let expires_at = now + self.default_ttl;

        // 检查是否已存在
        if let Some(existing) = self.cache.get_mut(&cache_key) {
            // 更新现有条目
            existing.result = report.result.clone();
            existing.cached_at = now;
            existing.expires_at = expires_at;
            existing.reporter_count += 1;
            debug!(
                "更新兼容性缓存: {} (上报次数: {})",
                cache_key, existing.reporter_count
            );
        } else {
            // 创建新条目
            let entry = CompatibilityCacheEntry {
                result: report.result.clone(),
                cached_at: now,
                expires_at,
                reporter_count: 1,
            };

            // 检查缓存大小限制
            if self.cache.len() >= self.max_entries {
                self.cleanup_oldest_entries();
            }

            self.cache.insert(cache_key.clone(), entry);
            debug!("新增兼容性缓存: {}", cache_key);
        }

        info!(
            "收到兼容性上报: {} -> {} = {}",
            report.from_fingerprint, report.to_fingerprint, report.result
        );
    }

    /// 清理过期缓存
    pub fn cleanup_expired(&mut self) -> usize {
        let now = SystemTime::now();
        let initial_count = self.cache.len();

        self.cache.retain(|_key, entry| now <= entry.expires_at);

        let removed_count = initial_count - self.cache.len();
        if removed_count > 0 {
            info!("清理了 {} 个过期的兼容性缓存条目", removed_count);
        }

        removed_count
    }

    /// 清理最旧的缓存条目（当达到大小限制时）
    fn cleanup_oldest_entries(&mut self) {
        if self.cache.is_empty() {
            return;
        }

        // 简单策略：删除25%的最旧条目
        let mut entries: Vec<_> = self
            .cache
            .iter()
            .map(|(k, v)| (k.clone(), v.cached_at))
            .collect();
        entries.sort_by_key(|(_, cached_at)| *cached_at);

        let remove_count = (self.cache.len() / 4).max(1);
        for (key, _) in entries.iter().take(remove_count.min(entries.len())) {
            self.cache.remove(key);
        }

        warn!(
            "缓存大小限制，清理了 {} 个最旧的兼容性缓存条目",
            remove_count
        );
    }

    /// 获取缓存统计信息
    pub fn stats(&self) -> CacheStats {
        let now = SystemTime::now();
        let mut expired_count = 0;
        let mut total_reports = 0;

        for entry in self.cache.values() {
            if now > entry.expires_at {
                expired_count += 1;
            }
            total_reports += entry.reporter_count;
        }

        CacheStats {
            total_entries: self.cache.len(),
            expired_entries: expired_count,
            total_reports,
            max_entries: self.max_entries,
        }
    }

    /// 清空所有缓存（用于测试）
    #[cfg(test)]
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

/// 缓存统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// 总缓存条目数
    pub total_entries: usize,
    /// 过期条目数
    pub expired_entries: usize,
    /// 总上报次数
    pub total_reports: u32,
    /// 最大条目数限制
    pub max_entries: usize,
}

impl Default for GlobalCompatibilityCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let key = GlobalCompatibilityCache::build_cache_key(
            "user-service",
            "sha256:old123",
            "sha256:new456",
        );
        assert_eq!(key, "user-service:sha256:old123:sha256:new456");
    }

    #[test]
    fn test_basic_cache_operations() {
        let mut cache = GlobalCompatibilityCache::new();

        // 查询不存在的条目
        let response = cache.query("nonexistent");
        assert!(!response.hit);
        assert!(response.result.is_none());

        // 上报兼容性结果
        let report = CompatibilityReport {
            from_fingerprint: "sha256:old".to_string(),
            to_fingerprint: "sha256:new".to_string(),
            service_type: "test-service".to_string(),
            result: "compatible".to_string(),
        };

        cache.report(report);

        // 查询应该命中
        let cache_key =
            GlobalCompatibilityCache::build_cache_key("test-service", "sha256:old", "sha256:new");
        let response = cache.query(&cache_key);
        assert!(response.hit);
        assert_eq!(response.result.unwrap(), "compatible");

        // 统计信息
        let stats = cache.stats();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.total_reports, 1);
    }

    #[test]
    fn test_cache_expiration() {
        let mut cache = GlobalCompatibilityCache::new();

        // 创建一个已过期的条目
        let cache_key = "test:old:new".to_string();
        let expired_entry = CompatibilityCacheEntry {
            result: "compatible".to_string(),
            cached_at: SystemTime::now() - Duration::from_secs(1000),
            expires_at: SystemTime::now() - Duration::from_secs(1), // 已过期
            reporter_count: 1,
        };

        cache.cache.insert(cache_key.clone(), expired_entry);

        // 查询应该未命中（因为过期）
        let response = cache.query(&cache_key);
        assert!(!response.hit);

        // 清理过期条目
        let removed = cache.cleanup_expired();
        assert_eq!(removed, 1);
        assert_eq!(cache.cache.len(), 0);
    }

    #[test]
    fn test_cache_size_limit() {
        let mut cache = GlobalCompatibilityCache::new();
        cache.max_entries = 5; // 设置小的限制用于测试

        // 添加超过限制的条目
        for i in 0..10 {
            let report = CompatibilityReport {
                from_fingerprint: format!("sha256:old{i}"),
                to_fingerprint: format!("sha256:new{i}"),
                service_type: "test".to_string(),
                result: "compatible".to_string(),
            };
            cache.report(report);
        }

        // 缓存大小应该被限制
        assert!(cache.cache.len() <= cache.max_entries);
    }

    #[test]
    fn test_duplicate_reports() {
        let mut cache = GlobalCompatibilityCache::new();

        let report = CompatibilityReport {
            from_fingerprint: "sha256:old".to_string(),
            to_fingerprint: "sha256:new".to_string(),
            service_type: "test".to_string(),
            result: "compatible".to_string(),
        };

        // 上报两次相同的结果
        cache.report(report.clone());
        cache.report(report.clone());

        // 应该只有一个缓存条目，但上报次数为2
        assert_eq!(cache.cache.len(), 1);

        let cache_key =
            GlobalCompatibilityCache::build_cache_key("test", "sha256:old", "sha256:new");
        let entry = cache.cache.get(&cache_key).unwrap();
        assert_eq!(entry.reporter_count, 2);
    }
}
