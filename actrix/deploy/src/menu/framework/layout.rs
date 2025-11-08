//! Layout system for organizing components

use super::components::Component;
use super::screen::Screen;

/// Layout trait for organizing page components
pub trait Layout {
    /// Render the complete layout with components
    fn render(&self, components: LayoutComponents);
}

/// Components that can be placed in a layout
pub struct LayoutComponents<'a> {
    pub tool_title: String,
    pub page_title: Option<String>,
    pub operation_hint: Option<String>,
    pub content: Vec<Box<dyn Component + 'a>>,
}

impl<'a> LayoutComponents<'a> {
    pub fn new(tool_title: impl Into<String>) -> Self {
        Self {
            tool_title: tool_title.into(),
            page_title: None,
            operation_hint: None,
            content: Vec::new(),
        }
    }

    pub fn with_page_title(mut self, title: impl Into<String>) -> Self {
        self.page_title = Some(title.into());
        self
    }

    pub fn with_operation_hint(mut self, hint: impl Into<String>) -> Self {
        self.operation_hint = Some(hint.into());
        self
    }

    pub fn add_content<C: Component + 'a>(mut self, component: C) -> Self {
        self.content.push(Box::new(component));
        self
    }
}

/// Standard layout implementation
pub struct StandardLayout;

impl StandardLayout {
    /// Clear the screen for a fresh view
    fn clear_screen(&self) {
        Screen::clear();
    }

    /// Render tool title with double line separator (weaker color)
    fn render_tool_title(&self, title: &str) {
        use colored::*;
        // Option 1: Blue (recommended) - 比白色弱但仍然清晰
        println!("{}", title.green());
        println!("{}", "═".repeat(title.len()).green());

        // 其他选项:
        // Option 2: Cyan - 青色，相对柔和
        // println!("{}", title.cyan());
        // println!("{}", "═".repeat(title.len()).cyan());

        // Option 3: Yellow - 黄色，比较醒目但不会超过白色
        // println!("{}", title.yellow());
        // println!("{}", "═".repeat(title.len()).yellow());

        // Option 4: Green - 绿色，中等强度
        // println!("{}", title.green());
        // println!("{}", "═".repeat(title.len()).green());

        // Option 5: Magenta - 品红色，有特色
        // println!("{}", title.magenta());
        // println!("{}", "═".repeat(title.len()).magenta());
    }

    /// Render page title with single line separator (stronger color)
    fn render_page_title(&self, title: &str) {
        use colored::*;
        println!("{}", title.bright_white().bold());
        println!("{}", "─".repeat(title.len()).bright_white());
    }

    /// Render operation hint in dimmed color
    fn render_operation_hint(&self, hint: &str) {
        use colored::*;
        println!("{}", hint.dimmed());
    }
}

impl Layout for StandardLayout {
    fn render(&self, components: LayoutComponents) {
        // Clear screen
        self.clear_screen();

        // 1. 工具标题 + 双线分割线
        self.render_tool_title(&components.tool_title);

        // 2. 页面标题或注释 + 单分割线 (如果有)
        if let Some(page_title) = &components.page_title {
            self.render_page_title(page_title);
        }

        // 3. 操作注释(暗色) (如果有)
        if let Some(hint) = &components.operation_hint {
            self.render_operation_hint(hint);
        }

        // 4. 空行
        println!();

        // 5. 用户操作区
        for component in components.content {
            component.render();
        }
    }
}
