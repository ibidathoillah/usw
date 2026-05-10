use crate::plugin;
use crate::project;
use crate::runtime;
use crate::state::{State, RuntimeStatus, UserState};
use crate::user;
use crate::validate_username;
use crate::output;
use anyhow::{Context, Result};
use chrono::Utc;

/// usw use <user> — create if new, switch if exists
pub fn run(username: &str) -> Result<()> {
    validate_username(username)
        .with_context(|| format!("invalid username: {username}"))?;

    let workspace = std::env::current_dir()
        .with_context(|| "cannot get current directory")?;

    let exists = State::load()
        .map(|s| s.users.contains_key(username))
        .unwrap_or(false);

    if exists {
        // ── Reuse existing: stop others, restart ───────
        output::print_header(&format!("Switching to '{}'", username));
        output::print_arrow("Stopping other runtimes…");

        let state = State::load()?;
        for (name, _) in &state.users {
            if *name != username && runtime::runtime_is_active(name) {
                let _ = runtime::stop_runtime(name);
                let _ = State::with_lock(|s| {
                    if let Ok(u) = s.get_user_mut(name) { u.status = RuntimeStatus::Stopped; }
                    Ok(())
                });
            }
        }

        if !runtime::runtime_is_active(username) {
            output::print_arrow("Starting runtime…");
            runtime::start_runtime(username)?;
        }

        State::with_lock(|s| {
            if let Ok(u) = s.get_user_mut(username) { u.status = RuntimeStatus::Running; }
            Ok(())
        })?;

        output::print_separator();
        output::print_success(&format!("Switched to '{}'", username));
        output::print_bullet(&format!("Workspace: {}", workspace.display()));
        println!();
    } else {
        // ── Create new ─────────────────────────────────
        output::print_header(&format!("Creating runtime '{}'", username));

        let home = State::with_lock(|state| {
            state.ensure_user_not_exists(username)?;
            let home = user::create_user(username)?;
            state.users.insert(username.to_string(), UserState {
                created_at: Utc::now(),
                home: home.clone(),
                projects: Vec::new(),
                status: RuntimeStatus::Stopped,
                workspace: Some(workspace.clone()),
            });
            Ok(home)
        })?;

        // Deploy binaries
        output::print_arrow("Deploying binaries…");
        let binaries = plugin::discover_binaries().unwrap_or_else(|e| {
            output::print_warning(&format!("Could not load plugins: {e}"));
            Vec::new()
        });
        let mut deployed = 0;
        for (src, name) in &binaries {
            if user::deploy_binary(username, src, name).is_ok() {
                deployed += 1;
            }
        }
        if deployed > 0 {
            output::print_success(&format!("Deployed {} binary/ies", deployed));
        }

        // Regenerate start script now that binaries are known
        user::generate_start_script(&home, username)?;

        let invoked_by_privileged = std::env::var("SUDO_USER").is_ok() || 
                                   std::env::var("USER").map(|u| u == "root").unwrap_or(false);

        if invoked_by_privileged {
            output::print_arrow("Adding to sudoers…");
            user::add_to_sudoers(username)?;
            output::print_success(&format!("User '{}' added to sudoers", username));
        }

        // Attach cwd as workspace
        output::print_arrow("Attaching workspace…");
        let proj_name = workspace.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "workspace".to_string());
        project::attach_workspace(username, &proj_name, &workspace)?;

        State::with_lock(|state| {
            if let Ok(u) = state.get_user_mut(username) {
                if !u.projects.contains(&proj_name) {
                    u.projects.push(proj_name.clone());
                }
            }
            Ok(())
        })?;

        output::print_arrow("Starting runtime…");
        runtime::start_runtime(username)?;

        State::with_lock(|s| {
            if let Ok(u) = s.get_user_mut(username) { u.status = RuntimeStatus::Running; }
            Ok(())
        })?;

        output::print_separator();
        output::print_success(&format!("Runtime '{}' ready", username));
        output::print_bullet(&format!("Project: {}", proj_name));
        println!();
    }

    // Drop into shell
    user::switch_to_user_in_dir(username, Some(&workspace.to_string_lossy()))
        .map_err(|e| anyhow::anyhow!(e))
}

/// usw current — show active
pub fn current() -> Result<()> {
    let state = State::load()?;

    for (name, _us) in &state.users {
        if runtime::runtime_is_active(name) {
            let proj = state.users.get(name)
                .and_then(|u| u.workspace.as_ref())
                .map(|w| w.to_string_lossy().to_string())
                .unwrap_or_else(|| "—".to_string());
            output::print_success(&format!("{name} → {proj}"));
            return Ok(());
        }
    }

    output::print_info("No active runtime");
    output::print_bullet("Switch to a runtime: usw use <user>");
    Ok(())
}
