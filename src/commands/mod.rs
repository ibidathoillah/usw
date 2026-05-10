mod kill;
mod create;
mod destroy;
pub mod install;
mod plugin_cmd;
mod purge;
mod monitor;
mod env;
mod sudo;

use crate::cli::Commands;

pub fn dispatch(cmd: Commands) -> anyhow::Result<()> {
    match cmd {
        Commands::Use(args) => crate::switch::run(&args.user),
        Commands::Create(args) => create::execute(args).map_err(|e| anyhow::anyhow!(e)),
        Commands::Plugin(args) => plugin_cmd::execute(args).map_err(|e| anyhow::anyhow!(e)),
        Commands::Install(args) => install::execute(args).map_err(|e| anyhow::anyhow!(e)),
        Commands::Monitor(args) => monitor::execute(args).map_err(|e| anyhow::anyhow!(e)),
        Commands::Destroy(args) => destroy::execute(args).map_err(|e| anyhow::anyhow!(e)),
        Commands::Kill(args) => kill::execute(args).map_err(|e| anyhow::anyhow!(e)),
        Commands::Purge(args) => purge::execute(args).map_err(|e| anyhow::anyhow!(e)),
        Commands::Env(args) => env::execute(args).map_err(|e| anyhow::anyhow!(e)),
        Commands::Current => crate::switch::current(),
        Commands::Sudo(args) => sudo::execute(args).map_err(|e| anyhow::anyhow!(e)),
    }
}
