use clap::Parser;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use usw::output;

fn init_tracing(verbose: u8) {
    let filter = match verbose {
        0 => EnvFilter::new("warn"),
        1 => EnvFilter::new("usw=info"),
        2 => EnvFilter::new("usw=debug"),
        _ => EnvFilter::new("usw=trace"),
    };

    let subscriber = fmt::layer()
        .with_target(false)
        .with_file(false)
        .without_time()
        .with_filter(filter);

    tracing_subscriber::registry().with(subscriber).init();
}

const SUBCOMMANDS: &[&str] = &[
    "create", "c", "mk", "up", "add", "new",
    "plugin", "p", "pl",
    "install", "i", "in",
    "monitor", "m", "ps", "s",
    "destroy", "d", "rm", "del",
    "kill", "k", "stop",
    "purge", "x", "clear", "nuke",
    "env", "e",
    "help", "--help", "-h",
    "version", "--version", "-V",
];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let first = args.get(1).map(|s| s.as_str()).unwrap_or("");

    let result = run(first);

    if let Err(e) = result {
        output::print_error(&e.to_string());
        std::process::exit(1);
    }
}

fn run(first: &str) -> anyhow::Result<()> {
    if first.is_empty() || SUBCOMMANDS.contains(&first) || first.starts_with('-') {
        let cli = usw::cli::Cli::parse();
        init_tracing(cli.verbose);
        usw::ensure_root().map_err(|e| anyhow::anyhow!(e))?;
        usw::ensure_directories().map_err(|e| anyhow::anyhow!(e))?;
        usw::commands::dispatch(cli.command)
    } else {
        usw::ensure_root().map_err(|e| anyhow::anyhow!(e))?;
        usw::ensure_directories().map_err(|e| anyhow::anyhow!(e))?;
        if first == "current" {
            return usw::switch::current();
        }
        usw::switch::run(first)
    }
}
