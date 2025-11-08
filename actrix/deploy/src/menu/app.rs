//! Application class for managing menu navigation

use super::framework::{Page, PageContext, PageResult};
use super::pages::MainPage;
use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Main application class
pub struct MenuApplication {
    context: PageContext,
    page_stack: Vec<Box<dyn Page>>,
    interrupted: Arc<AtomicBool>,
}

impl MenuApplication {
    /// Create new menu application
    pub fn new(debug: bool, interrupted: Arc<AtomicBool>) -> Self {
        Self {
            context: PageContext::new(debug, interrupted.clone()),
            page_stack: vec![Box::new(MainPage::new())],
            interrupted,
        }
    }

    /// Run the application
    pub fn run(&mut self) -> Result<()> {
        while let Some(mut current_page) = self.page_stack.pop() {
            // Enter the page
            current_page.on_enter(&mut self.context);

            // Render and handle the page
            let result = current_page.render(&mut self.context)?;

            // Leave the page
            current_page.on_leave(&mut self.context);

            // Check if Ctrl+C was pressed during page execution
            if self.interrupted.load(Ordering::SeqCst) {
                self.interrupted.store(false, Ordering::SeqCst); // Reset flag

                // Debug: print stack state (only in debug mode)
                if self.context.debug {
                    println!("\nDEBUG: Stack size: {}", self.page_stack.len());
                }

                // If we're on the main page (no pages left in stack), exit
                // Otherwise, return to main
                if self.page_stack.is_empty() {
                    println!("\n👋 Goodbye!\n");
                    break;
                } else {
                    println!("\n🔙 Returning to main...");
                    self.page_stack.clear();
                    self.page_stack.push(Box::new(MainPage::new()));
                    continue;
                }
            }

            // Handle the result
            match result {
                PageResult::Navigate(next_page) => {
                    // Push current page back to stack
                    self.page_stack.push(current_page);
                    // Push new page
                    self.page_stack.push(next_page);
                }
                PageResult::Back => {
                    // Current page is already popped, just continue
                    if self.page_stack.is_empty() {
                        // If no pages left, exit
                        break;
                    }
                }
                PageResult::BackToRoot => {
                    // Clear all pages except the root (keep only the first page)
                    self.page_stack.truncate(1);

                    // Also clear the navigation stack to root
                    while self.context.navigation.can_go_back() {
                        self.context.navigation.pop();
                    }
                }
                PageResult::Exit => {
                    // Clear the stack to exit
                    self.page_stack.clear();
                    break;
                }
                PageResult::Stay => {
                    // Push the page back
                    self.page_stack.push(current_page);
                }
            }
        }

        Ok(())
    }
}
