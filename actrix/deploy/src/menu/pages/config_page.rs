//! Configuration wizard page

use super::config::ConfigLocationPage;
use crate::menu::framework::{Page, PageContext, PageResult};
use anyhow::Result;

pub struct ConfigPage;

impl ConfigPage {
    pub fn new() -> Self {
        Self
    }
}

impl Page for ConfigPage {
    fn title(&self) -> &str {
        "Configuration Wizard"
    }

    fn render(&mut self, _context: &mut PageContext) -> Result<PageResult> {
        // Navigate to config location page to start the configuration process
        Ok(PageResult::Navigate(Box::new(ConfigLocationPage::new())))
    }
}
