//! Complete installation wizard page

use crate::config::{ConfigWizard, InstallConfig};
use crate::menu::framework::{
    ContentArea, DefaultTheme, Layout, LayoutComponents, Page, PageContext, PageResult,
    StandardLayout, Theme,
};
use crate::system::{
    check_dependencies, install_application, install_systemd_service,
    press_any_key_to_with_interrupt,
};
use anyhow::Result;
use dialoguer::Confirm;

pub struct WizardPage {
    theme: DefaultTheme,
    layout: StandardLayout,
}

impl WizardPage {
    pub fn new() -> Self {
        Self {
            theme: DefaultTheme::default(),
            layout: StandardLayout,
        }
    }

    fn run_wizard(&self, context: &mut PageContext) -> Result<()> {
        // Step 1: Dependencies
        println!("Step 1: Checking dependencies...");

        check_dependencies()?;
        println!();

        // Step 2: Configuration
        println!("Step 2: Running configuration wizard...");

        let mut wizard = ConfigWizard::new(context.debug);
        let _config_path = match wizard.run() {
            Ok(path) => path,
            Err(e) if e.to_string() == "config_file_exists" => {
                println!("⚠️  Configuration wizard cancelled - config file already exists");
                println!("You can skip to installation or use existing configuration.");
                return Err(anyhow::anyhow!("wizard_cancelled"));
            }
            Err(e) => return Err(e),
        };
        println!();

        // Step 3: Installation
        println!("Step 3: Installing application files...");

        let install_config = InstallConfig::default();
        install_application(&install_config)?;
        println!();

        // Step 4: Systemd service (optional)
        println!();

        match Confirm::with_theme(self.theme.dialoguer_theme())
            .with_prompt("Deploy as systemd service?")
            .default(true)
            .interact()
        {
            Ok(true) => {
                println!("Step 4: Deploying systemd service...");
                install_systemd_service()?;
            }
            Ok(false) => {
                println!("Step 4: Skipped systemd service deployment");
            }
            Err(e) => {
                if format!("{}", e).contains("read interrupted") {
                    return Err(e.into()); // Propagate interrupt to be handled by render()
                } else {
                    return Err(e.into()); // Other errors
                }
            }
        }

        println!();
        println!("🎉 Complete installation wizard finished!");
        println!("Your actrix server should now be ready to use.");

        Ok(())
    }
}

impl Page for WizardPage {
    fn title(&self) -> &str {
        "Complete Installation Wizard"
    }

    fn render(&mut self, context: &mut PageContext) -> Result<PageResult> {
        // Build layout components using new standard format
        let components = LayoutComponents::new("ActorRTC Auxiliary Services Deployment Helper")
            .with_page_title("Complete Installation Wizard")
            .with_operation_hint("This wizard will guide you through the complete setup process")
            .add_content(ContentArea::new().add_section(
                "Steps to be performed",
                vec![
                    "Check system dependencies".to_string(),
                    "Configure services".to_string(),
                    "Install application files".to_string(),
                    "Deploy systemd service (optional)".to_string(),
                ],
            ));

        // Render the layout
        self.layout.render(components);

        // Run the wizard
        match self.run_wizard(context) {
            Ok(_) => {
                let interrupted =
                    press_any_key_to_with_interrupt("continue", context.interrupted.clone());
                if interrupted {
                    return Ok(PageResult::Stay); // Let MenuApplication handle Ctrl+C
                }
                Ok(PageResult::Back)
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                // Check if this was an interrupt
                if error_msg.contains("read interrupted") {
                    // Let MenuApplication handle the interrupt
                    Ok(PageResult::Stay)
                } else if error_msg == "wizard_cancelled" {
                    // User cancelled due to existing config file - return to main
                    println!();
                    let interrupted = press_any_key_to_with_interrupt(
                        "return to main",
                        context.interrupted.clone(),
                    );
                    if interrupted {
                        return Ok(PageResult::Stay); // Let MenuApplication handle Ctrl+C
                    }
                    Ok(PageResult::Back)
                } else {
                    // Real error - propagate it
                    Err(e)
                }
            }
        }
    }
}
