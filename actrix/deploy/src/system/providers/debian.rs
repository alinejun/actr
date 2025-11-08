//! Debian/Ubuntu 系统提供者实现
//!
//! 实现基于 apt 包管理器和 systemd 的 Debian 系发行版支持

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::system::provider::{DependencyStatus, FirewallProtocol, ServiceStatus, SystemProvider};

/// Debian/Ubuntu 系统提供者
pub struct DebianProvider;

impl DebianProvider {
    pub fn new() -> Self {
        Self
    }

    /// 执行 sudo 命令
    fn sudo_command(&self, command: &str, args: &[&str]) -> Result<std::process::Output> {
        Command::new("sudo")
            .arg(command)
            .args(args)
            .output()
            .with_context(|| format!("Failed to execute sudo {} {}", command, args.join(" ")))
    }

    /// 执行命令并检查是否成功
    fn run_command(&self, command: &str, args: &[&str]) -> Result<()> {
        let output = Command::new(command)
            .args(args)
            .output()
            .with_context(|| format!("Failed to execute {} {}", command, args.join(" ")))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Command failed: {}", stderr))
        }
    }

    /// 执行 sudo 命令并检查是否成功
    fn run_sudo_command(&self, command: &str, args: &[&str]) -> Result<()> {
        let output = self.sudo_command(command, args)?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Sudo command failed: {}", stderr))
        }
    }
}

impl SystemProvider for DebianProvider {
    fn name(&self) -> &'static str {
        "Debian/Ubuntu"
    }

    fn description(&self) -> &'static str {
        "Debian-based Linux distribution (Ubuntu, Debian, Mint, etc.)"
    }

    // ========== 依赖检查 ==========
    fn check_dependencies(&self, dependencies: &[&str]) -> Result<Vec<DependencyStatus>> {
        let mut results = Vec::new();

        for &dep in dependencies {
            let available = self.command_exists(dep);
            let status = if available {
                DependencyStatus::available(dep)
            } else {
                DependencyStatus::missing(dep)
            };
            results.push(status);
        }

        Ok(results)
    }

    fn command_exists(&self, command: &str) -> bool {
        Command::new("which")
            .arg(command)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn has_package_manager(&self) -> bool {
        self.command_exists("apt-get")
    }

    fn has_systemd(&self) -> bool {
        Path::new("/run/systemd/system").exists() || self.command_exists("systemctl")
    }

    // ========== 包管理 ==========
    fn install_packages(&self, packages: &[&str]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        println!("🔄 Installing packages: {}", packages.join(", "));

        // 首先更新包索引
        self.update_package_index()?;

        // 安装包
        let mut args = vec!["-y", "install"];
        args.extend(packages);

        self.run_sudo_command("apt-get", &args)
            .with_context(|| format!("Failed to install packages: {}", packages.join(", ")))?;

        println!("✅ Packages installed successfully");
        Ok(())
    }

    fn update_package_index(&self) -> Result<()> {
        println!("🔄 Updating package index...");
        self.run_sudo_command("apt-get", &["update"])
            .context("Failed to update package index")?;
        println!("✅ Package index updated");
        Ok(())
    }

    fn is_package_installed(&self, package: &str) -> bool {
        Command::new("dpkg")
            .args(["-l", package])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    // ========== 用户和组管理 ==========
    fn create_system_user(&self, username: &str, home_dir: Option<&str>) -> Result<()> {
        if self.user_exists(username) {
            println!("✅ User '{}' already exists", username);
            return Ok(());
        }

        println!("🔄 Creating system user: {}", username);

        let mut args = vec!["--system", "--no-create-home"];

        if let Some(home) = home_dir {
            args.extend(["--home-dir", home]);
        }

        args.push(username);

        self.run_sudo_command("useradd", &args)
            .with_context(|| format!("Failed to create user: {}", username))?;

        println!("✅ User '{}' created successfully", username);
        Ok(())
    }

    fn create_system_group(&self, groupname: &str) -> Result<()> {
        if self.group_exists(groupname) {
            println!("✅ Group '{}' already exists", groupname);
            return Ok(());
        }

        println!("🔄 Creating system group: {}", groupname);

        self.run_sudo_command("groupadd", &["--system", groupname])
            .with_context(|| format!("Failed to create group: {}", groupname))?;

        println!("✅ Group '{}' created successfully", groupname);
        Ok(())
    }

    fn user_exists(&self, username: &str) -> bool {
        Command::new("id")
            .arg(username)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn group_exists(&self, groupname: &str) -> bool {
        Command::new("getent")
            .args(["group", groupname])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn add_user_to_group(&self, username: &str, groupname: &str) -> Result<()> {
        println!("🔄 Adding user '{}' to group '{}'", username, groupname);

        self.run_sudo_command("usermod", &["-a", "-G", groupname, username])
            .with_context(|| {
                format!("Failed to add user '{}' to group '{}'", username, groupname)
            })?;

        println!("✅ User '{}' added to group '{}'", username, groupname);
        Ok(())
    }

    // ========== 服务管理 ==========
    fn install_systemd_service(&self, service_name: &str, service_content: &str) -> Result<()> {
        if !self.has_systemd() {
            return Err(anyhow::anyhow!("systemd is not available on this system"));
        }

        let service_path = format!("/etc/systemd/system/{}.service", service_name);

        println!("🔄 Installing systemd service: {}", service_name);

        // 写入服务文件
        std::fs::write(&service_path, service_content)
            .or_else(|_| {
                // 如果直接写入失败，尝试使用 sudo
                let temp_file = format!("/tmp/{}.service", service_name);
                std::fs::write(&temp_file, service_content)?;
                self.run_sudo_command("mv", &[&temp_file, &service_path])?;
                Ok::<(), anyhow::Error>(())
            })
            .with_context(|| format!("Failed to write service file: {}", service_path))?;

        // 重新加载 systemd
        self.run_sudo_command("systemctl", &["daemon-reload"])
            .context("Failed to reload systemd daemon")?;

        println!("✅ Service '{}' installed successfully", service_name);
        Ok(())
    }

    fn enable_service(&self, service_name: &str) -> Result<()> {
        println!("🔄 Enabling service: {}", service_name);

        self.run_sudo_command("systemctl", &["enable", service_name])
            .with_context(|| format!("Failed to enable service: {}", service_name))?;

        println!("✅ Service '{}' enabled", service_name);
        Ok(())
    }

    fn start_service(&self, service_name: &str) -> Result<()> {
        println!("🔄 Starting service: {}", service_name);

        self.run_sudo_command("systemctl", &["start", service_name])
            .with_context(|| format!("Failed to start service: {}", service_name))?;

        println!("✅ Service '{}' started", service_name);
        Ok(())
    }

    fn stop_service(&self, service_name: &str) -> Result<()> {
        println!("🔄 Stopping service: {}", service_name);

        self.run_sudo_command("systemctl", &["stop", service_name])
            .with_context(|| format!("Failed to stop service: {}", service_name))?;

        println!("✅ Service '{}' stopped", service_name);
        Ok(())
    }

    fn restart_service(&self, service_name: &str) -> Result<()> {
        println!("🔄 Restarting service: {}", service_name);

        self.run_sudo_command("systemctl", &["restart", service_name])
            .with_context(|| format!("Failed to restart service: {}", service_name))?;

        println!("✅ Service '{}' restarted", service_name);
        Ok(())
    }

    fn service_status(&self, service_name: &str) -> Result<ServiceStatus> {
        let output = Command::new("systemctl")
            .args(["is-active", service_name])
            .output()
            .context("Failed to check service status")?;

        let status_str = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_lowercase();

        let status = match status_str.as_str() {
            "active" => ServiceStatus::Running,
            "inactive" => ServiceStatus::Stopped,
            "failed" => {
                let error = Command::new("systemctl")
                    .args(["status", service_name])
                    .output()
                    .map(|output| String::from_utf8_lossy(&output.stderr).to_string())
                    .unwrap_or_else(|_| "Failed to get service status details".to_string());
                ServiceStatus::Failed(error)
            }
            _ => ServiceStatus::Unknown,
        };

        Ok(status)
    }

    // ========== 文件和权限管理 ==========
    fn create_directory(&self, path: &Path, mode: Option<u32>) -> Result<()> {
        if path.exists() {
            return Ok(());
        }

        println!("🔄 Creating directory: {}", path.display());

        // 尝试直接创建
        if let Ok(()) = std::fs::create_dir_all(path) {
            if let Some(mode) = mode {
                self.set_file_permissions(path, mode)?;
            }
            return Ok(());
        }

        // 如果失败，尝试使用 sudo
        self.run_sudo_command("mkdir", &["-p", &path.to_string_lossy()])
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;

        if let Some(mode) = mode {
            self.set_file_permissions(path, mode)?;
        }

        println!("✅ Directory created: {}", path.display());
        Ok(())
    }

    fn set_file_owner(&self, path: &Path, user: &str, group: &str) -> Result<()> {
        let owner = format!("{}:{}", user, group);

        self.run_sudo_command("chown", &[&owner, &path.to_string_lossy()])
            .with_context(|| format!("Failed to set owner of {}", path.display()))?;

        Ok(())
    }

    fn set_file_permissions(&self, path: &Path, mode: u32) -> Result<()> {
        let mode_str = format!("{:o}", mode);

        self.run_sudo_command("chmod", &[&mode_str, &path.to_string_lossy()])
            .with_context(|| format!("Failed to set permissions of {}", path.display()))?;

        Ok(())
    }

    fn copy_file(&self, src: &Path, dst: &Path) -> Result<()> {
        // 尝试直接复制
        if std::fs::copy(src, dst).is_ok() {
            return Ok(());
        }

        // 如果失败，尝试使用 sudo
        self.run_sudo_command("cp", &[&src.to_string_lossy(), &dst.to_string_lossy()])
            .with_context(|| format!("Failed to copy {} to {}", src.display(), dst.display()))?;

        Ok(())
    }

    // ========== 网络和防火墙 ==========
    fn is_port_available(&self, port: u16) -> bool {
        use std::net::{TcpListener, UdpSocket};

        // 检查 TCP 端口
        let tcp_available = TcpListener::bind(("127.0.0.1", port)).is_ok();

        // 检查 UDP 端口
        let udp_available = UdpSocket::bind(("127.0.0.1", port)).is_ok();

        tcp_available && udp_available
    }

    fn configure_firewall(&self, port: u16, protocol: FirewallProtocol) -> Result<()> {
        // 检查是否有 ufw (Ubuntu Firewall)
        if self.command_exists("ufw") {
            let protocol_str = match protocol {
                FirewallProtocol::Tcp => "tcp",
                FirewallProtocol::Udp => "udp",
                FirewallProtocol::Both => {
                    return {
                        self.configure_firewall(port, FirewallProtocol::Tcp)?;
                        self.configure_firewall(port, FirewallProtocol::Udp)?;
                        Ok(())
                    };
                }
            };

            let rule = format!("{}/{}", port, protocol_str);

            println!("🔄 Configuring firewall rule: allow {}", rule);

            self.run_sudo_command("ufw", &["allow", &rule])
                .with_context(|| format!("Failed to configure firewall for port {}", port))?;

            println!("✅ Firewall rule added: allow {}", rule);
            return Ok(());
        }

        // 检查是否有 iptables
        if self.command_exists("iptables") {
            println!("⚠️  iptables detected but automatic configuration not implemented");
            println!(
                "   Please manually configure firewall rules for port {}",
                port
            );
            return Ok(());
        }

        println!("⚠️  No supported firewall found (ufw, iptables)");
        println!(
            "   Please manually configure firewall rules for port {}",
            port
        );
        Ok(())
    }

    // ========== 系统信息 ==========
    fn system_arch(&self) -> String {
        Command::new("uname")
            .arg("-m")
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    }

    fn system_version(&self) -> String {
        std::fs::read_to_string("/etc/os-release")
            .unwrap_or_default()
            .lines()
            .find(|line| line.starts_with("PRETTY_NAME="))
            .and_then(|line| line.split('=').nth(1))
            .map(|name| name.trim_matches('"').to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    fn is_container(&self) -> bool {
        // 检查常见的容器环境标识
        Path::new("/.dockerenv").exists()
            || std::env::var("container").is_ok()
            || std::fs::read_to_string("/proc/1/cgroup")
                .map(|content| content.contains("docker") || content.contains("lxc"))
                .unwrap_or(false)
    }

    fn has_sudo_access(&self) -> bool {
        Command::new("sudo")
            .args(["-n", "true"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}
