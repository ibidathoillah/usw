use crate::cli::MonitorArgs;
use crate::output::{self, StatusRow};
use crate::runtime;
use crate::state::State;
use crate::error::UswitchError;

pub fn execute(args: MonitorArgs) -> Result<(), UswitchError> {
    let state = State::load()?;

    let rows: Vec<StatusRow> = if let Some(user) = args.user {
        let user_state = state.get_user(&user)?;
        let active = runtime::runtime_is_active(&user);
        vec![StatusRow::new(
            &user,
            active,
            if active { "running" } else { "stopped" },
            &user_state.projects,
            user_state.workspace.as_deref(),
        )]
    } else {
        let mut rows: Vec<StatusRow> = state
            .users
            .iter()
            .map(|(name, us)| {
                let active = runtime::runtime_is_active(name);
                StatusRow::new(
                    name,
                    active,
                    if active { "running" } else { "stopped" },
                    &us.projects,
                    us.workspace.as_deref(),
                )
            })
            .collect();
        rows.sort_by(|a, b| a.user.cmp(&b.user));
        rows
    };

    let mut stdout = std::io::stdout();
    output::render_status_table(&mut stdout, &rows)?;

    Ok(())
}
