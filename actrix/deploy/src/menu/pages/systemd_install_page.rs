//! Systemd service installation page

use crate::menu::framework::{
    ContentArea, DefaultTheme, Layout, LayoutComponents, Page, PageContext, PageResult,
    StandardLayout,
};
use crate::system::{install_systemd_service, press_any_key_to_with_interrupt};
use anyhow::Result;

pub struct SystemdInstallPage {
    theme: DefaultTheme,
    layout: StandardLayout,
}

impl SystemdInstallPage {
    pub fn new() -> Self {
        Self {
            theme: DefaultTheme::default(),
            layout: StandardLayout,
        }
    }

    fn run_install(&self, _context: &mut PageContext) -> Result<()> {
        // Run the systemd service installation
        install_systemd_service()?;

        Ok(())
    }
}

impl Page for SystemdInstallPage {
    fn title(&self) -> &str {
        "Deploy systemd Service"
    }

    fn render(&mut self, context: &mut PageContext) -> Result<PageResult> {
        // Build layout components
        let components = LayoutComponents::new("ActorRTC Auxiliary Services Deployment Helper")
            .with_page_title("Deploy systemd Service")
            .with_operation_hint("Deploying application as systemd service")
            .add_content(ContentArea::new().add_section(
                "Deployment Process",
                vec![
                    "Check systemd availability".to_string(),
                    "Create service unit file".to_string(),
                    "Enable and start service".to_string(),
                ],
            ));

        // Render the layout
        self.layout.render(components);

        // Run the installation
        match self.run_install(context) {
            Ok(_) => {
                let interrupted =
                    press_any_key_to_with_interrupt("continue", context.interrupted.clone());
                if interrupted {
                    Ok(PageResult::Stay) // Let MenuApplication handle Ctrl+C
                } else {
                    Ok(PageResult::Back)
                }
            }
            Err(e) => {
                eprintln!("Systemd service deployment failed: {}", e);
                let interrupted =
                    press_any_key_to_with_interrupt("continue", context.interrupted.clone());
                if interrupted {
                    Ok(PageResult::Stay) // Let MenuApplication handle Ctrl+C
                } else {
                    Ok(PageResult::Back)
                }
            }
        }
    }
}
