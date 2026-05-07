use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::error::UswitchError;

pub const PLUGINS_DIR: &str = "/opt/ai-core/plugins";
pub const BINARY_CACHE: &str = "/opt/ai-core/binaries";

/// Loaded plugin manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub homepage: String,

    #[serde(default)]
    pub install: InstallConfig,

    #[serde(default)]
    pub runtime: RuntimeConfig,

    #[serde(default)]
    pub telegram: Option<TelegramConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstallConfig {
    /// How to install: binary, npm, pip, shell
    #[serde(default)]
    pub method: String,
    /// Paths to search for existing binaries
    #[serde(default)]
    pub binary_search: Vec<String>,
    /// npm package name
    #[serde(default)]
    pub npm_package: String,
    /// pip package name
    #[serde(default)]
    pub pip_package: String,
    /// shell install script
    #[serde(default)]
    pub shell_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeConfig {
    /// Binary or command to execute
    #[serde(default)]
    pub command: String,
    /// Arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Working directory
    #[serde(default)]
    pub work_dir: String,
    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Default port
    #[serde(default)]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TelegramConfig {
    /// npm package for telegram bot
    #[serde(default)]
    pub package: String,
    /// Wrapper command name
    #[serde(default)]
    pub wrapper_name: String,
    /// OpenCode API URL
    #[serde(default)]
    pub api_url: String,
    /// Model provider
    #[serde(default)]
    pub provider: String,
    /// Model ID
    #[serde(default)]
    pub model_id: String,
}

// ── Registry ──────────────────────────────────────────────

pub fn load_all() -> Result<Vec<Plugin>, UswitchError> {
    let dir = Path::new(PLUGINS_DIR);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut plugins = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| {
        UswitchError::State(format!("read plugins dir: {e}"))
    })? {
        let entry = entry.ok();
        if let Some(e) = entry {
            let path = e.path();
            if path.extension().map_or(false, |x| x == "toml") {
                if let Ok(p) = load(&path) {
                    plugins.push(p);
                }
            }
        }
    }

    plugins.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(plugins)
}

pub fn load(path: &Path) -> Result<Plugin, UswitchError> {
    let content = fs::read_to_string(path).map_err(|e| {
        UswitchError::State(format!("read plugin {}: {e}", path.display()))
    })?;

    toml::from_str(&content).map_err(|e| {
        UswitchError::State(format!("parse plugin {}: {e}", path.display()))
    })
}

pub fn find(name: &str) -> Result<Plugin, UswitchError> {
    let available = match available_names() {
        Ok(names) => names.join(", "),
        Err(_) => "none".to_string(),
    };
    load_all()?
        .into_iter()
        .find(|p| p.name == name)
        .ok_or_else(|| UswitchError::CommandFailed(
            "plugin".into(),
            format!("plugin '{name}' not found. Available: {available}"),
        ))
}

pub fn available_names() -> Result<Vec<String>, UswitchError> {
    Ok(load_all()?.into_iter().map(|p| p.name).collect())
}

/// Return the canonical binary name for a plugin (runtime.command if set, else name).
pub fn binary_name(plugin: &Plugin) -> String {
    if !plugin.runtime.command.is_empty() {
        plugin.runtime.command.clone()
    } else {
        plugin.name.clone()
    }
}

/// Discover all installable binaries from all plugin manifests.
/// Returns tuples of (source_path, target_name) where target_name is the
/// canonical name to use in the binary cache.
pub fn discover_binaries() -> Result<Vec<(PathBuf, String)>, UswitchError> {
    let plugins = load_all()?;
    let mut found = Vec::new();
    for plugin in &plugins {
        let name = binary_name(plugin);
        for search_path in &plugin.install.binary_search {
            let p = Path::new(search_path);
            if p.exists() && p.is_file() {
                if !found.iter().any(|(_, n): &(PathBuf, String)| n == &name) {
                    found.push((p.to_path_buf(), name.clone()));
                }
            }
        }
    }
    Ok(found)
}

/// Check if a plugin's binary is deployed to the cache
pub fn is_deployed(plugin: &Plugin) -> bool {
    let target = cache_path(plugin);
    target.exists() && !target.is_symlink()
}

/// Get the deployed binary path for a plugin
pub fn cache_path(plugin: &Plugin) -> PathBuf {
    Path::new(BINARY_CACHE).join(binary_name(plugin))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plugin() {
        let toml = r#"
name = "opencode"
description = "Test"

[install]
method = "binary"
binary_search = ["/usr/bin/opencode"]

[runtime]
command = "opencode"
args = ["serve"]
port = 4096
"#;
        let plugin: Plugin = toml::from_str(toml).unwrap();
        assert_eq!(plugin.name, "opencode");
        assert_eq!(plugin.install.binary_search, vec!["/usr/bin/opencode"]);
        assert_eq!(plugin.runtime.command, "opencode");
        assert_eq!(plugin.runtime.port, 4096);
    }
}
