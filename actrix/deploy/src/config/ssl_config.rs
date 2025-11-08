//! SSL certificate configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// SSL certificate configuration for HTTPS services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    pub domain_name: String,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}
