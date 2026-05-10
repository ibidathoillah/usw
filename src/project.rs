use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, debug};
use crate::error::UswitchError;

/// Attach a workspace directory to a user.
/// The source path (e.g. cwd) gets ACL for the user, and a bind mount
/// is created at /home/<user>/projects/<name> → source_path.
pub fn attach_workspace(username: &str, project_name: &str, source_path: &Path) -> Result<(), UswitchError> {
    if !source_path.exists() {
        return Err(UswitchError::ProjectNotFound(source_path.display().to_string()));
    }

    // ── Set ACL so the user can rwx (non-recursive, dir only) ──
    // Use default ACL for new files, direct ACL for the directory itself
    let output = Command::new("setfacl")
        .args([
            "-m", &format!("u:{username}:rwx"),
            "-m", &format!("d:u:{username}:rwx"),
        ])
        .arg(source_path)
        .output()
        .map_err(|e| {
            UswitchError::CommandFailed(
                format!("setfacl -m u:{username}:rwx,d:u:{username}:rwx {}", source_path.display()),
                e.to_string(),
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Operation not supported") || stderr.contains("Operation not permitted") {
            debug!("ACL not supported on this filesystem, skipping");
        } else {
            return Err(UswitchError::CommandFailed("setfacl".to_string(), stderr.trim().to_string()));
        }
    }

    // ── Create bind mount in user's home ──────────────────
    let projects_dir = PathBuf::from("/home").join(username).join("projects");
    fs::create_dir_all(&projects_dir).map_err(|e| {
        UswitchError::CommandFailed(
            format!("mkdir -p {}", projects_dir.display()),
            e.to_string(),
        )
    })?;

    let mount_point = projects_dir.join(project_name);
    if mount_point.exists() {
        if mount_point.is_dir() {
            fs::remove_dir(&mount_point).ok();
        } else if mount_point.is_symlink() {
            fs::remove_file(&mount_point).ok();
        }
    }

    fs::create_dir_all(&mount_point).map_err(|e| {
        UswitchError::CommandFailed(
            format!("mkdir -p {}", mount_point.display()),
            e.to_string(),
        )
    })?;

    let mount_status = Command::new("mount")
        .args(["--bind"])
        .arg(source_path)
        .arg(&mount_point)
        .status()
        .map_err(|e| {
            UswitchError::CommandFailed(
                format!("mount --bind {} {}", source_path.display(), mount_point.display()),
                e.to_string(),
            )
        })?;

    if !mount_status.success() {
        return Err(UswitchError::CommandFailed(
            format!("mount --bind {} {}", source_path.display(), mount_point.display()),
            "mount command failed".to_string(),
        ));
    }

    let chown_status = Command::new("chown")
        .args(["-R", &format!("{username}:{username}")])
        .arg(&mount_point)
        .status();

    if let Err(e) = chown_status {
        debug!("chown -R {}:{} {} failed: {}", username, username, mount_point.display(), e);
    }

    info!(user=%username, project=%project_name, path=%source_path.display(), "workspace attached");

    Ok(())
}

pub fn detach_workspace(username: &str, project_name: &str, source_path: Option<&Path>) -> Result<(), UswitchError> {
    // Remove ACL
    if let Some(path) = source_path.filter(|p| p.exists()) {
        let _ = Command::new("setfacl")
            .args([
                "-x", &format!("u:{username}"),
                "-x", &format!("d:u:{username}"),
            ])
            .arg(path)
            .output();
    }

    // Unmount bind mount
    let mount_point = PathBuf::from("/home").join(username).join("projects").join(project_name);
    let _ = Command::new("umount")
        .arg(&mount_point)
        .output();

    Ok(())
}
