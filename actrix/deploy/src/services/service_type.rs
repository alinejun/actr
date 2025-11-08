//! Service type enumeration for actrix services

use serde::{Deserialize, Serialize};
use std::fmt;

/// Available service types with their bit values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceType {
    Signaling = 1,
    Stun = 2,
    Turn = 4,
    Ais = 8,
}

impl ServiceType {
    pub fn all() -> Vec<Self> {
        vec![
            ServiceType::Signaling,
            ServiceType::Stun,
            ServiceType::Turn,
            ServiceType::Ais,
        ]
    }

    pub fn description(&self) -> &'static str {
        match self {
            ServiceType::Signaling => "WebSocket signaling service",
            ServiceType::Stun => "STUN server for NAT traversal",
            ServiceType::Turn => "TURN relay server (includes STUN)",
            ServiceType::Ais => "ActorRTC Identity Service",
        }
    }

    pub fn needs_http(&self) -> bool {
        matches!(self, ServiceType::Signaling | ServiceType::Ais)
    }

    pub fn needs_ice(&self) -> bool {
        matches!(self, ServiceType::Stun | ServiceType::Turn)
    }
}

impl fmt::Display for ServiceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceType::Signaling => write!(f, "Signaling"),
            ServiceType::Stun => write!(f, "STUN"),
            ServiceType::Turn => write!(f, "TURN"),
            ServiceType::Ais => write!(f, "Ais"),
        }
    }
}
