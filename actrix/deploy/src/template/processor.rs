//! Template processing for configuration file generation

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use crate::config::DeploymentConfig;

const DEFAULT_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tpl/config.template.toml"
));

/// Template processor for generating configuration files
pub struct TemplateProcessor {
    template_path: Option<PathBuf>,
}

impl TemplateProcessor {
    pub fn new() -> Self {
        Self {
            template_path: None,
        }
    }

    #[allow(unused)]
    pub fn with_template_path(template_path: PathBuf) -> Self {
        Self {
            template_path: Some(template_path),
        }
    }

    pub fn generate_config(
        &mut self,
        config: &DeploymentConfig,
        output_path: &PathBuf,
    ) -> Result<()> {
        // Load template
        let template = self.load_template()?;

        // Create placeholder map
        let placeholders = self.create_placeholders(config);

        // Process template
        let processed = self.process_template(&template, &placeholders);

        // Write configuration file
        self.write_config(&processed, output_path)?;

        Ok(())
    }

    fn load_template(&self) -> Result<String> {
        if let Some(path) = &self.template_path {
            std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read template from {}", path.display()))
        } else {
            Ok(DEFAULT_TEMPLATE.to_string())
        }
    }

    fn create_placeholders(&self, config: &DeploymentConfig) -> HashMap<String, String> {
        let mut placeholders = HashMap::new();

        // Service enable bitmask
        placeholders.insert(
            "ENABLE_BITMASK".to_string(),
            config.services.bitmask.to_string(),
        );
        placeholders.insert(
            "ENABLE_BITMASK_BIN".to_string(),
            format!("0b{:04b}", config.services.bitmask),
        );

        // Network configuration
        placeholders.insert(
            "SERVER_HOST".to_string(),
            config.network.server_host.clone(),
        );
        placeholders.insert("ICE_PORT".to_string(), config.network.ice_port.to_string());
        placeholders.insert(
            "HTTPS_PORT".to_string(),
            config.network.https_port.to_string(),
        );

        // SSL configuration
        if let Some(ssl) = &config.ssl {
            placeholders.insert(
                "SSL_CERT_PATH".to_string(),
                ssl.cert_path.display().to_string(),
            );
            placeholders.insert(
                "SSL_KEY_PATH".to_string(),
                ssl.key_path.display().to_string(),
            );
        } else {
            // Default values for non-SSL setup
            placeholders.insert("SSL_CERT_PATH".to_string(), "/path/to/cert.pem".to_string());
            placeholders.insert("SSL_KEY_PATH".to_string(), "/path/to/key.pem".to_string());
        }

        // System configuration
        placeholders.insert("SERVER_NAME".to_string(), config.system.server_name.clone());
        placeholders.insert(
            "LOCATION_TAG".to_string(),
            config.system.location_tag.clone(),
        );
        placeholders.insert("RUN_USER".to_string(), config.system.run_user.clone());
        placeholders.insert("RUN_GROUP".to_string(), config.system.run_group.clone());

        // TURN realm
        if let Some(realm) = &config.system.turn_realm {
            placeholders.insert("TURN_REALM".to_string(), realm.clone());
        } else {
            placeholders.insert("TURN_REALM".to_string(), "webrtc.rs".to_string());
        }

        placeholders
    }

    fn process_template(&self, template: &str, placeholders: &HashMap<String, String>) -> String {
        let mut result = template.to_string();

        for (key, value) in placeholders {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }

    fn write_config(&self, content: &str, output_path: &PathBuf) -> Result<()> {
        // Check if we need sudo for system paths
        let needs_sudo = output_path.starts_with("/etc") || output_path.starts_with("/opt");

        if needs_sudo {
            // Use sudo with tee to write the file
            let mut cmd = Command::new("sudo");
            cmd.arg("tee")
                .arg(output_path)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped());

            let mut child = cmd.spawn()?;

            if let Some(stdin) = child.stdin.as_mut() {
                use std::io::Write;
                stdin.write_all(content.as_bytes())?;
            }

            let output = child.wait_with_output()?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to write config file: {}", error);
            }
        } else {
            // Direct write for user paths
            std::fs::write(output_path, content)
                .with_context(|| format!("Failed to write config to {}", output_path.display()))?;
        }

        Ok(())
    }
}
