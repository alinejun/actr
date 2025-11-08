//! Network utilities for IP detection

use anyhow::Result;
use std::net::IpAddr;

/// Network utility functions
pub struct NetworkUtils;

impl NetworkUtils {
    /// Get list of local IP addresses
    pub fn get_local_ips() -> Result<Vec<IpAddr>> {
        let mut ips = Vec::new();

        // Get all network interfaces
        if let Ok(interfaces) = local_ip_address::list_afinet_netifas() {
            for (_, ip) in interfaces {
                if !ip.is_loopback() {
                    ips.push(ip);
                }
            }
        }

        // Always include localhost
        ips.push(IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));

        // Remove duplicates and sort
        ips.sort();
        ips.dedup();

        Ok(ips)
    }
}
