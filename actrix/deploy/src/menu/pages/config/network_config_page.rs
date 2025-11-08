//! Network configuration page for configuration wizard

use super::SystemConfigPage;
use crate::config::{NetworkConfig, SslConfig};
use crate::menu::framework::{
    ContentArea, Layout, LayoutComponents, Page, PageContext, PageResult, StandardLayout,
};
use crate::services::ServiceSelection;
use crate::system::{NetworkUtils, press_any_key_to_with_interrupt};
use anyhow::Result;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use std::path::PathBuf;

pub struct NetworkConfigPage {
    layout: StandardLayout,
    services: ServiceSelection,
}

impl NetworkConfigPage {
    pub fn new(services: ServiceSelection) -> Self {
        Self {
            layout: StandardLayout,
            services,
        }
    }

    fn classify_ip(ip: &str) -> &'static str {
        if ip.starts_with("127.") {
            "localhost"
        } else if ip.starts_with("10.")
            || ip.starts_with("192.168.")
            || (ip.starts_with("172.")
                && ip
                    .split('.')
                    .nth(1)
                    .and_then(|s| s.parse::<u8>().ok())
                    .map(|n| n >= 16 && n <= 31)
                    .unwrap_or(false))
        {
            "private IPv4"
        } else if ip.contains(':') {
            "IPv6"
        } else {
            "public IPv4"
        }
    }

    fn select_server_host(&self) -> Result<String> {
        let theme = ColorfulTheme::default();
        let local_ips = NetworkUtils::get_local_ips()?;

        // Create options with classification labels
        let mut options = Vec::new();
        let mut ip_strings = Vec::new();
        for ip in &local_ips {
            let ip_str = ip.to_string();
            let classification = Self::classify_ip(&ip_str);
            options.push(format!("{} ({})", ip_str, classification));
            ip_strings.push(ip_str);
        }
        options.push("Enter custom IP/domain".to_string());

        println!("Available network interfaces:");
        let selection = Select::with_theme(&theme)
            .with_prompt("Select server host (use ↑↓ arrow keys to navigate, enter to select)")
            .items(&options)
            .default(0)
            .interact()?;

        if selection == options.len() - 1 {
            // Custom input
            let custom_host: String = Input::with_theme(&theme)
                .with_prompt("Enter custom IP address or domain")
                .interact_text()?;
            Ok(custom_host)
        } else {
            Ok(ip_strings[selection].clone())
        }
    }

    fn configure_network_and_ssl(&self) -> Result<(NetworkConfig, Option<SslConfig>)> {
        let theme = ColorfulTheme::default();

        // Get server host using arrow key selection
        let server_host = self.select_server_host()?;

        // Get ICE port (for STUN/TURN services)
        let ice_port: u16 = Input::with_theme(&theme)
            .with_prompt("ICE Port (for STUN/TURN services)")
            .default(3478)
            .interact_text()?;

        // Get HTTPS port
        let https_port: u16 = Input::with_theme(&theme)
            .with_prompt("HTTPS Port (for web services)")
            .default(8443)
            .interact_text()?;

        let network_config = NetworkConfig {
            server_host,
            ice_port,
            https_port,
            http_port: 8080, // 默认 HTTP 端口
        };

        // Configure SSL if HTTP services are enabled
        let ssl_config = if self.services.needs_http() {
            println!("\n📋 SSL/TLS Configuration");
            let enable_ssl = Confirm::with_theme(&theme)
                .with_prompt("Enable SSL/TLS for HTTPS services?")
                .default(true)
                .interact()?;

            if enable_ssl {
                // Get domain name
                let domain_name: String = Input::with_theme(&theme)
                    .with_prompt("Domain name for SSL certificate")
                    .default("localhost".to_string())
                    .interact_text()?;

                // Get certificate path
                let cert_path_str: String = Input::with_theme(&theme)
                    .with_prompt("Path to SSL certificate file")
                    .default("/etc/ssl/certs/server.crt".to_string())
                    .interact_text()?;

                // Get private key path
                let key_path_str: String = Input::with_theme(&theme)
                    .with_prompt("Path to SSL private key file")
                    .default("/etc/ssl/private/server.key".to_string())
                    .interact_text()?;

                Some(SslConfig {
                    domain_name,
                    cert_path: PathBuf::from(cert_path_str),
                    key_path: PathBuf::from(key_path_str),
                })
            } else {
                None
            }
        } else {
            None
        };

        Ok((network_config, ssl_config))
    }
}

impl Page for NetworkConfigPage {
    fn title(&self) -> &str {
        "Network & SSL Configuration"
    }

    fn render(&mut self, context: &mut PageContext) -> Result<PageResult> {
        // Build layout components using standard format
        let components = LayoutComponents::new("ActorRTC Auxiliary Services Deployment Helper")
            .with_page_title("Configuration Wizard - Step 2: Network & SSL Configuration")
            .with_operation_hint("Configure network settings, ports, and SSL certificates")
            .add_content(
                ContentArea::new().add_section(
                    "Selected Services",
                    self.services
                        .services
                        .iter()
                        .map(|s| format!("{} - {}", s, s.description()))
                        .collect(),
                ),
            );

        // Render the layout
        self.layout.render(components);

        // Configure network and SSL settings
        match self.configure_network_and_ssl() {
            Ok((network_config, ssl_config)) => {
                if ssl_config.is_some() {
                    println!("✅ Network and SSL configuration completed.");
                } else {
                    println!("✅ Network configuration completed.");
                }

                // Wait for user to press a key before proceeding
                let interrupted = press_any_key_to_with_interrupt(
                    "continue to system configuration",
                    context.interrupted.clone(),
                );
                if interrupted {
                    return Ok(PageResult::Stay); // Let MenuApplication handle Ctrl+C
                }

                // Navigate directly to system configuration
                Ok(PageResult::Navigate(Box::new(SystemConfigPage::new(
                    self.services.clone(),
                    network_config,
                    ssl_config,
                ))))
            }
            Err(_) => {
                // User cancelled or error occurred
                Ok(PageResult::Back)
            }
        }
    }
}
