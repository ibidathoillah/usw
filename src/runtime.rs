use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::{info, debug};
use crate::error::UswitchError;

const SERVICE_TEMPLATE: &str = "/etc/systemd/system/ai-runtime@.service";
const TEMPLATE_CONTENT: &str = include_str!("../templates/ai-runtime@.service");

pub fn install_service_template() -> Result<(), UswitchError> {
    let path = Path::new(SERVICE_TEMPLATE);
    let mut updated = false;

    if !path.exists() {
        updated = true;
    } else {
        let existing = fs::read_to_string(path).unwrap_or_default();
        if existing != TEMPLATE_CONTENT {
            updated = true;
        }
    }

    if updated {
        fs::write(path, TEMPLATE_CONTENT).map_err(|e| {
            UswitchError::CommandFailed(
                format!("write {SERVICE_TEMPLATE}"),
                e.to_string(),
            )
        })?;

        let output = Command::new("systemctl")
            .args(["daemon-reload"])
            .output()
            .map_err(|e| {
                UswitchError::CommandFailed(
                    "systemctl daemon-reload".to_string(),
                    e.to_string(),
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(UswitchError::Systemd(format!(
                "daemon-reload failed: {stderr}"
            )));
        }

        info!("Systemd service template installed/updated");
    }
    Ok(())
}

pub fn start_runtime(username: &str) -> Result<(), UswitchError> {
    install_service_template()?;

    let service_name = format!("ai-runtime@{username}.service");

    debug!(service = %service_name, "Starting systemd service (async)");

    let output = Command::new("systemctl")
        .args(["start", &service_name])
        .output()
        .map_err(|e| {
            UswitchError::CommandFailed(
                format!("systemctl start {service_name}"),
                e.to_string(),
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(UswitchError::Systemd(format!(
            "failed to start {service_name}: {stderr}"
        )));
    }

    info!(user = %username, "Runtime start triggered");

    Ok(())
}

pub fn stop_runtime(username: &str) -> Result<(), UswitchError> {
    let service_name = format!("ai-runtime@{username}.service");

    debug!(service = %service_name, "Stopping systemd service");

    let output = Command::new("systemctl")
        .args(["stop", &service_name])
        .output()
        .map_err(|e| {
            UswitchError::CommandFailed(
                format!("systemctl stop {service_name}"),
                e.to_string(),
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(UswitchError::Systemd(format!(
            "failed to stop {service_name}: {stderr}"
        )));
    }

    info!(user = %username, "Runtime stopped");

    Ok(())
}

pub fn restart_runtime(username: &str) -> Result<(), UswitchError> {
    stop_runtime(username)?;
    start_runtime(username)
}

pub fn runtime_is_active(username: &str) -> bool {
    let service_name = format!("ai-runtime@{username}.service");
    let output = Command::new("systemctl")
        .args(["is-active", "--quiet", &service_name])
        .status();

    match output {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

pub fn get_logs(
    username: &str,
    lines: usize,
    follow: bool,
) -> Result<(), UswitchError> {
    let service_name = format!("ai-runtime@{username}.service");

    let mut args = vec![
        "-u".to_string(),
        service_name.clone(),
        "--no-pager".to_string(),
        "-n".to_string(),
        lines.to_string(),
    ];

    if follow {
        args.push("-f".to_string());
    }

    debug!(service = %service_name, lines = lines, follow, "Fetching logs");

    let status = Command::new("journalctl")
        .args(&args)
        .status()
        .map_err(|e| {
            UswitchError::CommandFailed(
                format!("journalctl {}", args.join(" ")),
                e.to_string(),
            )
        })?;

    if !status.success() {
        return Err(UswitchError::Systemd(format!(
            "failed to get logs for {service_name}"
        )));
    }

    Ok(())
}

pub fn disable_service_instance(username: &str) -> Result<(), UswitchError> {
    let service_name = format!("ai-runtime@{username}.service");

    let _ = Command::new("systemctl")
        .args(["disable", &service_name])
        .output();

    Ok(())
}
