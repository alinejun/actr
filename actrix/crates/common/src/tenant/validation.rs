//! 租户验证逻辑
//!
//! 包含租户相关的业务规则验证和检查

use chrono::Utc;

use super::model::Tenant;

/// 租户验证相关实现
impl Tenant {
    /// 检查租户是否存在
    pub async fn exists(tenant_id: &str, key_id: &str) -> bool {
        Self::get_by_tenant_key_id_service(tenant_id, key_id)
            .await
            .unwrap_or(None)
            .is_some()
    }

    /// 验证密钥
    pub fn verify_secret_key(&self, secret_key: &Vec<u8>) -> bool {
        self.secret_key == *secret_key
    }

    /// 检查是否过期（用于 Turn 服务）
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at < Utc::now().timestamp()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expiration_check() {
        let past_time = Utc::now().timestamp() - 3600; // 1 hour ago
        let mut tenant = Tenant::new(
            "expired_tenant".to_string(),
            "expired_key_id".to_string(),
            b"expired_public".to_vec(),
            b"expired_secret".to_vec(),
            "Expired App".to_string(),
        );

        // Set expired time to test expiration
        tenant.expires_at = Some(past_time);
        assert!(tenant.is_expired());

        // Test non-expiring tenant
        tenant.expires_at = None;
        assert!(!tenant.is_expired());
    }

    #[test]
    fn test_verify_secret_key() {
        let tenant = Tenant::new(
            "test".to_string(),
            "test".to_string(),
            b"correct_public".to_vec(),
            b"correct_secret".to_vec(),
            "test_name".to_string(),
        );

        assert!(tenant.verify_secret_key(&b"correct_secret".to_vec()));
        assert!(!tenant.verify_secret_key(&b"wrong_secret".to_vec()));
    }
}
