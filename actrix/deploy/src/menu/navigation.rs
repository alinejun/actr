//! Simple text-based navigation stack for nested menus

use std::collections::VecDeque;

/// Navigation item
#[derive(Debug, Clone)]
pub struct NavItem {
    pub title: String,
}

impl NavItem {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
        }
    }
}

/// Simple navigation stack for text-based nested menus
pub struct NavigationStack {
    items: VecDeque<NavItem>,
}

impl NavigationStack {
    pub fn new() -> Self {
        Self {
            items: VecDeque::new(),
        }
    }

    /// Initialize with root item
    pub fn with_root(title: &str) -> Self {
        let mut stack = Self::new();
        stack.push(NavItem::new(title));
        stack
    }

    /// Push a new navigation item
    pub fn push(&mut self, item: NavItem) {
        self.items.push_back(item);
    }

    /// Pop the current item (but keep at least one)
    pub fn pop(&mut self) -> bool {
        if self.items.len() > 1 {
            self.items.pop_back();
            true
        } else {
            false
        }
    }

    /// Check if we can go back
    pub fn can_go_back(&self) -> bool {
        self.items.len() > 1
    }

    /// Get depth
    pub fn depth(&self) -> usize {
        self.items.len()
    }
}
