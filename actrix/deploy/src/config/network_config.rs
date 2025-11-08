//! Network configuration for services

use serde::{Deserialize, Serialize};

/// Network configuration including server host and port settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub server_host: String,
    pub ice_port: u16,
    pub https_port: u16,
    pub http_port: u16,
}
