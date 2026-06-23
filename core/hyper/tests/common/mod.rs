//! Common test utilities for WebRTC and signaling tests
//!
//! This module provides shared infrastructure for integration tests:
//! - WebSocket-based signaling server (TestSignalingServer)
//! - Helper functions for creating peers and credentials
//! - Virtual network (VNet) for simulating network disconnection
//! - Common test utilities

pub mod harness;
pub mod signaling;
pub mod utils;
pub mod vnet;
pub mod wait;

pub use harness::{TestHarness, TestPeer};
pub use signaling::TestSignalingServer;
pub use utils::*;
pub use vnet::{VNetPair, create_vnet_pair};
pub use wait::*;

#[cfg(not(target_arch = "wasm32"))]
static RUSTLS_CRYPTO_PROVIDER_INIT: std::sync::Once = std::sync::Once::new();

#[cfg(not(target_arch = "wasm32"))]
pub fn install_default_crypto_provider_for_tests() {
    RUSTLS_CRYPTO_PROVIDER_INIT.call_once(|| {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });
}
