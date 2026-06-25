//! systemctl operations for `update`/`rollback` restarts and health checks.
//!
//! Used after a version switch to restart an existing service and confirm it
//! came up. `update` never creates or edits systemd units — it only restarts
//! already-deployed services.

use anyhow::{Result, bail};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

/// Restart a systemd service.
pub fn restart(service: &str) -> Result<()> {
    println!("🔄 Restarting service '{service}' ...");
    let out = Command::new("sudo")
        .args(["systemctl", "restart", service])
        .output()?;
    if !out.status.success() {
        bail!(
            "systemctl restart {service} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(())
}

/// Whether a service is currently active.
pub fn is_active(service: &str) -> bool {
    Command::new("sudo")
        .args(["systemctl", "is-active", "--quiet", service])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Poll `is-active` once per second for up to `seconds`.
pub fn wait_active(service: &str, seconds: u32) -> Result<()> {
    for _ in 0..seconds {
        if is_active(service) {
            return Ok(());
        }
        sleep(Duration::from_secs(1));
    }
    bail!("service '{service}' did not become active within {seconds}s")
}

/// Health-check wait window, overridable via `ACTRIX_HEALTH_WAIT_SECONDS`.
pub fn health_wait_seconds() -> u32 {
    std::env::var("ACTRIX_HEALTH_WAIT_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5)
}
