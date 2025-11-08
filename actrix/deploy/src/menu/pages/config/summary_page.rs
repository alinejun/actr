//! Summary and confirmation page for configuration wizard

use crate::config::{DeploymentConfig, NetworkConfig, SslConfig, SystemConfig};
use crate::menu::framework::{
    ContentArea, Layout, LayoutComponents, Page, PageContext, PageResult, StandardLayout,
};
use crate::services::ServiceSelection;
use crate::system::press_any_key_to_with_interrupt;
use anyhow::Result;
use dialoguer::{Confirm, theme::ColorfulTheme};

pub struct SummaryPage {
    layout: StandardLayout,
    services: ServiceSelection,
    network_config: NetworkConfig,
    ssl_config: Option<SslConfig>,
    system_config: SystemConfig,
}

impl SummaryPage {
    pub fn new(
        services: ServiceSelection,
        network_config: NetworkConfig,
        ssl_config: Option<SslConfig>,
        system_config: SystemConfig,
    ) -> Self {
        Self {
            layout: StandardLayout,
            services,
            network_config,
            ssl_config,
            system_config,
        }
    }

    fn build_summary(&self) -> Vec<String> {
        let mut summary = Vec::new();

        // Services section
        summary.push("📋 Selected Services:".to_string());
        for service in &self.services.services {
            summary.push(format!("  • {} - {}", service, service.description()));
        }
        summary.push("".to_string());

        // Network configuration
        summary.push("🌐 Network Configuration:".to_string());
        summary.push(format!(
            "  • Server Host: {}",
            self.network_config.server_host
        ));
        summary.push(format!("  • ICE Port: {}", self.network_config.ice_port));
        summary.push(format!(
            "  • HTTPS Port: {}",
            self.network_config.https_port
        ));
        summary.push("".to_string());

        // SSL configuration
        if let Some(ssl) = &self.ssl_config {
            summary.push("🔐 SSL/TLS Configuration:".to_string());
            summary.push("  • SSL/TLS: Enabled".to_string());
            summary.push(format!("  • Domain Name: {}", ssl.domain_name));
            summary.push(format!("  • Certificate Path: {}", ssl.cert_path.display()));
            summary.push(format!("  • Private Key Path: {}", ssl.key_path.display()));
        } else {
            summary.push("🔐 SSL/TLS Configuration:".to_string());
            summary.push("  • SSL/TLS: Disabled".to_string());
        }
        summary.push("".to_string());

        // System configuration
        summary.push("⚙️ System Configuration:".to_string());
        summary.push(format!(
            "  • Server Name: {}",
            self.system_config.server_name
        ));
        summary.push(format!(
            "  • Location Tag: {}",
            self.system_config.location_tag
        ));
        summary.push(format!("  • Run User: {}", self.system_config.run_user));
        summary.push(format!("  • Run Group: {}", self.system_config.run_group));
        if let Some(realm) = &self.system_config.turn_realm {
            summary.push(format!("  • TURN Realm: {}", realm));
        }

        summary
    }
}

impl Page for SummaryPage {
    fn title(&self) -> &str {
        "Configuration Summary"
    }

    fn render(&mut self, context: &mut PageContext) -> Result<PageResult> {
        // Build layout components using standard format
        let summary_lines = self.build_summary();
        let components = LayoutComponents::new("ActorRTC Auxiliary Services Deployment Helper")
            .with_page_title("Configuration Wizard - Summary")
            .with_operation_hint("Review your configuration and confirm to apply")
            .add_content({
                let mut content = ContentArea::new();
                for line in summary_lines {
                    content = content.add_line(line);
                }
                content
            });

        // Render the layout
        self.layout.render(components);

        // Add an empty line before the confirmation prompt
        println!();

        // Wait for user confirmation using dialoguer
        let theme = ColorfulTheme::default();
        match Confirm::with_theme(&theme)
            .with_prompt("Apply this configuration?")
            .default(true)
            .interact()
        {
            Ok(true) => {
                // Create final deployment config
                let deployment_config = DeploymentConfig {
                    services: self.services.clone(),
                    network: self.network_config.clone(),
                    ssl: self.ssl_config.clone(),
                    system: self.system_config.clone(),
                    // install: InstallConfig::default(),
                };

                // Apply configuration - actually generate the config file
                let config_path = std::path::PathBuf::from("/etc/actor-rtc-actrix/config.toml");
                let mut processor = crate::template::TemplateProcessor::new();
                processor.generate_config(&deployment_config, &config_path)?;

                println!("✅ Configuration applied successfully!");
                println!("📄 Configuration written to: {}", config_path.display());
                println!("🚀 Deployment process completed.");

                // Wait for user to acknowledge completion
                let interrupted =
                    press_any_key_to_with_interrupt("return to main", context.interrupted.clone());
                if interrupted {
                    return Ok(PageResult::Stay); // Let MenuApplication handle Ctrl+C
                }

                // Return to root (main) and clear the configuration wizard stack
                Ok(PageResult::BackToRoot)
            }
            Ok(false) => {
                // User cancelled
                Ok(PageResult::Back)
            }
            Err(_) => {
                // Error occurred (e.g., Ctrl+C)
                Ok(PageResult::Back)
            }
        }
    }
}
