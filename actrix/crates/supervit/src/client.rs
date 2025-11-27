//! gRPC client for supervisor communication

use crate::config::SupervitConfig;
use crate::error::{Result, SupervitError};
use crate::generated::supervisor_client::SupervisorClient as GrpcSupervisorClient;
use crate::generated::{
    HealthCheckRequest, HealthCheckResponse, StatusReport, TenantOperation, TenantOperationResponse,
};
use crate::metrics::{collect_service_status, collect_system_metrics};
use crate::nonce_auth::generate_credential;
use std::time::Duration;
use tokio::time::interval;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity};
use tracing::{debug, error, info, warn};

/// Supervit gRPC 客户端
pub struct SupervitClient {
    config: SupervitConfig,
    client: Option<GrpcSupervisorClient<Channel>>,
    shared_secret: Vec<u8>, // hex 解码后的共享密钥
}

impl SupervitClient {
    /// 创建新的 supervit 客户端
    pub fn new(config: SupervitConfig) -> Result<Self> {
        config.validate()?;

        // 解码共享密钥
        let shared_secret = if let Some(ref secret_hex) = config.shared_secret {
            hex::decode(secret_hex)
                .map_err(|e| SupervitError::Config(format!("Invalid shared_secret hex: {e}")))?
        } else {
            return Err(SupervitError::Config(
                "shared_secret is required for authentication".to_string(),
            ));
        };

        Ok(Self {
            config,
            client: None,
            shared_secret,
        })
    }

    /// 连接到 supervisor 服务器
    pub async fn connect(&mut self) -> Result<()> {
        info!(
            "Connecting to supervisor at: {} (node: {})",
            self.config.server_addr, self.config.node_id
        );

        let mut endpoint = Endpoint::from_shared(self.config.server_addr.clone())
            .map_err(|e| SupervitError::Config(format!("Invalid server address: {e}")))?
            .timeout(Duration::from_secs(self.config.connect_timeout_secs))
            .connect_timeout(Duration::from_secs(self.config.connect_timeout_secs));

        // 如果启用 TLS，配置 TLS/mTLS
        if self.config.enable_tls {
            let tls_config = self.build_tls_config()?;
            endpoint = endpoint
                .tls_config(tls_config)
                .map_err(|e| SupervitError::Config(format!("TLS configuration error: {e}")))?;
            info!("TLS enabled for connection");
        }

        let channel = endpoint.connect().await?;
        self.client = Some(GrpcSupervisorClient::new(channel));

        info!("Successfully connected to supervisor");
        Ok(())
    }

    /// 构建 TLS 配置（支持 mTLS）
    fn build_tls_config(&self) -> Result<ClientTlsConfig> {
        // TLS 域名是必需的
        let tls_domain = self.config.tls_domain.as_ref().ok_or_else(|| {
            SupervitError::Config("tls_domain is required when enable_tls is true".to_string())
        })?;

        let mut tls_config = ClientTlsConfig::new().domain_name(tls_domain);

        debug!("Configuring TLS with domain: {}", tls_domain);

        // 加载 CA 证书（验证服务端证书）
        if let Some(ca_cert_path) = &self.config.ca_cert {
            debug!("Loading CA certificate from: {}", ca_cert_path);
            let ca_cert_pem = std::fs::read(ca_cert_path).map_err(|e| {
                SupervitError::Config(format!(
                    "Failed to read CA certificate from {ca_cert_path}: {e}"
                ))
            })?;

            let ca_cert = Certificate::from_pem(ca_cert_pem);
            tls_config = tls_config.ca_certificate(ca_cert);
            info!("CA certificate loaded for server verification");
        }

        // 加载客户端证书和私钥（mTLS）
        if let (Some(cert_path), Some(key_path)) =
            (&self.config.client_cert, &self.config.client_key)
        {
            debug!("Loading client certificate from: {}", cert_path);
            debug!("Loading client private key from: {}", key_path);

            let client_cert_pem = std::fs::read(cert_path).map_err(|e| {
                SupervitError::Config(format!(
                    "Failed to read client certificate from {cert_path}: {e}"
                ))
            })?;

            let client_key_pem = std::fs::read(key_path).map_err(|e| {
                SupervitError::Config(format!(
                    "Failed to read client private key from {key_path}: {e}"
                ))
            })?;

            let identity = Identity::from_pem(client_cert_pem, client_key_pem);
            tls_config = tls_config.identity(identity);
            info!("mTLS enabled: client certificate and key loaded");
        } else if self.config.client_cert.is_some() || self.config.client_key.is_some() {
            // 如果只配置了证书或私钥其中一个，报错
            return Err(SupervitError::Config(
                "Both client_cert and client_key must be provided for mTLS".to_string(),
            ));
        }

        Ok(tls_config)
    }

    /// 启动状态上报循环
    pub async fn start_status_reporting(&mut self) -> Result<()> {
        let client = self
            .client
            .as_mut()
            .ok_or(SupervitError::ConnectionClosed)?;

        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let stream = ReceiverStream::new(rx);

        // 启动双向流
        let mut response_stream = client.stream_status(stream).await?.into_inner();

        // 克隆配置用于后台任务
        let node_id = self.config.node_id.clone();
        let interval_secs = self.config.status_report_interval_secs;
        let location_tag = std::env::var("LOCATION_TAG").unwrap_or_else(|_| "unknown".to_string());
        let name = std::env::var("NODE_NAME").unwrap_or_else(|_| node_id.clone());
        let shared_secret = self.shared_secret.clone();

        // 启动状态上报任务
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));

            loop {
                ticker.tick().await;

                match Self::create_status_report(&node_id, &location_tag, &name, &shared_secret)
                    .await
                {
                    Ok(report) => {
                        debug!("Sending status report for node: {}", node_id);
                        if let Err(e) = tx.send(report).await {
                            error!("Failed to send status report: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to create status report: {}", e);
                    }
                }
            }
        });

        // 启动响应处理任务
        tokio::spawn(async move {
            while let Some(result) = response_stream.next().await {
                match result {
                    Ok(ack) => {
                        debug!("Received status ack for node: {}", ack.node_id);
                    }
                    Err(e) => {
                        error!("Status stream error: {}", e);
                        break;
                    }
                }
            }
            info!("Status reporting stream closed");
        });

        Ok(())
    }

    /// 执行健康检查
    pub async fn health_check(&mut self) -> Result<HealthCheckResponse> {
        let client = self
            .client
            .as_mut()
            .ok_or(SupervitError::ConnectionClosed)?;

        // 构造请求负载（用于签名）
        let payload = format!("health_check:{}", self.config.node_id);

        // 生成认证凭证
        let credential = generate_credential(&self.shared_secret, payload.as_bytes())?;

        let request = HealthCheckRequest {
            node_id: self.config.node_id.clone(),
            credential,
        };

        debug!("Sending health check request with nonce-auth credential");

        let response = client.health_check(request).await?.into_inner();

        debug!(
            "Health check successful, latency: {}ms",
            response.latency_ms
        );

        Ok(response)
    }

    /// 执行租户操作
    pub async fn manage_tenant(
        &mut self,
        mut operation: TenantOperation,
    ) -> Result<TenantOperationResponse> {
        let client = self
            .client
            .as_mut()
            .ok_or(SupervitError::ConnectionClosed)?;

        // 构造请求负载（包含操作类型和租户 ID）
        let payload = format!(
            "manage_tenant:{:?}:{}",
            operation.operation, operation.tenant_id
        );

        // 生成认证凭证
        let credential = generate_credential(&self.shared_secret, payload.as_bytes())?;
        operation.credential = credential;

        let response = client.manage_tenant(operation).await?.into_inner();

        Ok(response)
    }

    /// 创建状态报告（带认证凭证）
    async fn create_status_report(
        node_id: &str,
        location_tag: &str,
        name: &str,
        shared_secret: &[u8],
    ) -> Result<StatusReport> {
        let metrics = collect_system_metrics().await?;
        let services = collect_service_status();

        // 获取 power_reserve_level（0-5 级别）
        let power_reserve_level =
            pwrzv::get_power_reserve_level_direct().await.unwrap_or(0.0) as u32;

        let timestamp = chrono::Utc::now().timestamp();

        // 构造请求负载
        let payload = format!("status_report:{node_id}:{timestamp}");

        // 生成认证凭证
        let credential = generate_credential(shared_secret, payload.as_bytes())?;

        Ok(StatusReport {
            node_id: node_id.to_string(),
            timestamp,
            metrics: Some(metrics),
            services,
            location_tag: location_tag.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            power_reserve_level,
            name: name.to_string(),
            credential,
        })
    }

    /// 断开连接
    pub fn disconnect(&mut self) {
        self.client = None;
        info!("Disconnected from supervisor");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = SupervitConfig {
            node_id: "test-node".to_string(),
            server_addr: "http://localhost:50051".to_string(),
            shared_secret: Some(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            ),
            ..Default::default()
        };

        let client = SupervitClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_invalid_config() {
        let config = SupervitConfig {
            node_id: String::new(),
            ..Default::default()
        };

        let client = SupervitClient::new(config);
        assert!(client.is_err());
    }

    #[tokio::test]
    async fn test_create_status_report() {
        let secret =
            hex::decode("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap();
        let report = SupervitClient::create_status_report(
            "test-node",
            "test-location",
            "test-name",
            &secret,
        )
        .await;
        assert!(report.is_ok());
        let report = report.unwrap();
        assert_eq!(report.node_id, "test-node");
        assert_eq!(report.location_tag, "test-location");
        assert_eq!(report.name, "test-name");
        // proto2 required 字段不是 Option
        assert!(report.credential.timestamp > 0);
        assert!(!report.credential.nonce.is_empty());
        assert!(!report.credential.signature.is_empty());
    }

    #[test]
    fn test_tls_config_validation() {
        // 测试：启用 TLS 但未提供 tls_domain 应该失败
        let config = SupervitConfig {
            node_id: "test-node".to_string(),
            server_addr: "https://localhost:50051".to_string(),
            shared_secret: Some(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            ),
            enable_tls: true,
            tls_domain: None, // 缺少 tls_domain
            ..Default::default()
        };

        let result = SupervitClient::new(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_tls_config_with_domain() {
        // 测试：启用 TLS 并提供 tls_domain 应该成功
        let config = SupervitConfig {
            node_id: "test-node".to_string(),
            server_addr: "https://localhost:50051".to_string(),
            shared_secret: Some(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            ),
            enable_tls: true,
            tls_domain: Some("localhost".to_string()),
            ..Default::default()
        };

        let client = SupervitClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_mtls_partial_config_error() {
        // 测试：只配置客户端证书但不配置私钥应该在验证阶段失败
        let config = SupervitConfig {
            node_id: "test-node".to_string(),
            server_addr: "https://localhost:50051".to_string(),
            shared_secret: Some(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            ),
            enable_tls: true,
            tls_domain: Some("localhost".to_string()),
            client_cert: Some("/path/to/cert.pem".to_string()),
            client_key: None, // 缺少私钥
            ..Default::default()
        };

        // validate() 会检查 mTLS 配置完整性，应该失败
        let client = SupervitClient::new(config);
        assert!(client.is_err());
    }

    #[test]
    fn test_tls_disabled() {
        // 测试：不启用 TLS 应该可以正常创建客户端
        let config = SupervitConfig {
            node_id: "test-node".to_string(),
            server_addr: "http://localhost:50051".to_string(),
            shared_secret: Some(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            ),
            enable_tls: false,
            ..Default::default()
        };

        let client = SupervitClient::new(config);
        assert!(client.is_ok());
    }
}
