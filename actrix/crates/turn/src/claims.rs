//! TURN 认证 Claims 结构
//!
//! 用于 TURN 服务器认证的身份声明信息

use super::token::Token;
use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// TURN 认证 Claims
///
/// 此结构体包含加密的 token，会被 JSON 序列化后作为 username 传递给 TURN 服务器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// 租户 ID
    pub tid: String,

    /// 密钥 ID
    pub key_id: String,

    /// 加密的 token (base64 编码)
    #[serde(
        serialize_with = "serialize_base64",
        deserialize_with = "deserialize_base64"
    )]
    pub token: Vec<u8>,
}

/// 序列化为 base64 字符串
fn serialize_base64<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&STANDARD.encode(bytes))
}

/// 从 base64 字符串反序列化
fn deserialize_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    STANDARD.decode(s).map_err(serde::de::Error::custom)
}

impl Claims {
    /// 创建新的 Claims
    pub fn new(tid: String, key_id: String, token: Vec<u8>) -> Self {
        Self { tid, key_id, token }
    }

    /// 获取 token 信息
    ///
    /// 此方法会直接反序列化 self.token（不再使用 ECIES 解密）
    ///
    /// 注意：Token 不再加密传输，因为：
    /// - 加密后的数据太大，超过 STUN username 属性的 763 字节限制
    /// - PSK 是随机数据，本身不包含敏感信息
    /// - 网络层通过 TURN over TLS 保护
    pub fn get_token(&self) -> Result<Token> {
        // 直接反序列化为 Token 结构（token 是明文 JSON）
        let token: Token = serde_json::from_slice(&self.token)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize token: {e}"))?;

        // 验证 token 是否过期
        if token.is_expired() {
            return Err(anyhow::anyhow!("Token has expired"));
        }

        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_creation() {
        let token_bytes = b"test_token".to_vec();
        let claims = Claims::new(
            "tenant_123".to_string(),
            "key_456".to_string(),
            token_bytes.clone(),
        );

        assert_eq!(claims.tid, "tenant_123");
        assert_eq!(claims.key_id, "key_456");
        assert_eq!(claims.token, token_bytes);
    }

    #[test]
    fn test_claims_serialization() {
        let token_bytes = b"test_token".to_vec();
        let claims = Claims::new("tenant_123".to_string(), "key_456".to_string(), token_bytes);

        // 序列化
        let json = serde_json::to_string(&claims).unwrap();
        assert!(json.contains("tenant_123"));
        assert!(json.contains("key_456"));

        // 反序列化
        let decoded: Claims = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.tid, claims.tid);
        assert_eq!(decoded.key_id, claims.key_id);
        assert_eq!(decoded.token, claims.token);
    }

    #[test]
    fn test_base64_encoding() {
        let token_bytes = vec![0x01, 0x02, 0x03, 0x04];
        let claims = Claims::new("t1".to_string(), "k1".to_string(), token_bytes.clone());

        let json = serde_json::to_string(&claims).unwrap();

        // token 字段应该是 base64 编码的
        let expected_base64 = STANDARD.encode(&token_bytes);
        assert!(json.contains(&expected_base64));
    }
}
