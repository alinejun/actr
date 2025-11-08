//! System check result structures

/// Result of a single dependency check
#[derive(Debug, Clone)]
pub struct CheckItem {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
}

/// Status of a dependency check
#[derive(Debug, Clone, PartialEq)]
pub enum CheckStatus {
    Ok,
    Warning,
    Error,
}

impl CheckItem {
    pub fn ok(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Ok,
            message: message.into(),
        }
    }

    pub fn warning(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Warning,
            message: message.into(),
        }
    }

    pub fn error(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Error,
            message: message.into(),
        }
    }

    /// Get the status icon
    pub fn icon(&self) -> &'static str {
        match self.status {
            CheckStatus::Ok => "✅",
            CheckStatus::Warning => "⚠️ ",
            CheckStatus::Error => "❌",
        }
    }
}

/// Overall dependency check result
#[derive(Debug)]
pub struct DependencyCheckResult {
    pub items: Vec<CheckItem>,
    pub all_critical_ok: bool,
}

impl DependencyCheckResult {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            all_critical_ok: true,
        }
    }

    pub fn add_item(&mut self, item: CheckItem) {
        if item.status == CheckStatus::Error {
            self.all_critical_ok = false;
        }
        self.items.push(item);
    }

    pub fn summary_message(&self) -> String {
        if self.all_critical_ok {
            "🎉 All essential dependencies are satisfied!".to_string()
        } else {
            "❌ Some critical dependencies are missing!".to_string()
        }
    }
}
