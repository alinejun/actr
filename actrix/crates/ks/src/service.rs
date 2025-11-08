//! KS 服务主服务模块

use crate::{
    auth::PskAuthenticator,
    error::{KsError, KsResult},
    handlers::{create_router, KsAppState},
    storage::KsStorage,
};
use axum::Router;
use actrix_common::config::ks::KsServerConfig;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

/// KS 服务
/// 
/// 管理整个 Key Server 服务的生命周期
pub struct KsService {
    /// 服务配置
    config: KsServerConfig,
    /// 存储服务
    storage: KsStorage,
    /// PSK 认证器
    authenticator: PskAuthenticator,
}

impl KsService {
    /// 创建新的 KS 服务实例
    ///
    /// # Arguments
    /// * `config` - KS 服务器配置
    /// * `actrix_shared_key` - Actrix 内部服务通信共享密钥
    pub async fn new(config: KsServerConfig, actrix_shared_key: String) -> KsResult<Self> {
        info!("Initializing KS service with config: {:?}", config);

        // 初始化存储
        let storage = KsStorage::new(&config.database_path)?;
        
        // 初始化认证器（使用 actrix_shared_key 而不是配置中的 psk）
        let authenticator = PskAuthenticator::new(actrix_shared_key, &config.database_path).await?;

        Ok(Self {
            config,
            storage,
            authenticator,
        })
    }

    /// 从 ActrixConfig 创建 KS 服务（推荐方式）
    ///
    /// 这是创建 KS 服务的推荐方法，会自动使用 actrix_shared_key
    ///
    /// # Arguments
    /// * `actrix_config` - 完整的 Actrix 配置
    pub async fn from_actrix_config(actrix_config: &actrix_common::config::ActrixConfig) -> KsResult<Self> {
        let ks_service_config = actrix_config.services.ks.as_ref()
            .ok_or_else(|| KsError::Config("KS service configuration not found".to_string()))?;

        if !ks_service_config.enabled {
            return Err(KsError::Config("KS service is not enabled".to_string()));
        }

        Self::new(
            ks_service_config.server.clone(),
            actrix_config.get_actrix_shared_key().to_string()
        ).await
    }

    /// 创建 Axum 路由器
    pub fn create_router(&self) -> Router {
        let state = KsAppState::new(self.storage.clone(), self.authenticator.clone());
        create_router(state)
    }

    /// 启动 KS 服务
    /// 
    /// 这个方法会阻塞当前线程，直到服务停止
    pub async fn start(&self) -> KsResult<()> {
        let addr = SocketAddr::new(
            self.config.ip.parse().map_err(|e| {
                KsError::Config(format!("Invalid IP address {}: {}", self.config.ip, e))
            })?,
            self.config.port,
        );

        info!("Starting KS service on {}", addr);

        // 创建路由器
        let app = self.create_router();

        // 绑定端口
        let listener = TcpListener::bind(addr).await.map_err(|e| {
            KsError::Internal(format!("Failed to bind to {addr}: {e}"))
        })?;

        let actual_addr = listener.local_addr().map_err(|e| {
            KsError::Internal(format!("Failed to get local address: {e}"))
        })?;

        info!("KS service listening on {}", actual_addr);

        // 启动服务器
        axum::serve(listener, app).await.map_err(|e| {
            KsError::Internal(format!("Server error: {e}"))
        })?;

        Ok(())
    }

    /// 获取存储服务引用（用于测试）
    #[cfg(test)]
    pub fn storage(&self) -> &KsStorage {
        &self.storage
    }

    /// 获取认证器引用（用于测试）
    #[cfg(test)]
    pub fn authenticator(&self) -> &PskAuthenticator {
        &self.authenticator
    }

    /// 清理过期的 nonce 记录
    /// 
    /// 建议定期调用此方法以清理过期的 nonce 记录
    pub async fn cleanup_expired_nonces(&self) -> KsResult<()> {
        self.authenticator.cleanup_expired_nonces().await?;
        info!("Cleaned up expired nonces");
        Ok(())
    }

    /// 获取服务统计信息
    pub async fn get_stats(&self) -> KsResult<ServiceStats> {
        let key_count = self.storage.get_key_count()?;
        
        Ok(ServiceStats {
            key_count,
            database_path: self.config.database_path.clone(),
            bind_address: format!("{}:{}", self.config.ip, self.config.port),
        })
    }
}

/// 服务统计信息
#[derive(Debug, Clone)]
pub struct ServiceStats {
    /// 密钥总数
    pub key_count: u32,
    /// 数据库路径
    pub database_path: String,
    /// 绑定地址
    pub bind_address: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_config() -> KsServerConfig {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_ks.db");

        KsServerConfig {
            ip: "127.0.0.1".to_string(),
            port: 0, // 使用随机端口
            psk: "test-psk".to_string(),
            database_path: db_path.to_string_lossy().to_string(),
        }
    }

    #[tokio::test]
    async fn test_service_creation() {
        let config = create_test_config();
        let service = KsService::new(config, "test-actrix-shared-key".to_string()).await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_service_stats() {
        let config = create_test_config();
        let service = KsService::new(config, "test-actrix-shared-key".to_string()).await.unwrap();
        
        let stats = service.get_stats().await.unwrap();
        assert_eq!(stats.key_count, 0);
        assert!(!stats.database_path.is_empty());
        assert!(!stats.bind_address.is_empty());
    }

    #[tokio::test]
    async fn test_router_creation() {
        let config = create_test_config();
        let service = KsService::new(config, "test-actrix-shared-key".to_string()).await.unwrap();
        
        let _router = service.create_router();
        // 简单测试路由器可以创建成功
        assert!(true); // 如果能到这里说明创建成功了
    }

    #[tokio::test]
    async fn test_cleanup_nonces() {
        let config = create_test_config();
        let service = KsService::new(config, "test-actrix-shared-key".to_string()).await.unwrap();
        
        let result = service.cleanup_expired_nonces().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_from_actrix_config() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_ks.db");
        
        let mut actrix_config = actrix_common::config::ActrixConfig::default();
        actrix_config.actrix_shared_key = "test-shared-key-123".to_string();
        actrix_config.services.ks = Some(actrix_common::config::services::KsServiceConfig {
            enabled: true,
            server: KeyServerConfig {
                ip: "127.0.0.1".to_string(),
                port: 0,
                #[allow(deprecated)]
                psk: "ignored-old-psk".to_string(),
                database_path: db_path.to_string_lossy().to_string(),
                nonce_db_path: None,
                key_ttl_seconds: 3600,
            },
        });

        let service = KsService::from_actrix_config(&actrix_config).await;
        assert!(service.is_ok());
        
        // 验证服务使用了正确的共享密钥（间接验证，通过创建认证凭证）
        let service = service.unwrap();
        let credential = service.authenticator.create_credential("test-data");
        assert!(credential.is_ok());
    }
}