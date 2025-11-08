//! Interactive configuration wizard for deployment settings

use anyhow::{Context, Result};
use dialoguer::{Input, Select, theme::ColorfulTheme};
use std::net::IpAddr;
use std::path::PathBuf;

use super::{DeploymentConfig, NetworkConfig, SslConfig, SystemConfig};
use crate::services::ServiceSelection;
use crate::system::{NetworkUtils, clear_input_buffer, validate_port, validate_username};
use crate::template::TemplateProcessor;

/// Interactive wizard for creating deployment configuration
pub struct ConfigWizard {
    debug: bool,
    theme: ColorfulTheme,
}

impl ConfigWizard {
    pub fn new(debug: bool) -> Self {
        Self {
            debug,
            theme: ColorfulTheme::default(),
        }
    }

    pub fn run(&mut self) -> Result<PathBuf> {
        // Step 0: Choose configuration file location
        let output_path = self.choose_config_location()?;

        // Don't clear screen - let the page layout handle it

        // Step 1: Service Selection
        let services = ServiceSelection::prompt_for_selection()?;
        self.log_debug(&format!("Selected services: {:?}", services.services));
        println!(
            "✅ Enable bitmask: 0b{:04b} (decimal: {})",
            services.bitmask, services.bitmask
        );
        println!();

        // Step 2: Network Configuration
        let network = self.configure_network(&services)?;

        // Step 3: SSL Configuration (if HTTP services are enabled)
        let ssl = if services.needs_http() {
            Some(self.configure_ssl(&network.server_host)?)
        } else {
            None
        };

        // Step 4: System Configuration
        let system = self.configure_system(&services, ssl.as_ref())?;

        // Step 5: Generate Configuration
        let config = DeploymentConfig {
            services,
            network,
            ssl,
            system,
            // install: InstallConfig::default(),
        };

        self.generate_config(&config, &output_path)?;
        println!("✅ Configuration generated successfully!");

        Ok(output_path)
    }

    fn configure_network(&self, services: &ServiceSelection) -> Result<NetworkConfig> {
        println!("🌐 Network Configuration");
        println!("=======================");

        // Server IP/domain selection
        let server_host = self.select_server_host()?;

        // Port configuration
        let ice_port = if services.needs_ice() {
            self.prompt_port("ICE service port (STUN/TURN)", 3478)?
        } else {
            3478 // default, not used
        };

        let https_port = if services.needs_http() {
            self.prompt_port("HTTPS API port", 8443)?
        } else {
            8443 // default, not used
        };

        println!();
        Ok(NetworkConfig {
            server_host,
            ice_port,
            https_port,
            http_port: 8080, // 默认 HTTP 端口
        })
    }

    fn select_server_host(&self) -> Result<String> {
        let local_ips = NetworkUtils::get_local_ips()?;
        let mut choices: Vec<String> = local_ips
            .iter()
            .map(|ip| format!("{} ({})", ip, self.classify_ip(ip)))
            .collect();

        choices.push("Enter custom IP/domain".to_string());

        println!("----------------------");
        println!("use ↑↓ arrow keys to navigate, enter to select, ctrl+c to exit");
        println!();

        // Clear input buffer before selection
        clear_input_buffer();

        let selection = Select::with_theme(&self.theme)
            .with_prompt("Select server IP or domain")
            .items(&choices)
            .default(0)
            .interact()?;

        if selection < local_ips.len() {
            Ok(local_ips[selection].to_string())
        } else {
            // Custom input
            println!("----------------------");
            println!("type custom IP/domain, enter to confirm, ctrl+c to exit");
            println!();

            let custom: String = Input::with_theme(&self.theme)
                .with_prompt("Enter custom IP or domain")
                .interact_text()?;
            Ok(custom)
        }
    }

    fn classify_ip(&self, ip: &IpAddr) -> &'static str {
        match ip {
            IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                if octets[0] == 127 {
                    "localhost"
                } else if octets[0] == 10
                    || (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31)
                    || (octets[0] == 192 && octets[1] == 168)
                {
                    "private IPv4"
                } else {
                    "public IPv4"
                }
            }
            IpAddr::V6(_) => "IPv6",
        }
    }

    fn prompt_port(&self, service: &str, default: u16) -> Result<u16> {
        println!("----------------------");
        println!("type port number or enter for default, ctrl+c to exit");
        println!();

        // Clear input buffer before port input
        clear_input_buffer();

        loop {
            let input: String = Input::with_theme(&self.theme)
                .with_prompt(format!("{} port (default: {})", service, default))
                .default(default.to_string())
                .interact_text()?;

            if input == default.to_string() {
                return Ok(default);
            }

            match input.parse::<u16>() {
                Ok(port) if validate_port(port) => return Ok(port),
                _ => println!("❌ Invalid port. Please enter a port number between 1 and 65535."),
            }
        }
    }

    fn configure_ssl(&self, _server_host: &str) -> Result<SslConfig> {
        println!("🔒 SSL Configuration");
        println!("===================");

        println!("----------------------");
        println!("type domain name or enter for default, ctrl+c to exit");
        println!();

        // Clear input buffer before domain input
        clear_input_buffer();

        let domain_name: String = Input::with_theme(&self.theme)
            .with_prompt("Domain name for SSL certificate")
            .default("example.com".to_string())
            .interact_text()?;

        let default_cert_path = format!("/etc/actor-rtc-actrix/ssl/{}/fullchain.pem", domain_name);
        println!("----------------------");
        println!("type certificate path or enter for default, ctrl+c to exit");
        println!();

        let cert_path: String = Input::with_theme(&self.theme)
            .with_prompt("SSL certificate path")
            .default(default_cert_path)
            .interact_text()?;

        let default_key_path = format!("/etc/actor-rtc-actrix/ssl/{}/privkey.pem", domain_name);
        println!("----------------------");
        println!("type private key path or enter for default, ctrl+c to exit");
        println!();

        let key_path: String = Input::with_theme(&self.theme)
            .with_prompt("SSL private key path")
            .default(default_key_path)
            .interact_text()?;

        println!();
        Ok(SslConfig {
            domain_name,
            cert_path: PathBuf::from(cert_path),
            key_path: PathBuf::from(key_path),
        })
    }

    fn configure_system(
        &self,
        services: &ServiceSelection,
        ssl_config: Option<&SslConfig>,
    ) -> Result<SystemConfig> {
        println!("⚙️  System Configuration");
        println!("=======================");

        let default_server_name = ssl_config
            .map(|ssl| format!("actrix-{}", ssl.domain_name))
            .unwrap_or_else(|| "actrix-server".to_string());

        println!("----------------------");
        println!("type server name or enter for default, ctrl+c to exit");
        println!();

        let server_name: String = Input::with_theme(&self.theme)
            .with_prompt("Server name/identifier")
            .default(default_server_name)
            .interact_text()?;

        println!("----------------------");
        println!("type location tag or enter for default, ctrl+c to exit");
        println!();

        let location_tag: String = Input::with_theme(&self.theme)
            .with_prompt("Location tag (e.g., aws,us-west-2)")
            .default("local".to_string())
            .interact_text()?;

        let run_user = self.prompt_username("Runtime user", "actor-rtc")?;
        let run_group = self.prompt_username("Runtime group", "actor-rtc")?;

        let turn_realm = if services.needs_turn() {
            println!("----------------------");
            println!("type TURN realm or enter for default, ctrl+c to exit");
            println!();

            let realm: String = Input::with_theme(&self.theme)
                .with_prompt("TURN realm (authentication domain)")
                .default("webrtc.rs".to_string())
                .interact_text()?;
            Some(realm)
        } else {
            None
        };

        println!();
        Ok(SystemConfig {
            server_name,
            location_tag,
            run_user,
            run_group,
            turn_realm,
        })
    }

    fn prompt_username(&self, prompt: &str, default: &str) -> Result<String> {
        println!("----------------------");
        println!("type username or enter for default, ctrl+c to exit");
        println!();

        // Clear input buffer before username input
        clear_input_buffer();

        loop {
            let input: String = Input::with_theme(&self.theme)
                .with_prompt(prompt)
                .default(default.to_string())
                .interact_text()?;

            if validate_username(&input) {
                return Ok(input);
            } else {
                println!(
                    "❌ Invalid username. Use alphanumeric characters, dash, underscore only (1-32 chars)."
                );
            }
        }
    }

    fn generate_config(&self, config: &DeploymentConfig, output_path: &PathBuf) -> Result<()> {
        println!("📝 Generating configuration file...");

        if self.debug {
            println!(
                "🐛 Debug mode: Configuration would be written to: {}",
                output_path.display()
            );
            return Ok(());
        }

        // Create parent directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Process template and write config file
        let mut processor = TemplateProcessor::new();
        processor.generate_config(config, output_path)?;

        Ok(())
    }

    fn choose_config_location(&self) -> Result<PathBuf> {
        println!("📁 Configuration File Location");
        println!("═══════════════════════════════");
        println!();

        // Clear input buffer to prevent fast keypress from affecting input
        clear_input_buffer();

        let default_path = PathBuf::from("/etc/actor-rtc-actrix/config.toml");

        let config_path: String = Input::with_theme(&self.theme)
            .with_prompt("Configuration file path")
            .default(default_path.to_string_lossy().to_string())
            .interact_text()?;

        let path = PathBuf::from(config_path);

        // Check if file already exists
        if path.exists() {
            println!("⚠️  Configuration file already exists: {}", path.display());
            let overwrite = dialoguer::Confirm::with_theme(&self.theme)
                .with_prompt("Overwrite existing file?")
                .default(false)
                .interact()?;

            if !overwrite {
                return Err(anyhow::anyhow!("config_file_exists"));
            }
        }

        // Check if parent directory exists, create if needed
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                println!("📂 Creating configuration directory: {}", parent.display());

                // Try to create directory first without sudo
                match std::fs::create_dir_all(parent) {
                    Ok(_) => {
                        println!("✅ Directory created successfully");
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                        // Permission denied, try with sudo
                        println!("⚠️  Permission denied, creating directory with sudo...");

                        let output = std::process::Command::new("sudo")
                            .args(["mkdir", "-p", &parent.to_string_lossy()])
                            .output()
                            .with_context(|| "Failed to execute sudo")?;

                        if output.status.success() {
                            println!("✅ Directory created with sudo: {}", parent.display());
                        } else {
                            let error = String::from_utf8_lossy(&output.stderr);
                            anyhow::bail!("Failed to create directory with sudo: {}", error);
                        }
                    }
                    Err(e) => {
                        return Err(e).with_context(|| {
                            format!("Failed to create directory: {}", parent.display())
                        });
                    }
                }
            }
        }

        println!("✅ Configuration file location: {}", path.display());
        println!();

        Ok(path)
    }

    fn log_debug(&self, message: &str) {
        if self.debug {
            println!("🐛 Debug: {}", message);
        }
    }
}
