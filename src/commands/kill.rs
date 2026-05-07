use crate::cli::KillArgs;
use crate::error::UswitchError;
use crate::output;
use crate::runtime;
use crate::state::{State, RuntimeStatus};
use crate::validate_username;
use std::process::Command;

pub fn execute(args: KillArgs) -> Result<(), UswitchError> {
    let state = State::load()?;

    let targets: Vec<String> = if args.all {
        state.users.iter()
            .filter(|(n, _)| runtime::runtime_is_active(n))
            .map(|(n, _)| n.clone())
            .collect()
    } else if let Some(user) = args.user {
        validate_username(&user)?;
        if !state.users.contains_key(&user) {
            return Err(UswitchError::UserNotFound(user));
        }
        if !runtime::runtime_is_active(&user) {
            return Err(UswitchError::NotRunning(user));
        }
        vec![user]
    } else {
        let active: Vec<String> = state.users.iter()
            .filter(|(n, _)| runtime::runtime_is_active(n))
            .map(|(n, _)| n.clone())
            .collect();
        match active.len() {
            0 => {
                output::print_info("No active runtime.");
                output::print_bullet("Kill a specific user: usw kill <user>");
                output::print_bullet("Kill all users: usw kill --all");
                return Ok(());
            }
            1 => active,
            _ => {
                output::print_info(&format!("Multiple active runtimes found ({}).", active.len()));
                output::print_bullet("Kill a specific user: usw kill <user>");
                output::print_bullet("Kill all users: usw kill --all");
                return Ok(());
            }
        }
    };

    if targets.is_empty() {
        output::print_info("No active processes.");
        return Ok(());
    }

    if !args.force {
        let msg = if args.all {
            format!("Kill {} active runtime(s)? [y/N]:", targets.len())
        } else {
            format!("Kill runtime '{}'? [y/N]:", targets[0])
        };
        if !output::confirm(&msg) {
            output::print_info("Aborted.");
            return Ok(());
        }
    }

    output::print_header("Cleaning up processes");
    output::print_arrow("Stopping services…");
    println!();

    let mut count = 0;
    for name in &targets {
        if validate_username(name).is_err() {
            output::print_warning(&format!("Skipping invalid username in state: {name}"));
            continue;
        }

        let _ = runtime::stop_runtime(name);
        let _ = runtime::disable_service_instance(name);

        let _ = Command::new("killall")
            .args(["-9", "-u", name])
            .output();

        count += 1;
        output::print_success(name);
    }

    let _ = State::with_lock(|state| {
        for name in &targets {
            if let Ok(u) = state.get_user_mut(name) {
                u.status = RuntimeStatus::Stopped;
            }
        }
        Ok(())
    });

    output::print_separator();
    output::print_success(&format!("Killed {count} process(es) — users & data preserved"));
    println!();

    Ok(())
}
