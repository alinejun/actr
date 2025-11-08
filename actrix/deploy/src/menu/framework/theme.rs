//! Theme system for consistent styling

use dialoguer::theme::ColorfulTheme;

/// Theme trait for customizing UI appearance
pub trait Theme {
    /// Get dialoguer theme
    fn dialoguer_theme(&self) -> &ColorfulTheme;

    /// Get primary color name
    #[allow(unused)]
    fn primary_color(&self) -> &str;

    /// Get secondary color name
    #[allow(unused)]
    fn secondary_color(&self) -> &str;

    /// Get success color name
    #[allow(unused)]
    fn success_color(&self) -> &str;

    /// Get error color name
    #[allow(unused)]
    fn error_color(&self) -> &str;
}

/// Default theme implementation
pub struct DefaultTheme {
    dialoguer: ColorfulTheme,
}

impl Default for DefaultTheme {
    fn default() -> Self {
        Self {
            dialoguer: ColorfulTheme::default(),
        }
    }
}

impl Theme for DefaultTheme {
    fn dialoguer_theme(&self) -> &ColorfulTheme {
        &self.dialoguer
    }

    fn primary_color(&self) -> &str {
        "bright_cyan"
    }

    fn secondary_color(&self) -> &str {
        "bright_yellow"
    }

    fn success_color(&self) -> &str {
        "bright_green"
    }

    fn error_color(&self) -> &str {
        "bright_red"
    }
}
