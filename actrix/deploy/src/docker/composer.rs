//! Docker Compose 配置生成器实现

use actrix_common::config::ActrixConfig;
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

/// Docker Compose 配置生成器
pub struct DockerComposeGenerator {
    config: ActrixConfig,
}

impl DockerComposeGenerator {
    /// 从配置文件创建生成器
    pub fn from_config_file(config_path: &Path) -> Result<Self> {
        let config_content = fs::read_to_string(config_path)
            .with_context(|| format!("无法读取配置文件: {}", config_path.display()))?;

        let config: ActrixConfig =
            toml::from_str(&config_content).with_context(|| "解析配置文件失败")?;

        Ok(Self { config })
    }

    /// 生成 docker-compose.yml 内容
    pub fn generate(&self) -> Result<String> {
        let mut compose = json!({
            "version": "3.8",
            "services": {},
            "networks": {
                "actrix-network": {
                    "driver": "bridge"
                }
            },
            "volumes": {
                "actrix-data": {},
                "actrix-certs": {}
            }
        });

        // 生成主服务
        let main_service = self.generate_main_service()?;
        compose["services"]["actrix"] = main_service;

        // 转换为 YAML
        let yaml = serde_yaml::to_string(&compose).with_context(|| "转换为 YAML 失败")?;

        Ok(yaml)
    }

    /// 生成主 Actrix 服务配置
    fn generate_main_service(&self) -> Result<Value> {
        let mut ports = Vec::new();
        let mut environment = Vec::new();

        // HTTP/HTTPS 端口
        if let Some(ref http) = self.config.bind.http {
            ports.push(format!("{}:{}", http.port, http.port));
        }
        if let Some(ref https) = self.config.bind.https {
            ports.push(format!("{}:{}", https.port, https.port));
        }

        // ICE 端口 (STUN/TURN)
        let ice = &self.config.bind.ice;
        ports.push(format!("{}:{}/udp", ice.port, ice.port));

        // TURN relay 端口范围
        if self.config.is_turn_enabled() {
            let turn = &self.config.turn;
            // 解析端口范围
            if let Some((start, end)) = Self::parse_port_range(&turn.relay_port_range) {
                ports.push(format!("{}-{}:{}-{}/udp", start, end, start, end));
            }
        }

        // 环境变量
        if let Ok(kek) = std::env::var("ACTRIX_KEK") {
            environment.push(format!("ACTRIX_KEK={}", kek));
        }

        let service = json!({
            "image": "actrix:latest",
            "container_name": "actrix",
            "restart": "unless-stopped",
            "ports": ports,
            "environment": environment,
            "volumes": [
                "./config.toml:/app/config.toml:ro",
                "actrix-data:/app/data",
                "actrix-certs:/app/certificates:ro"
            ],
            "networks": ["actrix-network"],
            "command": ["--config", "/app/config.toml"]
        });

        Ok(service)
    }

    /// 解析端口范围字符串（如 "49152-65535"）
    fn parse_port_range(range: &str) -> Option<(u16, u16)> {
        let parts: Vec<&str> = range.split('-').collect();
        if parts.len() == 2 {
            if let (Ok(start), Ok(end)) = (parts[0].parse::<u16>(), parts[1].parse::<u16>()) {
                return Some((start, end));
            }
        }
        None
    }

    /// 保存到文件
    pub fn save_to_file(&self, output_path: &Path) -> Result<()> {
        let content = self.generate()?;
        fs::write(output_path, content)
            .with_context(|| format!("无法写入文件: {}", output_path.display()))?;

        println!("✅ Docker Compose 配置已生成: {}", output_path.display());
        Ok(())
    }
}
