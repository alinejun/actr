//! Dependencies check page

use crate::menu::framework::{
    DefaultTheme, DependencyCheckComponent, Layout, LayoutComponents, Page, PageContext,
    PageResult, StandardLayout,
};
use crate::system::check_dependencies_data;
use crate::system::press_any_key_to_with_interrupt;
use anyhow::Result;

pub struct DependenciesPage {
    layout: StandardLayout,
    theme: DefaultTheme,
}

impl DependenciesPage {
    pub fn new() -> Self {
        Self {
            layout: StandardLayout,
            theme: DefaultTheme::default(),
        }
    }
}

impl Page for DependenciesPage {
    fn title(&self) -> &str {
        "Check Dependencies"
    }

    fn render(&mut self, context: &mut PageContext) -> Result<PageResult> {
        // Get dependency check results
        let check_result = check_dependencies_data()?;

        // Build layout components using new standard format
        let components = LayoutComponents::new("ActorRTC Auxiliary Services Deployment Helper")
            .with_page_title("System Dependencies Check")
            .with_operation_hint("Checking system compatibility and required tools...")
            .add_content(DependencyCheckComponent::new(check_result));

        // Render the layout
        self.layout.render(components);

        let interrupted = press_any_key_to_with_interrupt("continue", context.interrupted.clone());
        if interrupted {
            Ok(PageResult::Stay) // Let MenuApplication handle Ctrl+C
        } else {
            Ok(PageResult::Back)
        }
    }
}
