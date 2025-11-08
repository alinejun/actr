//! Supervisor client integration (formerly managed)

use crate::service::ServiceType;
use crate::service::{HttpRouterService, info::ServiceInfo};
use actrix_common::config::ActrixConfig;
use anyhow::Result;
use async_trait::async_trait;
use axum::{Router, routing::get};
use tracing::info;

/// Supervisor client service (placeholder)
#[derive(Debug)]
pub struct SupervisorService {
    info: ServiceInfo,
}

impl SupervisorService {
    pub fn new(config: ActrixConfig) -> Self {
        Self {
            info: ServiceInfo::new(
                "Supervisor Client Service",
                ServiceType::Supervisor,
                Some("WebSocket-based supervisor client integration".to_string()),
                &config,
            ),
        }
    }
}

#[async_trait]
impl HttpRouterService for SupervisorService {
    fn info(&self) -> &ServiceInfo {
        &self.info
    }

    fn info_mut(&mut self) -> &mut ServiceInfo {
        &mut self.info
    }

    async fn build_router(&mut self) -> Result<Router> {
        info!("Building Supervisor Client router");

        // Simple health check router for supervisor client
        let router = Router::new()
            .route("/health", get(|| async { "Supervisor client is healthy" }))
            .route("/status", get(|| async { "Supervisor client status" }))
            .route("/metrics", get(super::metrics_endpoint));

        info!("Supervisor Client router built successfully");
        Ok(router)
    }

    fn route_prefix(&self) -> &str {
        "/supervisor"
    }
}
