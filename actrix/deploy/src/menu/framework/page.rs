//! Page abstraction and context

use crate::menu::navigation::NavigationStack;
use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Result type for page actions
pub enum PageResult {
    /// Continue to another page
    Navigate(Box<dyn Page>),
    /// Go back to previous page
    Back,
    /// Go back to root page (main)
    BackToRoot,
    /// Exit the application
    Exit,
    /// Stay on current page
    Stay,
}

/// Context passed to pages
pub struct PageContext {
    pub navigation: NavigationStack,
    pub debug: bool,
    pub interrupted: Arc<AtomicBool>,
}

impl PageContext {
    pub fn new(debug: bool, interrupted: Arc<AtomicBool>) -> Self {
        Self {
            navigation: NavigationStack::new(),
            debug,
            interrupted,
        }
    }
}

/// Page trait that all menu pages must implement
pub trait Page: Send {
    /// Get the page title
    fn title(&self) -> &str;

    /// Render and handle the page
    fn render(&mut self, context: &mut PageContext) -> Result<PageResult>;

    /// Called when entering this page
    fn on_enter(&mut self, context: &mut PageContext) {
        context
            .navigation
            .push(crate::menu::navigation::NavItem::new(self.title()));
    }

    /// Called when leaving this page
    fn on_leave(&mut self, context: &mut PageContext) {
        context.navigation.pop();
    }
}
