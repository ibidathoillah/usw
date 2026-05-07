use crate::cli::InstallArgs;
use crate::error::UswitchError;
use crate::output;
use crate::plugin::{self, Plugin};
use std::fs;
use std::path::Path;
use std::process::Command;

const BINARY_CACHE: &str = "/opt/ai-core/binaries";

pub fn execute(args: InstallArgs) -> Result<(), UswitchError> {
    if args.list {
        return list_plugins();
    }

    let tool = args.tool.as_deref().unwrap_or("");

    if tool.is_empty() {
        return install_all();
    }

    let p = plugin::find(tool)?;
    output::print_header(&format!("Installing '{}'", p.name));
    match install_plugin(&p)? {
        true => {
            output::print_separator();
            output::print_success(&format!("'{}' installed successfully", p.name));
        }
        false => {
            output::print_info(&format!("'{}' already up to date", p.name));
        }
    }

    Ok(())
}

fn list_plugins() -> Result<(), UswitchError> {
    let plugins = plugin::load_all()?;

    if plugins.is_empty() {
        output::print_empty(
            "No plugins found",
            &format!("The plugins directory is empty: {}", plugin::PLUGINS_DIR),
            "Hint: Add .toml plugin manifests to the plugins directory",
        );
        return Ok(());
    }

    output::print_header("Available plugins");

    let max_name_w = plugins.iter().map(|p| p.name.len()).max().unwrap_or(12).max(12);
    let max_method_w = plugins.iter().map(|p| p.install.method.len()).max().unwrap_or(8).max(8);

    // Header
    println!(
        "  {}{:<name_w$}  {:<method_w$}  {:<20}  {}{}",
        output::BOLD,
        "NAME",
        "METHOD",
        "DESCRIPTION",
        "STATUS",
        output::RESET,
        name_w = max_name_w,
        method_w = max_method_w,
    );
    println!(
        "  {}{}{}",
        output::DIM,
        "─".repeat(max_name_w + max_method_w + 40),
        output::RESET
    );

    for p in &plugins {
        let installed = if plugin::is_deployed(p) {
            output::green("installed")
        } else {
            output::dim("not installed")
        };
        println!(
            "  {:<name_w$}  {:<method_w$}  {:<20}  {}",
            p.name,
            if p.install.method.is_empty() { "binary" } else { &p.install.method },
            p.description,
            installed,
            name_w = max_name_w,
            method_w = max_method_w,
        );
    }

    println!();
    output::print_bullet("Install one:    usw install <name>");
    output::print_bullet("Install all:    usw install");
    println!();

    Ok(())
}

fn install_all() -> Result<(), UswitchError> {
    let plugins = plugin::load_all()?;

    if plugins.is_empty() {
        output::print_info("No plugins available. Use --list to check.");
        return Ok(());
    }

    output::print_header("Installing all plugins");

    let mut installed = 0;
    let mut skipped = 0;
    let mut failed = 0;

    for p in &plugins {
        match install_plugin(p) {
            Ok(true) => installed += 1,
            Ok(false) => skipped += 1,
            Err(e) => {
                failed += 1;
                output::print_error(&format!("{}: {}", p.name, e));
            }
        }
    }

    output::print_separator();
    if installed > 0 {
        output::print_success(&format!("Installed {installed} plugin(s)"));
    }
    if skipped > 0 {
        output::print_info(&format!("{skipped} already up to date"));
    }
    if failed > 0 {
        output::print_warning(&format!("{failed} failed"));
    }
    println!();

    Ok(())
}

fn install_plugin(p: &Plugin) -> Result<bool, UswitchError> {
    if plugin::is_deployed(p) {
        return Ok(false);
    }

    output::print_arrow(&format!("Installing '{}'…", p.name));

    match p.install.method.as_str() {
        "binary" => install_binary(p),
        "npm" => install_npm(p),
        "pip" => install_pip(p),
        "shell" => install_shell(p),
        _ if !p.install.binary_search.is_empty() => install_binary(p),
        _ => {
            output::print_info(&format!("{}: no install method configured, trying binary search…", p.name));
            install_binary(p)
        }
    }
}

fn install_binary(p: &Plugin) -> Result<bool, UswitchError> {
    let mut found: Option<String> = None;

    // Also check if already at cache path
    let cache = plugin::cache_path(p);
    if cache.exists() && !cache.is_symlink() {
        return Ok(false);
    }

    for search in &p.install.binary_search {
        if Path::new(search).exists() {
            found = Some(search.clone());
            break;
        }
    }

    let src = match found {
        Some(s) => s,
        None => {
            output::print_warning(&format!("{} not found", p.name));
            output::print_bullet(&format!("Try: manual install, then add path to {}", plugin::PLUGINS_DIR));
            return Ok(false);
        }
    };

    fs::create_dir_all(BINARY_CACHE).map_err(|e| {
        UswitchError::CommandFailed(format!("mkdir -p {BINARY_CACHE}"), e.to_string())
    })?;

    fs::copy(&src, &cache).map_err(|e| {
        UswitchError::CommandFailed(format!("cp {src} {}", cache.display()), e.to_string())
    })?;

    let perms = std::os::unix::fs::PermissionsExt::from_mode(0o755);
    fs::set_permissions(&cache, perms).map_err(|e| {
        UswitchError::CommandFailed(format!("chmod 755 {}", cache.display()), e.to_string())
    })?;

    output::print_success(&format!("{} → {}", p.name, cache.display()));
    Ok(true)
}

fn install_npm(p: &Plugin) -> Result<bool, UswitchError> {
    let cache = plugin::cache_path(p);
    if cache.exists() {
        return Ok(false);
    }

    let pkg = &p.install.npm_package;
    if pkg.is_empty() {
        output::print_warning(&format!("{}: no npm_package configured", p.name));
        return Ok(false);
    }

    output::print_step(1, &format!("npm install -g {}", pkg));

    // Install globally then copy to cache
    let output = Command::new("npm")
        .args(["install", "-g", pkg])
        .output()
        .map_err(|e| {
            UswitchError::CommandFailed(format!("npm install -g {pkg}"), e.to_string())
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // npm warnings are not fatal, try to find the binary
        if stderr.contains("ERR!") {
            return Err(UswitchError::CommandFailed(
                format!("npm install -g {pkg}"),
                stderr.trim().to_string(),
            ));
        }
    }

    // Find the binary and cache it
    for search in &p.install.binary_search {
        if Path::new(search).exists() {
            return install_binary(p);
        }
    }

    output::print_warning("installed but couldn't locate binary. Add path to plugin manifest.");
    Ok(false)
}

fn install_pip(p: &Plugin) -> Result<bool, UswitchError> {
    let cache = plugin::cache_path(p);
    if cache.exists() {
        return Ok(false);
    }

    let pkg = &p.install.pip_package;
    if pkg.is_empty() {
        output::print_warning(&format!("{}: no pip_package configured", p.name));
        return Ok(false);
    }

    output::print_step(1, &format!("pip install {}", pkg));

    let output = Command::new("pip")
        .args(["install", pkg])
        .output()
        .map_err(|e| {
            UswitchError::CommandFailed(format!("pip install {pkg}"), e.to_string())
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(UswitchError::CommandFailed(
            format!("pip install {pkg}"),
            stderr.trim().to_string(),
        ));
    }

    for search in &p.install.binary_search {
        if Path::new(search).exists() {
            return install_binary(p);
        }
    }

    output::print_warning("installed but couldn't locate binary. Add path to plugin manifest.");
    Ok(false)
}

fn install_shell(p: &Plugin) -> Result<bool, UswitchError> {
    let cache = plugin::cache_path(p);
    if cache.exists() {
        return Ok(false);
    }

    let cmd = &p.install.shell_command;
    if cmd.is_empty() {
        output::print_warning(&format!("{}: no shell_command configured", p.name));
        return Ok(false);
    }

    output::print_step(1, &format!("shell: {}", cmd));

    let output = Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| {
            UswitchError::CommandFailed(format!("bash -c '{cmd}'"), e.to_string())
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(UswitchError::CommandFailed(
            format!("bash -c '{cmd}'"),
            stderr.trim().to_string(),
        ));
    }

    for search in &p.install.binary_search {
        if Path::new(search).exists() {
            return install_binary(p);
        }
    }

    Ok(true)
}
