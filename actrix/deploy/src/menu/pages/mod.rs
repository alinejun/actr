//! Concrete page implementations

pub mod config;
pub mod config_page;
pub mod dependencies_page;
pub mod install_page;
pub mod main_page;
pub mod systemd_install_page;
pub mod uninstall_page;
pub mod wizard_page;

pub use config_page::ConfigPage;
pub use dependencies_page::DependenciesPage;
pub use install_page::InstallPage;
pub use main_page::MainPage;
pub use systemd_install_page::SystemdInstallPage;
pub use uninstall_page::UninstallPage;
pub use wizard_page::WizardPage;
