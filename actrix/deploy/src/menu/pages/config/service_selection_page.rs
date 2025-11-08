//! Service selection page for configuration wizard

use super::NetworkConfigPage;
use crate::menu::framework::{
    DefaultTheme, Layout, LayoutComponents, Page, PageContext, PageResult, StandardLayout,
};
use crate::services::ServiceSelection;
use anyhow::Result;

pub struct ServiceSelectionPage {
    layout: StandardLayout,
    theme: DefaultTheme,
    selected_services: Option<ServiceSelection>,
}

impl ServiceSelectionPage {
    pub fn new() -> Self {
        Self {
            layout: StandardLayout,
            theme: DefaultTheme::default(),
            selected_services: None,
        }
    }
}

impl Page for ServiceSelectionPage {
    fn title(&self) -> &str {
        "Service Selection"
    }

    fn render(&mut self, _context: &mut PageContext) -> Result<PageResult> {
        // Build layout components using standard format
        let components = LayoutComponents::new("ActorRTC Auxiliary Services Deployment Helper")
            .with_page_title("Configuration Wizard - Step 1: Service Selection")
            .with_operation_hint(
                "Use ↑↓ arrow keys to navigate, space to toggle, enter to continue",
            );

        // Render the layout
        self.layout.render(components);

        // Get service selection - this will show the interactive selection dialog
        match ServiceSelection::prompt_for_selection() {
            Ok(services) => {
                // Store the selected services
                self.selected_services = Some(services.clone());

                // Directly navigate to network configuration page
                Ok(PageResult::Navigate(Box::new(NetworkConfigPage::new(
                    services,
                ))))
            }
            Err(_) => {
                // User cancelled or error occurred
                Ok(PageResult::Back)
            }
        }
    }
}
