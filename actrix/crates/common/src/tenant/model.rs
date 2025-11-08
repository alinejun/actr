//! 租户核心数据结构
//!
//! 定义租户实体的核心数据结构和基础方法

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 租户结构体
///
/// 租户是用于分离不同应用程序资源的虚拟概念。
///
/// ## 字段说明
/// - `name`: 租户名称，在一个宇宙中是唯一的
/// - `key_id`: 租户密钥 ID
/// - `app_id`: 应用 ID
/// - `secret_key`: 认证私钥，用于租户级别的 Token 认证等
/// - `public_key`: 认证公钥，用于租户级别的 Token 认证等，用于加密 credential
///
/// 统一租户表结构
///
/// 合并了 TenantForAuthority、TenantForSignaling 和 TenantForTurn 的所有字段
#[derive(Debug, Clone, Serialize, Deserialize, Default, FromRow)]
pub struct Tenant {
    pub rowid: Option<i64>,

    // 基础字段 - 所有服务都需要
    pub tenant_id: String,   // 租户ID（应用ID）
    pub key_id: String,      // 密钥ID
    pub secret_key: Vec<u8>, // 私钥

    // 可选字段 - 部分服务需要
    pub name: String,            // 应用名称 (Authority 服务需要)
    pub public_key: Vec<u8>,     // 公钥 (Authority 服务需要)
    pub expires_at: Option<i64>, // 过期时间戳

    // 元数据字段
    pub created_at: Option<i64>, // 创建时间
    pub updated_at: Option<i64>, // 更新时间
}

impl Tenant {
    /// 创建新的统一租户实例
    pub fn new(
        tenant_id: String,
        key_id: String,
        public_key: Vec<u8>,
        secret_key: Vec<u8>,
        name: String,
    ) -> Self {
        let now = Utc::now().timestamp();
        Self {
            rowid: None,
            tenant_id,
            key_id,
            secret_key,
            name,
            public_key,
            expires_at: None,
            created_at: Some(now),
            updated_at: Some(now),
        }
    }

    // Getter methods for accessing private fields
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn public_key(&self) -> &Vec<u8> {
        &self.public_key
    }

    pub fn secret_key(&self) -> &Vec<u8> {
        &self.secret_key
    }

    // Setter methods for admin operations
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_public_key(&mut self, public_key: Vec<u8>) {
        self.public_key = public_key;
    }

    pub fn set_secret_key(&mut self, secret_key: Vec<u8>) {
        self.secret_key = secret_key;
    }

    // Add tenant_id field for compatibility
    pub fn tenant_id(&self) -> String {
        self.tenant_id.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_creation() {
        let tenant = Tenant::new(
            "test_tenant".to_string(),
            "test_key_id".to_string(),
            b"test_public".to_vec(),
            b"test_secret".to_vec(),
            "test_name".to_string(),
        );

        assert_eq!(tenant.tenant_id, "test_tenant");
        assert_eq!(tenant.key_id, "test_key_id");
        assert_eq!(tenant.secret_key, b"test_secret".to_vec());
        assert_eq!(tenant.public_key, b"test_public".to_vec());
        assert_eq!(tenant.name, "test_name");
        assert!(tenant.created_at.is_some());
        assert!(tenant.updated_at.is_some());
    }

    #[test]
    fn test_authority_tenant_creation() {
        let tenant = Tenant::new(
            "auth_tenant".to_string(),
            "auth_key_id".to_string(),
            b"auth_public".to_vec(),
            b"auth_secret".to_vec(),
            "Auth App".to_string(),
        );

        assert_eq!(tenant.name, "Auth App");
        assert_eq!(tenant.public_key, b"auth_public".to_vec());
    }
}
