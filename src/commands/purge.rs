use crate::cli::PurgeArgs;
use crate::error::UswitchError;
use crate::output;
use crate::runtime;
use crate::state::State;
use crate::user;
use crate::validate_username;
use std::fs;
use tracing::debug;

pub fn execute(args: PurgeArgs) -> Result<(), UswitchError> {
    let state = State::load()?;

    let targets: Vec<String> = if args.all {
        state.users.keys().cloned().collect()
    } else if let Some(user) = args.user {
        validate_username(&user)?;
        if !state.users.contains_key(&user) {
            return Err(UswitchError::UserNotFound(user));
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
                output::print_bullet("Purge a specific user: usw purge <user>");
                output::print_bullet("Purge all users: usw purge --all");
                return Ok(());
            }
            1 => active,
            _ => {
                output::print_info(&format!("Multiple active runtimes found ({}).", active.len()));
                output::print_bullet("Purge a specific user: usw purge <user>");
                output::print_bullet("Purge all users: usw purge --all");
                return Ok(());
            }
        }
    };

    if targets.is_empty() {
        output::print_info("No runtimes to purge.");
        return Ok(());
    }

    if !args.force {
        let msg = if args.all {
            format!("Destroy ALL {} runtime(s) — users, homes, data? [y/N]:", targets.len())
        } else {
            format!("Destroy runtime '{}' permanently? [y/N]:", targets[0])
        };
        if !output::confirm_danger(&msg) {
            output::print_info("Aborted.");
            return Ok(());
        }
    }

    output::print_header("Purging runtime(s)");
    output::print_arrow("Stopping services and removing users…");
    println!();

    let mut count = 0;
    let mut failed = 0;
    let mut purged_names = Vec::new();

    for name in &targets {
        if validate_username(&name).is_err() {
            output::print_warning(&format!("Skipping invalid username in state: {name}"));
            continue;
        }

        if runtime::runtime_is_active(&name) {
            let _ = runtime::stop_runtime(&name);
        }
        let _ = runtime::disable_service_instance(&name);

        // Try to kill processes twice with a small delay
        for _ in 0..2 {
            let _ = std::process::Command::new("killall")
                .args(["-9", "-u", &name])
                .output();
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        match user::destroy_user(&name) {
            Ok(()) => {
                count += 1;
                purged_names.push(name.clone());
                output::print_success(&name);
            }
            Err(e) => {
                failed += 1;
                output::print_error(&format!("{name}: {e}"));
            }
        }
    }

    State::with_lock(|state| {
        for name in &purged_names {
            state.users.remove(name);
        }
        Ok(())
    })?;

    if args.all && state.users.is_empty() {
        if let Err(e) = fs::remove_file("/var/lib/usw/state.json") {
            debug!("failed to remove state.json: {}", e);
        }
    }

    output::print_separator();
    if failed == 0 {
        output::print_success(&format!("Purged {count} runtime(s)"));
    } else {
        output::print_warning(&format!("Purged {count} runtime(s), {failed} failed"));
    }
    println!();

    Ok(())
}
