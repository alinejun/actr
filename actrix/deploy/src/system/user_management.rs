//! User and group management utilities

use anyhow::Result;
use dialoguer::{Confirm, theme::ColorfulTheme};
use std::process::Command;

#[cfg(unix)]
use users::{get_group_by_name, get_user_by_name};

use super::helpers::show_confirm_help;

/// Ensure system user exists, create if needed
#[cfg(unix)]
#[allow(unused)]
pub fn ensure_user_exists(username: &str, is_system: bool, home_dir: Option<&str>) -> Result<()> {
    if get_user_by_name(username).is_some() {
        println!("✅ User '{}' already exists", username);
        return Ok(());
    }

    let theme = ColorfulTheme::default();
    show_confirm_help();

    if !Confirm::with_theme(&theme)
        .with_prompt(&format!("User '{}' does not exist. Create it?", username))
        .default(true)
        .interact()?
    {
        println!("⚠️  Please create user '{}' manually", username);
        return Ok(());
    }

    create_user(username, is_system, home_dir)
}

/// Ensure system group exists, create if needed
#[cfg(unix)]
#[allow(unused)]
pub fn ensure_group_exists(groupname: &str) -> Result<()> {
    if get_group_by_name(groupname).is_some() {
        println!("✅ Group '{}' already exists", groupname);
        return Ok(());
    }

    let theme = ColorfulTheme::default();
    show_confirm_help();

    if !Confirm::with_theme(&theme)
        .with_prompt(&format!("Group '{}' does not exist. Create it?", groupname))
        .default(true)
        .interact()?
    {
        println!("⚠️  Please create group '{}' manually", groupname);
        return Ok(());
    }

    create_group(groupname)
}

/// Stub for non-Unix systems
#[cfg(not(unix))]
pub fn ensure_user_exists(
    _username: &str,
    _is_system: bool,
    _home_dir: Option<&str>,
) -> Result<()> {
    println!("⚠️  User creation not supported on this platform");
    Ok(())
}

/// Stub for non-Unix systems
#[cfg(not(unix))]
pub fn ensure_group_exists(_groupname: &str) -> Result<()> {
    println!("⚠️  Group creation not supported on this platform");
    Ok(())
}

#[cfg(unix)]
#[allow(unused)]
fn create_user(username: &str, is_system: bool, home_dir: Option<&str>) -> Result<()> {
    let mut cmd = Command::new("sudo");
    cmd.arg("useradd");

    if is_system {
        cmd.arg("--system");
        cmd.arg("--shell").arg("/bin/false");
    }

    if let Some(home) = home_dir {
        cmd.arg("--home").arg(home);
    }

    cmd.arg(username);

    let output = cmd.output()?;

    if output.status.success() {
        println!("✅ User '{}' created successfully", username);
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create user '{}': {}", username, error);
    }
}

#[cfg(unix)]
#[allow(unused)]
fn create_group(groupname: &str) -> Result<()> {
    let output = Command::new("sudo")
        .args(["groupadd", "--system", groupname])
        .output()?;

    if output.status.success() {
        println!("✅ Group '{}' created successfully", groupname);
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create group '{}': {}", groupname, error);
    }
}
