//! System configuration page for configuration wizard

use super::SummaryPage;
use crate::config::{NetworkConfig, SslConfig, SystemConfig};
use crate::menu::framework::{
    Layout, LayoutComponents, Page, PageContext, PageResult, StandardLayout,
};
use crate::services::ServiceSelection;
use crate::system::press_any_key_to_with_interrupt;
use anyhow::Result;
use dialoguer::{Confirm, Input, theme::ColorfulTheme};

pub struct SystemConfigPage {
    layout: StandardLayout,
    services: ServiceSelection,
    network_config: NetworkConfig,
    ssl_config: Option<SslConfig>,
}

impl SystemConfigPage {
    pub fn new(
        services: ServiceSelection,
        network_config: NetworkConfig,
        ssl_config: Option<SslConfig>,
    ) -> Self {
        Self {
            layout: StandardLayout,
            services,
            network_config,
            ssl_config,
        }
    }

    fn configure_system(&self) -> Result<SystemConfig> {
        let theme = ColorfulTheme::default();

        // Get server name
        let server_name: String = Input::with_theme(&theme)
            .with_prompt("Server name (identifier for this deployment)")
            .default("actor-rtc-actrix-server".to_string())
            .interact_text()?;

        // Get location tag
        let location_tag: String = Input::with_theme(&theme)
            .with_prompt("Location tag (region/datacenter identifier)")
            .default("default".to_string())
            .interact_text()?;

        // Get run user
        let run_user: String = Input::with_theme(&theme)
            .with_prompt("System user to run services as")
            .default("actor-rtc".to_string())
            .interact_text()?;

        // Get run group
        let run_group: String = Input::with_theme(&theme)
            .with_prompt("System group to run services as")
            .default("actor-rtc".to_string())
            .interact_text()?;

        // Get TURN realm (optional, only if TURN service is selected)
        let turn_realm = if self.services.needs_turn() {
            let use_turn_realm = Confirm::with_theme(&theme)
                .with_prompt("Configure TURN realm?")
                .default(true)
                .interact()?;

            if use_turn_realm {
                let realm: String = Input::with_theme(&theme)
                    .with_prompt("TURN realm (domain for TURN authentication)")
                    .default("localhost".to_string())
                    .interact_text()?;
                Some(realm)
            } else {
                None
            }
        } else {
            None
        };

        Ok(SystemConfig {
            server_name,
            location_tag,
            run_user,
            run_group,
            turn_realm,
        })
    }
}

impl Page for SystemConfigPage {
    fn title(&self) -> &str {
        "System Configuration"
    }

    fn render(&mut self, context: &mut PageContext) -> Result<PageResult> {
        // Build layout components using standard format
        let components = LayoutComponents::new("ActorRTC Auxiliary Services Deployment Helper")
            .with_page_title("Configuration Wizard - Step 3: System Configuration")
            .with_operation_hint("Configure system settings and service parameters");

        // Render the layout
        self.layout.render(components);

        // Configure system settings
        match self.configure_system() {
            Ok(system_config) => {
                println!("✅ System configuration completed.");

                // Wait for user to press a key before proceeding
                let interrupted = press_any_key_to_with_interrupt(
                    "continue to configuration summary",
                    context.interrupted.clone(),
                );
                if interrupted {
                    return Ok(PageResult::Stay); // Let MenuApplication handle Ctrl+C
                }

                // Navigate to summary page
                Ok(PageResult::Navigate(Box::new(SummaryPage::new(
                    self.services.clone(),
                    self.network_config.clone(),
                    self.ssl_config.clone(),
                    system_config,
                ))))
            }
            Err(_) => {
                // User cancelled or error occurred
                Ok(PageResult::Back)
            }
        }
    }
}
