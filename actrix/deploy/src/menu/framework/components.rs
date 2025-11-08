//! UI components for menu pages

use crate::system::DependencyCheckResult;
use colored::*;

/// Base trait for all UI components
pub trait Component {
    fn render(&self);
}

/// Header component with title and optional subtitle
pub struct Header {
    pub title: String,
    pub subtitle: Option<String>,
}

impl Header {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
        }
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }
}

impl Component for Header {
    fn render(&self) {
        println!("{}", self.title.bright_cyan().bold());
        if let Some(subtitle) = &self.subtitle {
            println!("{}", subtitle.dimmed());
        }
        println!("{}", "─".repeat(50).bright_cyan());
    }
}

/// Footer component with navigation hints
pub struct Footer {
    pub show_back: bool,
    pub custom_hints: Option<String>,
}

impl Footer {
    pub fn new(show_back: bool) -> Self {
        Self {
            show_back,
            custom_hints: None,
        }
    }

    #[allow(unused)]
    pub fn with_hints(mut self, hints: impl Into<String>) -> Self {
        self.custom_hints = Some(hints.into());
        self
    }
}

impl Component for Footer {
    fn render(&self) {
        println!();
        println!("{}", "─".repeat(50).dimmed());

        if let Some(hints) = &self.custom_hints {
            println!("{}", hints.dimmed());
        } else if self.show_back {
            println!(
                "{}",
                "↑↓ - navigate | Enter - select | Ctrl+C - back to main".dimmed()
            );
        } else {
            println!(
                "{}",
                "↑↓ - navigate | Enter - select | Ctrl+C - exit".dimmed()
            );
        }
    }
}

/// Menu list component (reserved for future use)
#[allow(unused)]
pub struct MenuList {
    pub title: String,
    pub items: Vec<String>,
}

#[allow(unused)]
impl MenuList {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
        }
    }

    pub fn add_item(mut self, item: impl Into<String>) -> Self {
        self.items.push(item.into());
        self
    }

    pub fn with_items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        self
    }
}

/// Content area for displaying text
pub struct ContentArea {
    pub content: Vec<String>,
}

impl ContentArea {
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
        }
    }

    pub fn add_line(mut self, line: impl Into<String>) -> Self {
        self.content.push(line.into());
        self
    }

    pub fn add_section(mut self, title: impl Into<String>, lines: Vec<String>) -> Self {
        self.content
            .push(format!("{}:", title.into()).bold().to_string());
        for line in lines {
            self.content.push(format!("  • {}", line));
        }
        self.content.push(String::new());
        self
    }
}

impl Component for ContentArea {
    fn render(&self) {
        for line in &self.content {
            println!("{}", line);
        }
    }
}

/// Component for displaying dependency check results
pub struct DependencyCheckComponent {
    result: DependencyCheckResult,
}

impl DependencyCheckComponent {
    pub fn new(result: DependencyCheckResult) -> Self {
        Self { result }
    }
}

impl Component for DependencyCheckComponent {
    fn render(&self) {
        // Display each check item
        for item in &self.result.items {
            println!("{} {}: {}", item.icon(), item.name, item.message);
        }

        println!();

        // Display summary
        if self.result.all_critical_ok {
            println!("{}", self.result.summary_message().bright_green().bold());
        } else {
            println!("{}", self.result.summary_message().bright_red().bold());
        }
    }
}
