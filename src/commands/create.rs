use crate::cli::CreateArgs;
use crate::error::UswitchError;
use crate::output;
use crate::plugin;
use crate::project;
use crate::runtime;
use crate::state::{State, RuntimeStatus, UserState};
use crate::user;
use crate::validate_username;
use chrono::Utc;

pub fn execute(args: CreateArgs) -> Result<(), UswitchError> {
    let user = &args.name;
    validate_username(user)?;

    let cwd = std::env::current_dir().map_err(|e| {
        UswitchError::CommandFailed("pwd".into(), e.to_string())
    })?;

    output::print_header("Creating runtime");

    // Step 1: Create user
    output::print_step(1, "Creating Linux user…");
    let home = State::with_lock(|state| {
        state.ensure_user_not_exists(user)?;
        let home = user::create_user(user)?;
        state.users.insert(user.to_string(), UserState {
            created_at: Utc::now(),
            home: home.clone(),
            projects: Vec::new(),
            status: RuntimeStatus::Stopped,
            workspace: Some(cwd.clone()),
        });
        Ok(home)
    })?;
    output::print_success(&format!("User '{}' created", user));

    // Step 2: Deploy binaries
    output::print_step(2, "Deploying AI binaries…");
    let binaries = plugin::discover_binaries().unwrap_or_else(|e| {
        output::print_warning(&format!("Could not load plugins: {e}"));
        Vec::new()
    });
    let mut deployed = 0;
    for (src, name) in &binaries {
        if user::deploy_binary(user, src, name).is_ok() {
            deployed += 1;
        }
    }
    if deployed > 0 {
        output::print_success(&format!("Deployed {} binary/ies", deployed));
    } else {
        output::print_info("No binaries to deploy");
    }

    // Regenerate start script now that binaries are in the cache
    user::generate_start_script(&home, user)?;

    // Step 3: Attach workspace
    output::print_step(3, "Attaching workspace…");
    let project_name = cwd.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "workspace".to_string());
    project::attach_workspace(user, &project_name, &cwd)?;
    output::print_success(&format!("Workspace '{}' attached", project_name));

    State::with_lock(|state| {
        if let Ok(u) = state.get_user_mut(user) {
            if !u.projects.contains(&project_name) {
                u.projects.push(project_name.clone());
            }
            u.workspace = Some(cwd.clone());
        }
        Ok(())
    })?;

    // Step 4: Start runtime
    if !args.no_start {
        output::print_step(4, "Starting runtime service…");
        runtime::start_runtime(user)?;
        State::with_lock(|state| {
            if let Ok(u) = state.get_user_mut(user) { u.status = RuntimeStatus::Running; }
            Ok(())
        })?;
        output::print_success("Runtime started");
    }

    output::print_separator();
    output::print_success(&format!("Runtime '{}' ready → {}", user, project_name));
    output::print_bullet(&format!("Home:    {}", home.display()));
    output::print_bullet(&format!("Project: {}", cwd.display()));
    if !args.no_start {
        output::print_bullet("Switch to runtime: usw <user>");
    }
    println!();

    Ok(())
}
