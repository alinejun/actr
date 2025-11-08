//! Token 数据结构
//!
//! 用于 TURN 认证的明文 Token 结构，包含用户身份信息和预共享密钥

use actr_protocol::ActrId;
use serde::{Deserialize, Serialize};

/// Token 明文结构
///
/// 此结构从 Claims 的加密 token 字段中解密得到，包含 TURN 认证所需的 PSK
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// 过期时间 (Unix timestamp)
    pub exp: Option<u64>,

    /// 租户 ID
    pub tenant: String,

    /// ActorId (可选)
    pub id: Option<ActrId>,

    /// Actor 类型
    pub act_type: String,

    /// 预共享密钥 (hex 编码)
    pub psk: String,

    /// 可选设备指纹
    pub device_fingerprint: Option<String>,
}

impl Token {
    /// 创建新的 Token
    pub fn new(tenant: String, act_type: String, psk: String, exp: Option<u64>) -> Self {
        Self {
            exp,
            tenant,
            id: None,
            act_type,
            psk,
            device_fingerprint: None,
        }
    }

    /// 检查 Token 是否过期
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.exp {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now > exp
        } else {
            false // 没有过期时间，永不过期
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_token_creation() {
        let token = Token::new(
            "test_tenant".to_string(),
            "user".to_string(),
            "abc123".to_string(),
            Some(1730614800),
        );

        assert_eq!(token.tenant, "test_tenant");
        assert_eq!(token.act_type, "user");
        assert_eq!(token.psk, "abc123");
        assert_eq!(token.exp, Some(1730614800));
    }

    #[test]
    fn test_token_expiration() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 未过期
        let valid_token = Token::new(
            "tenant".to_string(),
            "user".to_string(),
            "psk".to_string(),
            Some(now + 3600),
        );
        assert!(!valid_token.is_expired());

        // 已过期
        let expired_token = Token::new(
            "tenant".to_string(),
            "user".to_string(),
            "psk".to_string(),
            Some(now - 1),
        );
        assert!(expired_token.is_expired());

        // 无过期时间
        let no_exp_token = Token::new(
            "tenant".to_string(),
            "user".to_string(),
            "psk".to_string(),
            None,
        );
        assert!(!no_exp_token.is_expired());
    }
}
