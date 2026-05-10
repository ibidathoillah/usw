use crate::cli::SudoArgs;
use crate::error::UswitchError;
use crate::output;
use crate::user;
use crate::state::State;
use crate::runtime;

pub fn execute(args: SudoArgs) -> Result<(), UswitchError> {
    let state = State::load()?;
    
    let targets: Vec<String> = if args.all {
        state.users.keys().cloned().collect()
    } else if let Some(u) = args.user {
        vec![u]
    } else {
        let active: Vec<String> = state.users.iter()
            .filter(|(n, _)| runtime::runtime_is_active(n))
            .map(|(n, _)| n.clone())
            .collect();
            
        match active.len() {
            0 => {
                output::print_info("No active runtime.");
                output::print_bullet("Grant sudo to a specific user: usw sudo <user>");
                output::print_bullet("Grant sudo to all users: usw sudo --all");
                return Ok(());
            }
            1 => active,
            _ => {
                output::print_info(&format!("Multiple active runtimes found ({}).", active.len()));
                output::print_bullet("Grant sudo to a specific user: usw sudo <user>");
                output::print_bullet("Grant sudo to all users: usw sudo --all");
                return Ok(());
            }
        }
    };

    if targets.is_empty() {
        output::print_info("No users found.");
        return Ok(());
    }

    if !args.force {
        let msg = if args.all {
            format!("Grant sudo privileges to ALL {} runtimes? [y/N]:", targets.len())
        } else {
            format!("Grant sudo privileges to runtime '{}'? [y/N]:", targets[0])
        };
        if !output::confirm(&msg) {
            output::print_info("Aborted.");
            return Ok(());
        }
    }
    
    output::print_header("Granting sudo privileges");
    
    for username in &targets {
        // Check if user is in our state
        if let Err(e) = state.get_user(username) {
            output::print_warning(&format!("Skipping user '{}': {}", username, e));
            continue;
        }
        
        output::print_arrow(&format!("Adding '{}' to sudoers group…", username));
        if let Err(e) = user::add_to_sudoers(username) {
            output::print_error(&format!("Failed to add '{}' to sudoers: {}", username, e));
        } else {
            output::print_success(&format!("User '{}' is now a sudoer", username));
        }
    }
    
    Ok(())
}
