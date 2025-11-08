//! Main deployment configuration combining all settings

// Configuration Wizard

use super::{NetworkConfig, SslConfig, SystemConfig};
use crate::services::ServiceSelection;

/// Complete deployment configuration for actor-rtc-actrix services
#[derive(Debug, Clone)]
pub struct DeploymentConfig {
    pub services: ServiceSelection,
    pub network: NetworkConfig,
    pub ssl: Option<SslConfig>,
    pub system: SystemConfig,
    // pub install: InstallConfig,
}
