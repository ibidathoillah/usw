use clap::{Parser, Subcommand, Args, ArgAction};

#[derive(Parser)]
#[command(
    name = "usw",
    version,
    about = "Switch AI runtimes (1 Linux user = 1 isolated runtime)",
    long_about = None,
    after_help = "Quick switch:  usw use <user>\n\nShortcuts:\n  usw c <user>   Create\n  usw d <user>   Destroy\n  usw m          Monitor\n  usw i          Install\n  usw p          Plugin\n  usw k          Kill\n  usw x          Purge\n  usw e <user>   Env\n  usw su <user>  Sudo",
)]
pub struct Cli {
    #[arg(short, long, action = ArgAction::Count, global = true, help = "Verbose")]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create or switch to a runtime
    #[command(name = "use")]
    Use(UseArgs),

    /// Create runtime
    #[command(aliases = ["c", "mk", "up", "add", "new"])]
    Create(CreateArgs),

    /// List plugins
    #[command(name = "plugin", aliases = ["p", "pl"])]
    Plugin(PluginArgs),

    /// Install tool
    #[command(name = "install", aliases = ["i", "in"])]
    Install(InstallArgs),

    /// Monitor runtimes
    #[command(name = "monitor", aliases = ["m", "ps", "s", "list", "ls"])]
    Monitor(MonitorArgs),

    /// Destroy runtime
    #[command(aliases = ["d", "rm", "del"])]
    Destroy(DestroyArgs),

    /// Stop runtime processes
    #[command(name = "kill", aliases = ["k", "stop"])]
    Kill(KillArgs),

    /// Destroy runtime
    #[command(name = "purge", aliases = ["x", "clear", "nuke"])]
    Purge(PurgeArgs),

    /// Manage runtime environment variables
    #[command(name = "env", aliases = ["e"])]
    Env(EnvArgs),

    /// Show active runtime
    Current,

    /// Grant sudo privileges to a runtime user
    #[command(name = "sudo", aliases = ["su", "admin"])]
    Sudo(SudoArgs),

    /// View runtime logs
    #[command(name = "logs", aliases = ["l", "journal"])]
    Logs(LogsArgs),
}

#[derive(Args)]
pub struct UseArgs {
    pub user: String,
}

#[derive(Args)]
pub struct CreateArgs {
    pub name: String,
    #[arg(long)]
    pub no_start: bool,
    #[arg(long, help = "Add user to sudoers group")]
    pub sudo: bool,
}

#[derive(Args)]
pub struct PluginArgs {
    pub name: Option<String>,
}

#[derive(Args)]
pub struct InstallArgs {
    pub tool: Option<String>,
    #[arg(short, long)]
    pub list: bool,
}

#[derive(Args)]
pub struct MonitorArgs {
    pub user: Option<String>,
}

#[derive(Args)]
pub struct DestroyArgs {
    pub user: String,
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args)]
pub struct KillArgs {
    /// Target user (default: active runtime)
    pub user: Option<String>,
    #[arg(short, long, help = "Kill all runtimes")]
    pub all: bool,
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args)]
pub struct PurgeArgs {
    /// Target user (default: active runtime)
    pub user: Option<String>,
    #[arg(short, long, help = "Purge all runtimes")]
    pub all: bool,
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args)]
pub struct EnvArgs {
    pub user: String,

    #[command(subcommand)]
    pub action: Option<EnvAction>,
}

#[derive(Args)]
pub struct SudoArgs {
    /// Target user (default: active runtime)
    pub user: Option<String>,
    #[arg(short, long, help = "Grant sudo to all runtimes")]
    pub all: bool,
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args)]
pub struct LogsArgs {
    /// Target user (default: active runtime)
    pub user: Option<String>,
    #[arg(short, long, default_value_t = 50)]
    pub lines: usize,
    #[arg(short, long)]
    pub follow: bool,
}

#[derive(Subcommand)]
pub enum EnvAction {
    /// Show all environment variables (default)
    Show,
    /// Set or update a variable
    Set {
        /// KEY=value pair
        pair: String,
    },
    /// Remove a variable
    Unset {
        /// Variable name
        key: String,
    },
    /// Open env file in $EDITOR
    Edit,
}
