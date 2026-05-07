use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, debug};
use crate::error::UswitchError;

/// Attach a workspace directory to a user.
/// The source path (e.g. cwd) gets ACL for the user, and a symlink
/// is created in /home/<user>/projects/<name> → source_path.
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
            // Filesystem doesn't support ACL — fall through gracefully
            debug!("ACL not supported on this filesystem, skipping");
        } else {
            return Err(UswitchError::CommandFailed("setfacl".to_string(), stderr.trim().to_string()));
        }
    }

    // ── Create symlink in user's home ──────────────────
    let projects_dir = PathBuf::from("/home").join(username).join("projects");
    fs::create_dir_all(&projects_dir).map_err(|e| {
        UswitchError::CommandFailed(
            format!("mkdir -p {}", projects_dir.display()),
            e.to_string(),
        )
    })?;

    let symlink = projects_dir.join(project_name);
    if symlink.exists() {
        if symlink.is_symlink() {
            fs::remove_file(&symlink).ok();
        }
    }

    unix_fs::symlink(source_path, &symlink).map_err(|e| {
        UswitchError::CommandFailed(
            format!("ln -s {} {}", source_path.display(), symlink.display()),
            e.to_string(),
        )
    })?;

    let chown_status = Command::new("chown")
        .args(["-h", &format!("{username}:{username}")])
        .arg(&symlink)
        .status();

    if let Err(e) = chown_status {
        debug!("chown -h {}:{} {} failed: {}", username, username, symlink.display(), e);
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

    // Remove symlink
    let symlink = PathBuf::from("/home").join(username).join("projects").join(project_name);
    if symlink.is_symlink() {
        fs::remove_file(&symlink).ok();
    }

    Ok(())
}
