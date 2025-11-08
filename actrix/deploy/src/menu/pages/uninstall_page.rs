//! Uninstall application page

use crate::menu::framework::{
    DefaultTheme, Layout, LayoutComponents, Page, PageContext, PageResult, StandardLayout,
};
use crate::system::{press_any_key_to_with_interrupt, uninstall_application};
use anyhow::Result;

pub struct UninstallPage {
    theme: DefaultTheme,
    layout: StandardLayout,
}

impl UninstallPage {
    pub fn new() -> Self {
        Self {
            theme: DefaultTheme::default(),
            layout: StandardLayout,
        }
    }

    fn run_uninstall(&self, _context: &mut PageContext) -> Result<()> {
        // Run the uninstallation
        uninstall_application()?;

        Ok(())
    }
}

impl Page for UninstallPage {
    fn title(&self) -> &str {
        "Uninstall Application"
    }

    fn render(&mut self, context: &mut PageContext) -> Result<PageResult> {
        // Build layout components
        let components = LayoutComponents::new("ActorRTC Auxiliary Services Deployment Helper")
            .with_page_title("Uninstall Application")
            .with_operation_hint("Selectively remove actor-rtc-actrix components");

        // Render the layout
        self.layout.render(components);

        // Run the uninstallation
        match self.run_uninstall(context) {
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
                eprintln!("Uninstallation failed: {}", e);
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
