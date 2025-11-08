//! System configuration for runtime settings

use serde::{Deserialize, Serialize};

/// System configuration including server aid and runtime user/group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub server_name: String,
    pub location_tag: String,
    pub run_user: String,
    pub run_group: String,
    pub turn_realm: Option<String>,
}
