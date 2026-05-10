use crate::cli::LogsArgs;
use crate::error::UswitchError;
use crate::output;
use crate::runtime;
use crate::state::State;

pub fn execute(args: LogsArgs) -> Result<(), UswitchError> {
    let state = State::load()?;
    
    let username = if let Some(u) = args.user {
        u
    } else {
        let active: Vec<String> = state.users.iter()
            .filter(|(n, _)| runtime::runtime_is_active(n))
            .map(|(n, _)| n.clone())
            .collect();
            
        match active.len() {
            0 => {
                return Err(UswitchError::RuntimeNotFound("No active runtime found".into()));
            }
            1 => active[0].clone(),
            _ => {
                output::print_info(&format!("Multiple active runtimes found ({}).", active.len()));
                output::print_bullet("Specify a user: usw logs <user>");
                return Ok(());
            }
        }
    };

    // Check if user is in our state
    state.get_user(&username)?;
    
    output::print_header(&format!("Runtime logs: {}", username));
    runtime::get_logs(&username, args.lines, args.follow)?;
    
    Ok(())
}
