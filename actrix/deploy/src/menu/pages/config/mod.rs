//! Configuration wizard pages

pub mod config_location_page;
pub mod network_config_page;
pub mod service_selection_page;
pub mod ssl_config_page;
pub mod summary_page;
pub mod system_config_page;

pub use config_location_page::ConfigLocationPage;
pub use network_config_page::NetworkConfigPage;
pub use service_selection_page::ServiceSelectionPage;
// ssl_config_page::SslConfigPage is now integrated into NetworkConfigPage
pub use summary_page::SummaryPage;
pub use system_config_page::SystemConfigPage;
