use crate::cli::DestroyArgs;
use crate::output;
use crate::project;
use crate::runtime;
use crate::state::State;
use crate::user;

pub fn execute(args: DestroyArgs) -> Result<(), crate::error::UswitchError> {
    output::print_header("Destroying runtime");

    if !args.force {
        if !output::confirm_danger(&format!("Destroy runtime for '{}' permanently? [y/N]:", args.user)) {
            output::print_info("Aborted.");
            return Ok(());
        }
    }

    let workspace = State::load().ok()
        .and_then(|s| s.users.get(&args.user).and_then(|u| u.workspace.clone()));

    output::print_arrow("Removing workspace attachments…");
    State::with_lock(|state| {
        let user_state = state.get_user(&args.user)?;

        for project_name in &user_state.projects {
            let _ = project::detach_workspace(&args.user, project_name, workspace.as_deref());
        }

        if runtime::runtime_is_active(&args.user) {
            output::print_arrow("Stopping runtime service…");
            let _ = runtime::stop_runtime(&args.user);
        }

        let _ = runtime::disable_service_instance(&args.user);
        user::destroy_user(&args.user)?;

        state.users.remove(&args.user);
        Ok(())
    })?;

    output::print_separator();
    output::print_success(&format!("Runtime '{}' destroyed", args.user));
    println!();

    Ok(())
}
