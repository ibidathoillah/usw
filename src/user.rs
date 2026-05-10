use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, debug};
use crate::error::UswitchError;
use crate::plugin;

const BINARY_CACHE_DIR: &str = "/opt/ai-core/binaries";

const USER_RUNTIME_DIRS: &[&str] = &[
    ".config/ai",
    "projects",
    ".local/share/ai",
    "runtime",
];

pub fn create_env_file(home: &Path) -> Result<(), UswitchError> {
    let env_file = home.join("runtime/env");
    if !env_file.exists() {
        let content = r##"# ═══ AI Runtime Environment ═══
# Per-user configuration loaded by runtime scripts.
# Set API keys and other environment variables here.
# Each user has isolated configuration for instant account switching.

# Example:
# API_KEY=your_key_here

# Add your env vars below...
"##;
        fs::write(&env_file, content).map_err(|e| {
            UswitchError::CommandFailed(format!("write {}", env_file.display()), e.to_string())
        })?;
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&env_file, PermissionsExt::from_mode(0o600)).map_err(|e| {
            UswitchError::CommandFailed(format!("chmod 600 {}", env_file.display()), e.to_string())
        })?;
        debug!("env file created at {}", env_file.display());
    }
    Ok(())
}

pub fn deploy_binary(username: &str, src: &Path, target_name: &str) -> Result<(), UswitchError> {
    let cache_dir = Path::new(BINARY_CACHE_DIR);
    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir).map_err(|e| {
            UswitchError::CommandFailed(format!("mkdir -p {BINARY_CACHE_DIR}"), e.to_string())
        })?;
    }

    let dst = cache_dir.join(target_name);

    if !dst.exists() {
        fs::copy(src, &dst).map_err(|e| {
            UswitchError::CommandFailed(
                format!("cp {} {}", src.display(), dst.display()),
                e.to_string(),
            )
        })?;

        let perms = std::os::unix::fs::PermissionsExt::from_mode(0o755);
        fs::set_permissions(&dst, perms).map_err(|e| {
            UswitchError::CommandFailed(
                format!("chmod 755 {}", dst.display()),
                e.to_string(),
            )
        })?;
    }

    info!(user=%username, binary=target_name, "binary deployed");

    Ok(())
}

pub fn create_user(username: &str) -> Result<PathBuf, UswitchError> {
    let home = PathBuf::from("/home").join(username);

    let output = Command::new("useradd")
        .args(["-m", "-d"])
        .arg(&home)
        .args(["-s", "/bin/bash", username])
        .output()
        .map_err(|e| {
            UswitchError::CommandFailed(
                format!("useradd {username}"),
                e.to_string(),
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("already exists") {
            return Err(UswitchError::UserExists(username.to_string()));
        }
        return Err(UswitchError::CommandFailed(
            format!("useradd {username}"),
            stderr.trim().to_string(),
        ));
    }

    // Verify user actually exists in /etc/passwd
    if !user_exists_on_system(username) {
        return Err(UswitchError::CommandFailed(
            format!("useradd {username}"),
            "useradd returned success but user not found in /etc/passwd".into(),
        ));
    }

    info!(user = %username, "Linux user created");

    setup_runtime_dirs(&home, username)?;

    Ok(home)
}

fn setup_runtime_dirs(home: &Path, username: &str) -> Result<(), UswitchError> {
    for dir in USER_RUNTIME_DIRS {
        let path = home.join(dir);
        fs::create_dir_all(&path).map_err(|e| {
            UswitchError::CommandFailed(
                format!("mkdir -p {}", path.display()),
                e.to_string(),
            )
        })?;
    }

    // Ensure AI binaries are on PATH for login shells
    let profile = home.join(".profile");
    let path_line = format!("\n# usw: AI runtime binaries\nexport PATH=\"{BINARY_CACHE_DIR}:$PATH\"\n");
    let existing = fs::read_to_string(&profile).unwrap_or_default();
    if !existing.contains(BINARY_CACHE_DIR) {
        use std::io::Write;
        let mut f = fs::OpenOptions::new().append(true).create(true).open(&profile).map_err(|e| {
            UswitchError::CommandFailed(format!("open {} for append", profile.display()), e.to_string())
        })?;
        writeln!(f, "{}", path_line.trim()).map_err(|e| {
            UswitchError::CommandFailed(format!("write {}", profile.display()), e.to_string())
        })?;
    }

    generate_start_script(home, username)?;

    create_env_file(home)?;

    generate_telegram_config(home)?;

    Command::new("chown")
        .args(["-R", &format!("{username}:{username}")])
        .arg(home)
        .status()
        .map_err(|e| {
            UswitchError::CommandFailed(
                format!("chown -R {username}:{username} {}", home.display()),
                e.to_string(),
            )
        })?;

    debug!(home = %home.display(), "Runtime directories set up");

    Ok(())
}

/// Escape a string for safe use as a single POSIX shell word.
fn shell_escape(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }
    if s.chars().all(|c| c.is_ascii_alphanumeric() || "_-./=@%+,".contains(c)) {
        return s.to_string();
    }
    format!("'{}'", s.replace('\'', "'\\''"))
}

pub fn generate_start_script(home: &Path, username: &str) -> Result<(), UswitchError> {
    let start_script = home.join("runtime/start.sh");
    let plugins = plugin::load_all()?;

    let mut plugin_blocks = String::new();

    for p in &plugins {
        let bin_name = plugin::binary_name(p);
        if bin_name.is_empty() {
            continue;
        }

        let bin_path = format!("{}/{}", BINARY_CACHE_DIR, shell_escape(&bin_name));
        let args = p.runtime.args.iter().map(|a| shell_escape(a)).collect::<Vec<_>>().join(" ");
        let short = p.name.replace('-', "_").to_uppercase();
        let work_dir = if p.runtime.work_dir.is_empty() {
            "\"$HOME\"".to_string()
        } else {
            shell_escape(&p.runtime.work_dir)
        };
        let has_bot = p.telegram.as_ref().map_or(false, |t| !t.package.is_empty());

        // Open: if binary exists
        plugin_blocks.push_str(&format!(r#"
# -- {name}: {desc} --
if [ -x {bin_path} ]; then
    echo "▪ {name}"
    mkdir -p {work_dir} 2>/dev/null
    {bin_path} {args} &
    PID_{short}=$!
"#,
            name = shell_escape(&p.name),
            desc = shell_escape(&p.description),
            bin_path = bin_path,
            args = args,
            short = short,
            work_dir = work_dir,
        ));

        // Bot block (inside the binary check)
        if has_bot {
            let bot = p.telegram.as_ref().unwrap();
            let wrapper_path = format!("{}/{}", BINARY_CACHE_DIR, shell_escape(&bot.wrapper_name));
            plugin_blocks.push_str(&format!(r#"
    if [ -x {wrapper_path} ] && [ -f "$HOME/.config/ai-bot/.env" ]; then
        token=$(grep '^TELEGRAM_BOT_TOKEN=' "$HOME/.config/ai-bot/.env" 2>/dev/null | cut -d= -f2 || true)
        uid=$(grep '^TELEGRAM_ALLOWED_USER_ID=' "$HOME/.config/ai-bot/.env" 2>/dev/null | cut -d= -f2 || true)
        if [ -n "${{token}}" ] && [ -n "${{uid}}" ]; then
            echo "▪ {name} bot"
            {wrapper_path} start &
            PID_{short}_BOT=$!
        fi
    fi
"#,
                wrapper_path = wrapper_path,
                name = shell_escape(&p.name),
                short = short,
            ));
        }

        // Close: fi for binary check
        plugin_blocks.push_str("fi\nsleep 2\n");
    }

    // Build PID list
    let mut pid_parts: Vec<String> = Vec::new();
    for p in &plugins {
        if plugin::binary_name(p).is_empty() { continue; }
        let short = p.name.replace('-', "_").to_uppercase();
        pid_parts.push(format!("${{PID_{short}:-none}}"));
    }

    let script = format!(r#"#!/bin/bash
set -euo pipefail

USERNAME={username}
export HOME="/home/${{USERNAME}}"
export PATH="{binary_cache}:${{HOME}}/.local/bin:/usr/local/bin:/usr/bin:/bin"
export AI_RUNTIME_USER="${{USERNAME}}"

echo "=== AI Runtime: ${{USERNAME}} ==="
echo "Started at $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "Projects: $(ls \"${{HOME}}/projects\" 2>/dev/null || echo 'none')"
echo "Plugins: $(ls {binary_cache}/* 2>/dev/null | xargs -I{{}} basename '{{}}' | tr '\n' ' ' || echo 'none')"

# Source per-user environment (API keys, etc.)
if [ -f \"${{HOME}}/runtime/env\" ]; then
    set -a; source \"${{HOME}}/runtime/env\"; set +a
fi

echo ""
{plugin_blocks}
echo ""
echo "Runtime active. PIDs: {pid_list}"
wait -n 2>/dev/null || sleep infinity
"#,
        binary_cache = BINARY_CACHE_DIR,
        username = shell_escape(username),
        plugin_blocks = plugin_blocks,
        pid_list = pid_parts.join(" "),
    );

    fs::write(&start_script, script).map_err(|e| {
        UswitchError::CommandFailed(format!("write {}", start_script.display()), e.to_string())
    })?;

    let perms = std::os::unix::fs::PermissionsExt::from_mode(0o755);
    fs::set_permissions(&start_script, perms).map_err(|e| {
        UswitchError::CommandFailed(format!("chmod 755 {}", start_script.display()), e.to_string())
    })?;

    Ok(())
}

fn generate_telegram_config(home: &Path) -> Result<(), UswitchError> {
    let config_dir = home.join(".config/ai-bot");
    fs::create_dir_all(&config_dir).map_err(|e| {
        UswitchError::CommandFailed(format!("mkdir -p {}", config_dir.display()), e.to_string())
    })?;

    let env_file = config_dir.join(".env");
    if !env_file.exists() {
        let content = r##"# ═══ AI Bot Configuration ═══
# Generic Telegram bot configuration.
# Get token from @BotFather:  /newbot
TELEGRAM_BOT_TOKEN=
# Get your ID from @userinfobot
TELEGRAM_ALLOWED_USER_ID=

# Plugin-specific settings (configured by plugin):
# API_URL=http://localhost:8080
# MODEL_PROVIDER=default
# MODEL_ID=default

BOT_LOCALE=en
LOG_LEVEL=info
"##;
        fs::write(&env_file, &content).map_err(|e| {
            UswitchError::CommandFailed(format!("write {}", env_file.display()), e.to_string())
        })?;
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&env_file, PermissionsExt::from_mode(0o600)).map_err(|e| {
            UswitchError::CommandFailed(format!("chmod 600 {}", env_file.display()), e.to_string())
        })?;
        debug!("bot config created at {}", env_file.display());
    }

    Ok(())
}

pub fn destroy_user(username: &str) -> Result<(), UswitchError> {
    if !user_exists_on_system(username) {
        debug!(user = %username, "User already removed from system");
        return Ok(());
    }

    let output = Command::new("userdel")
        .args(["-r", username])
        .output()
        .map_err(|e| {
            UswitchError::CommandFailed(
                format!("userdel -r {username}"),
                e.to_string(),
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(UswitchError::CommandFailed(
            format!("userdel -r {username}"),
            stderr.trim().to_string(),
        ));
    }

    info!(user = %username, "Linux user and home directory removed");

    Ok(())
}

pub fn user_exists_on_system(username: &str) -> bool {
    nix::unistd::User::from_name(username)
        .ok()
        .flatten()
        .is_some()
}

pub fn add_to_sudoers(username: &str) -> Result<(), UswitchError> {
    // 1. Add to group
    let output = Command::new("usermod")
        .args(["-aG", "sudo", username])
        .output()
        .map_err(|e| {
            UswitchError::CommandFailed(
                format!("usermod -aG sudo {username}"),
                e.to_string(),
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(UswitchError::CommandFailed(
            format!("usermod -aG sudo {username}"),
            stderr.trim().to_string(),
        ));
    }

    info!(user = %username, "Added to sudoers (group: sudo, passwordless enabled)");
    Ok(())
}

pub fn switch_to_user(username: &str) -> Result<(), UswitchError> {
    switch_to_user_in_dir(username, None)
}

pub fn switch_to_user_in_dir(username: &str, work_dir: Option<&str>) -> Result<(), UswitchError> {
    info!(user = %username, "Switching to user shell");

    let is_tty = nix::unistd::isatty(0).unwrap_or(false);

    let mut cmd = if is_tty {
        Command::new("su")
    } else {
        let mut c = Command::new("setsid");
        c.arg("su");
        c
    };

    cmd.arg("-l").arg(username);

    if let Some(dir) = work_dir {
        let escaped = shell_escape(dir);
        // We use -i to force an interactive shell which handles job control and TTY better
        cmd.arg("-c").arg(format!("cd {escaped} 2>/dev/null; exec $SHELL -li"));
    }

    let status = cmd.status().map_err(|e| {
        UswitchError::CommandFailed(format!("su - {username}"), e.to_string())
    })?;

    if !status.success() {
        return Err(UswitchError::CommandFailed(
            format!("su - {username}"),
            "shell exited with error".to_string(),
        ));
    }

    Ok(())
}
