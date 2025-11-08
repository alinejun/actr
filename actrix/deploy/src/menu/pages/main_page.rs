//! main page implementation

use super::{
    ConfigPage, DependenciesPage, InstallPage, SystemdInstallPage, UninstallPage, WizardPage,
};
use crate::menu::framework::{
    DefaultTheme, EnhancedSelect, Layout, LayoutComponents, Page, PageContext, PageResult,
    SelectResult, StandardLayout, Theme,
};
use anyhow::Result;

pub struct MainPage {
    theme: DefaultTheme,
    layout: StandardLayout,
}

impl MainPage {
    pub fn new() -> Self {
        Self {
            theme: DefaultTheme::default(),
            layout: StandardLayout,
        }
    }
}

impl Page for MainPage {
    fn title(&self) -> &str {
        "Main"
    }

    fn render(&mut self, context: &mut PageContext) -> Result<PageResult> {
        // Build layout components using new standard format
        let components = LayoutComponents::new("ActorRTC Auxiliary Services Deployment Helper")
            .with_page_title("Main")
            .with_operation_hint("Use ↑↓ to navigate, Enter to select, Ctrl+C to exit");

        // Render the layout
        self.layout.render(components);

        // Define menu options
        let options = vec![
            "Complete Installation Wizard",
            "Check System Dependencies",
            "Configuration Wizard",
            "Install Application (Deploy Files)",
            "Deploy as systemd Service",
            "Uninstall",
            "Exit",
        ];

        // Use enhanced select (enable interrupt to catch Ctrl+C for double-press logic)
        let selection = EnhancedSelect::new(self.theme.dialoguer_theme())
            .with_prompt("Choose an action")
            .items(&options)
            .with_interrupt(true, Some(context.interrupted.clone()))
            .interact()?;

        // Handle user selection
        match selection {
            SelectResult::Selected(index) => match index {
                0 => Ok(PageResult::Navigate(Box::new(WizardPage::new()))),
                1 => Ok(PageResult::Navigate(Box::new(DependenciesPage::new()))),
                2 => Ok(PageResult::Navigate(Box::new(ConfigPage::new()))),
                3 => Ok(PageResult::Navigate(Box::new(InstallPage::new()))),
                4 => Ok(PageResult::Navigate(Box::new(SystemdInstallPage::new()))),
                5 => Ok(PageResult::Navigate(Box::new(UninstallPage::new()))),
                6 => {
                    println!("👋 Thank you for using the deployment helper!\n");
                    Ok(PageResult::Exit)
                }
                _ => unreachable!(),
            },
            SelectResult::Interrupted => {
                // Let MenuApplication handle Ctrl+C based on stack state
                Ok(PageResult::Stay)
            }
        }
    }
}
