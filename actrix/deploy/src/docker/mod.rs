//! Docker Compose 配置生成器
//!
//! 从 Actrix 配置文件生成 docker-compose.yml

mod composer;

pub use composer::DockerComposeGenerator;

use anyhow::Result;
use std::path::Path;
use tokio::process::Command;

/// 执行 docker compose up -d
pub async fn docker_compose_up(compose_file: &Path, legacy: bool) -> Result<()> {
    let (cmd, args) = if legacy {
        (
            "docker-compose",
            vec!["-f", compose_file.to_str().unwrap(), "up", "-d"],
        )
    } else {
        (
            "docker",
            vec!["compose", "-f", compose_file.to_str().unwrap(), "up", "-d"],
        )
    };

    println!(
        "🐳 执行 {} ...",
        if legacy {
            "docker-compose"
        } else {
            "docker compose"
        }
    );

    let output = Command::new(cmd).args(&args).output().await?;

    if output.status.success() {
        println!("✅ Docker Compose 启动成功");
        if !output.stdout.is_empty() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Docker Compose 启动失败:\n{}", stderr);
    }
}

/// 检查 docker 命令是否可用
pub async fn check_docker_available(legacy: bool) -> Result<bool> {
    let (cmd, args) = if legacy {
        ("docker-compose", vec!["--version"])
    } else {
        ("docker", vec!["compose", "version"])
    };

    let result = Command::new(cmd).args(&args).output().await;

    Ok(result.is_ok() && result.unwrap().status.success())
}
