mod kill;
mod create;
mod destroy;
pub mod install;
mod plugin_cmd;
mod purge;
mod monitor;
mod env;

use crate::cli::Commands;

pub fn dispatch(cmd: Commands) -> anyhow::Result<()> {
    let result = match cmd {
        Commands::Create(args) => create::execute(args),
        Commands::Plugin(args) => plugin_cmd::execute(args),
        Commands::Install(args) => install::execute(args),
        Commands::Monitor(args) => monitor::execute(args),
        Commands::Destroy(args) => destroy::execute(args),
        Commands::Kill(args) => kill::execute(args),
        Commands::Purge(args) => purge::execute(args),
        Commands::Env(args) => env::execute(args),
    };

    result.map_err(|e| anyhow::anyhow!(e))
}
