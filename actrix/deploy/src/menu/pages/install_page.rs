//! Install application page

use crate::config::InstallConfig;
use crate::menu::framework::{
    ContentArea, DefaultTheme, Layout, LayoutComponents, Page, PageContext, PageResult,
    StandardLayout,
};
use crate::system::{install_application, press_any_key_to_with_interrupt};
use anyhow::Result;
use dialoguer::{Confirm, Input, theme::ColorfulTheme};
use std::path::PathBuf;

pub struct InstallPage {
    theme: DefaultTheme,
    layout: StandardLayout,
}

impl InstallPage {
    pub fn new() -> Self {
        Self {
            theme: DefaultTheme::default(),
            layout: StandardLayout,
        }
    }

    fn configure_install_paths(&self) -> Result<InstallConfig> {
        let theme = ColorfulTheme::default();

        println!("📂 Installation Path Configuration");
        println!("═══════════════════════════════════");
        println!("Configure installation paths (press Enter for defaults):");
        println!();

        let default_config = InstallConfig::default();

        // Install directory
        let install_dir: String = Input::with_theme(&theme)
            .with_prompt("Installation directory")
            .default(default_config.install_dir.to_string_lossy().to_string())
            .interact_text()?;

        // Binary name
        let binary_name: String = Input::with_theme(&theme)
            .with_prompt("Binary name")
            .default(default_config.binary_name.clone())
            .interact_text()?;

        // Add to PATH
        let add_to_path = Confirm::with_theme(&theme)
            .with_prompt("Create symlink in /usr/local/bin for PATH access?")
            .default(default_config.add_to_path)
            .interact()?;

        println!();

        Ok(InstallConfig {
            install_dir: PathBuf::from(install_dir),
            binary_name,
            add_to_path,
        })
    }

    fn run_install(&self, _context: &mut PageContext) -> Result<()> {
        // Configure installation paths
        let install_config = self.configure_install_paths()?;

        // Show summary
        println!("📋 Installation Summary:");
        println!(
            "  • Install Directory: {}",
            install_config.install_dir.display()
        );
        println!("  • Binary Name: {}", install_config.binary_name);
        println!(
            "  • Add to PATH: {}",
            if install_config.add_to_path {
                "Yes"
            } else {
                "No"
            }
        );
        println!();

        // Confirm installation
        let theme = ColorfulTheme::default();
        let proceed = Confirm::with_theme(&theme)
            .with_prompt("Proceed with installation?")
            .default(true)
            .interact()?;

        if !proceed {
            println!("❌ Installation cancelled by user");
            return Ok(());
        }

        println!();
        println!("🚀 Starting installation...");
        println!();

        // Run the installation
        install_application(&install_config)?;

        Ok(())
    }
}

impl Page for InstallPage {
    fn title(&self) -> &str {
        "Install Application"
    }

    fn render(&mut self, context: &mut PageContext) -> Result<PageResult> {
        // Build layout components
        let components = LayoutComponents::new("ActorRTC Auxiliary Services Deployment Helper")
            .with_page_title("Install Application")
            .with_operation_hint("Installing application files to system directories")
            .add_content(ContentArea::new().add_section(
                "Installation Process",
                vec![
                    "Configure installation paths".to_string(),
                    "Create system directories".to_string(),
                    "Deploy application files".to_string(),
                    "Set proper permissions".to_string(),
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
                eprintln!("Installation failed: {}", e);
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
