//! Service selection with bitmask calculation

use anyhow::Result;
use dialoguer::{MultiSelect, theme::ColorfulTheme};

use super::ServiceType;

/// Selected services with calculated bitmask
#[derive(Debug, Clone)]
pub struct ServiceSelection {
    pub services: Vec<ServiceType>,
    pub bitmask: u8,
}

impl ServiceSelection {
    pub fn new(services: Vec<ServiceType>) -> Self {
        let mut bitmask = 0;
        let mut filtered_services = services;

        // If TURN is selected, remove STUN (TURN includes STUN functionality)
        if filtered_services.contains(&ServiceType::Turn)
            && filtered_services.contains(&ServiceType::Stun)
        {
            filtered_services.retain(|&s| s != ServiceType::Stun);
            println!("⚠️ TURN service includes STUN functionality. Standalone STUN disabled.");
        }

        for service in &filtered_services {
            bitmask |= *service as u8;
        }

        Self {
            services: filtered_services,
            bitmask,
        }
    }

    pub fn needs_http(&self) -> bool {
        self.services.iter().any(|s| s.needs_http())
    }

    pub fn needs_ice(&self) -> bool {
        self.services.iter().any(|s| s.needs_ice())
    }

    pub fn needs_turn(&self) -> bool {
        self.services.contains(&ServiceType::Turn)
    }

    pub fn prompt_for_selection() -> Result<Self> {
        let theme = ColorfulTheme::default();
        let services = ServiceType::all();
        let service_names: Vec<String> = services
            .iter()
            .map(|s| format!("{} - {}", s, s.description()))
            .collect();

        loop {
            let selections = MultiSelect::with_theme(&theme)
                .with_prompt("Select services to enable")
                .items(&service_names)
                .interact()?;

            if selections.is_empty() {
                println!("⚠️  At least one service must be selected. Please try again.");
                println!();
                continue;
            }

            return Ok(Self::from_selection(selections, &services));
        }
    }

    fn from_selection(selections: Vec<usize>, services: &[ServiceType]) -> Self {
        let selected_services: Vec<ServiceType> =
            selections.into_iter().map(|i| services[i]).collect();

        Self::new(selected_services)
    }
}
