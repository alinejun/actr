//! Supervit - gRPC-based supervisor client for actrix nodes
//!
//! This crate provides a gRPC client for connecting actrix nodes to
//! the centralized actrix-supervisor management platform.
//!
//! # Features
//!
//! - **Status Reporting**: Periodic system metrics and service status reporting
//! - **Configuration Updates**: Receive and apply configuration changes from supervisor
//! - **Tenant Management**: Remote tenant CRUD operations
//! - **Health Checks**: Built-in health check and heartbeat mechanism
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────┐         gRPC/HTTP2          ┌─────────────────┐
//! │  actrix-node    │◄──────────────────────────►│ actrix-supervisor│
//! │  (SupervitClient)│    Bidirectional Stream    │  (gRPC Server)  │
//! └─────────────────┘                             └─────────────────┘
//! ```

pub mod client;
pub mod config;
pub mod error;
pub mod metrics;
pub mod nonce_auth;

// Re-export important types
pub use client::SupervitClient;
pub use config::SupervitConfig;
pub use error::{Result, SupervitError};

// Generated protobuf code
pub mod generated {
    tonic::include_proto!("supervisor.v1");
}

// Re-export commonly used proto types
pub use generated::{
    supervisor_client::SupervisorClient, ConfigType, ConfigUpdateRequest, ConfigUpdateResponse,
    HealthCheckRequest, HealthCheckResponse, OperationType, ResourceType, ServiceStatus, StatusAck,
    StatusReport, SystemMetrics, TenantCreateInfo, TenantInfo, TenantList, TenantOperation,
    TenantOperationResponse, TenantUpdateInfo,
};
