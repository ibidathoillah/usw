use crate::cli::{EnvArgs, EnvAction};
use crate::error::UswitchError;
use crate::output;
use crate::state::State;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const ENV_HEADER: &str = r#"# ═══ AI Runtime Environment ═══
# Per-user configuration loaded by runtime scripts.
# Set API keys and other environment variables here.
# Each user has isolated configuration for instant account switching.
"#;

pub fn execute(args: EnvArgs) -> Result<(), UswitchError> {
    let user = &args.user;

    // Verify user exists in usw state
    let _ = State::load()?.get_user(user)?;

    let home = PathBuf::from("/home").join(user);
    let env_path = home.join("runtime/env");

    match args.action.unwrap_or(EnvAction::Show) {
        EnvAction::Show => show_env(&env_path),
        EnvAction::Set { pair } => set_env(&env_path, &pair),
        EnvAction::Unset { key } => unset_env(&env_path, &key),
        EnvAction::Edit => edit_env(&env_path),
    }
}

fn parse_env(path: &PathBuf) -> BTreeMap<String, String> {
    let mut vars = BTreeMap::new();
    let contents = fs::read_to_string(path).unwrap_or_default();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            vars.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    vars
}

fn write_env(path: &PathBuf, vars: &BTreeMap<String, String>) -> Result<(), UswitchError> {
    let mut content = ENV_HEADER.to_string();
    content.push('\n');
    for (k, v) in vars {
        content.push_str(&format!("{}={}\n", k, v));
    }
    fs::write(path, content).map_err(|e| {
        UswitchError::CommandFailed(format!("write {}", path.display()), e.to_string())
    })?;
    let perms = std::os::unix::fs::PermissionsExt::from_mode(0o600);
    fs::set_permissions(path, perms).map_err(|e| {
        UswitchError::CommandFailed(format!("chmod 600 {}", path.display()), e.to_string())
    })?;
    Ok(())
}

fn show_env(path: &PathBuf) -> Result<(), UswitchError> {
    let vars = parse_env(path);
    if vars.is_empty() {
        output::print_info(&format!("No environment variables set in {}", path.display()));
        output::print_bullet("Set one:  usw env <user> set KEY=value");
        return Ok(());
    }
    output::print_header(&format!("Environment: {}", path.display()));
    for (k, v) in &vars {
        output::print_kv(k, v);
    }
    println!();
    Ok(())
}

fn set_env(path: &PathBuf, pair: &str) -> Result<(), UswitchError> {
    let (key, value) = pair.split_once('=').ok_or_else(|| {
        UswitchError::CommandFailed(
            "env set".into(),
            format!("invalid format '{pair}', expected KEY=value"),
        )
    })?;

    let mut vars = parse_env(path);
    let existed = vars.contains_key(key.trim());
    vars.insert(key.trim().to_string(), value.trim().to_string());
    write_env(path, &vars)?;

    if existed {
        output::print_success(&format!("Updated {}={}", key.trim(), value.trim()));
    } else {
        output::print_success(&format!("Set {}={}", key.trim(), value.trim()));
    }
    Ok(())
}

fn unset_env(path: &PathBuf, key: &str) -> Result<(), UswitchError> {
    let mut vars = parse_env(path);
    if vars.remove(key).is_none() {
        output::print_warning(&format!("'{key}' not found in env file"));
        return Ok(());
    }
    write_env(path, &vars)?;
    output::print_success(&format!("Unset {key}"));
    Ok(())
}

fn edit_env(path: &PathBuf) -> Result<(), UswitchError> {
    // Ensure file exists
    if !path.exists() {
        write_env(path, &BTreeMap::new())?;
    }

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".into());
    let status = Command::new(&editor)
        .arg(path)
        .status()
        .map_err(|e| {
            UswitchError::CommandFailed(
                format!("{editor} {}", path.display()),
                e.to_string(),
            )
        })?;

    if !status.success() {
        return Err(UswitchError::CommandFailed(
            format!("{editor} {}", path.display()),
            "editor exited with error".to_string(),
        ));
    }

    output::print_success(&format!("Env file saved: {}", path.display()));
    Ok(())
}
