use serde::{Deserialize, Serialize};

/// Supervisor 平台集成配置
///
/// 用于与 Supervisor 管理平台集成的配置信息（gRPC 模式）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SupervisorConfig {
    /// 节点唯一标识符
    ///
    /// 在 Supervisor 平台中的唯一标识符，用于识别此服务实例。
    pub node_id: String,

    /// Supervisor gRPC 服务器地址
    ///
    /// gRPC 服务器地址，格式：http://hostname:port 或 https://hostname:port
    /// 示例：http://supervisor.example.com:50051
    pub server_addr: String,

    /// 连接超时（秒）
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_secs: u64,

    /// 状态上报间隔（秒）
    #[serde(default = "default_status_interval")]
    pub status_report_interval_secs: u64,

    /// 健康检查间隔（秒）
    #[serde(default = "default_health_check_interval")]
    pub health_check_interval_secs: u64,

    /// 是否启用 TLS
    #[serde(default)]
    pub enable_tls: bool,

    /// TLS 域名（用于证书验证）
    pub tls_domain: Option<String>,
}

fn default_connect_timeout() -> u64 {
    30
}

fn default_status_interval() -> u64 {
    60
}

fn default_health_check_interval() -> u64 {
    30
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            node_id: String::new(),
            server_addr: "http://localhost:50051".to_string(),
            connect_timeout_secs: default_connect_timeout(),
            status_report_interval_secs: default_status_interval(),
            health_check_interval_secs: default_health_check_interval(),
            enable_tls: false,
            tls_domain: None,
        }
    }
}
