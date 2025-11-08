//! SSL configuration page for configuration wizard

use super::SystemConfigPage;
use crate::config::{NetworkConfig, SslConfig};
use crate::menu::framework::{
    Layout, LayoutComponents, Page, PageContext, PageResult, StandardLayout,
};
use crate::services::ServiceSelection;
use crate::system::press_any_key_to_with_interrupt;
use anyhow::Result;
use dialoguer::{Input, theme::ColorfulTheme};
use std::path::PathBuf;

pub struct SslConfigPage {
    layout: StandardLayout,
    services: ServiceSelection,
    network_config: NetworkConfig,
}

impl SslConfigPage {
    fn configure_ssl(&self) -> Result<SslConfig> {
        let theme = ColorfulTheme::default();

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

        Ok(SslConfig {
            domain_name,
            cert_path: PathBuf::from(cert_path_str),
            key_path: PathBuf::from(key_path_str),
        })
    }
}

impl Page for SslConfigPage {
    fn title(&self) -> &str {
        "SSL Configuration"
    }

    fn render(&mut self, context: &mut PageContext) -> Result<PageResult> {
        // Build layout components using standard format
        let components = LayoutComponents::new("ActorRTC Auxiliary Services Deployment Helper")
            .with_page_title("Configuration Wizard - Step 3: SSL Configuration")
            .with_operation_hint("Configure SSL/TLS settings for HTTPS services");

        // Render the layout
        self.layout.render(components);

        // Configure SSL settings
        match self.configure_ssl() {
            Ok(ssl_config) => {
                println!("✅ SSL configuration completed.");

                // Wait for user to press a key before proceeding
                let interrupted = press_any_key_to_with_interrupt(
                    "continue to system configuration",
                    context.interrupted.clone(),
                );
                if interrupted {
                    return Ok(PageResult::Stay); // Let MenuApplication handle Ctrl+C
                }

                // Navigate to next step
                Ok(PageResult::Navigate(Box::new(SystemConfigPage::new(
                    self.services.clone(),
                    self.network_config.clone(),
                    Some(ssl_config),
                ))))
            }
            Err(_) => {
                // User cancelled or error occurred
                Ok(PageResult::Back)
            }
        }
    }
}
