pub mod cli;
pub mod commands;
pub mod error;
pub mod output;
pub mod plugin;
pub mod project;
pub mod runtime;
pub mod state;
pub mod switch;
pub mod user;

use std::fs;
use std::path::Path;
use nix::unistd::geteuid;
use crate::error::UswitchError;

pub const AI_CORE_ROOT: &str = "/opt/ai-core";
pub const STATE_DIR: &str = "/var/lib/usw";

pub fn ensure_root() -> Result<(), UswitchError> {
    let euid = geteuid().as_raw();
    if euid != 0 {
        return Err(UswitchError::NotRoot(euid));
    }
    Ok(())
}

pub fn ensure_directories() -> Result<(), UswitchError> {
    for dir in &[AI_CORE_ROOT, STATE_DIR] {
        let path = Path::new(dir);
        if !path.exists() {
            fs::create_dir_all(path).map_err(|e| {
                UswitchError::CommandFailed(
                    format!("mkdir -p {dir}"),
                    e.to_string(),
                )
            })?;
        }
    }
    Ok(())
}

pub fn validate_username(username: &str) -> Result<(), UswitchError> {
    if username.is_empty() || username.len() > 32 {
        return Err(UswitchError::InvalidUsername(username.to_string()));
    }

    let valid = username
        .bytes()
        .enumerate()
        .all(|(i, b)| {
            if i == 0 {
                b.is_ascii_lowercase()
            } else {
                b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_'
            }
        });

    if !valid {
        return Err(UswitchError::InvalidUsername(username.to_string()));
    }

    Ok(())
}
