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

fn main() {
    let cli = usw::cli::Cli::try_parse();

    match cli {
        Ok(args) => {
            init_tracing(args.verbose);
            if let Err(e) = run(args) {
                output::print_error(&e.to_string());
                std::process::exit(1);
            }
        }
        Err(e) => {
            // This handles help, version, and parsing errors (invalid commands)
            e.exit();
        }
    }
}

fn run(cli: usw::cli::Cli) -> anyhow::Result<()> {
    usw::ensure_root().map_err(|e| anyhow::anyhow!(e))?;
    usw::ensure_directories().map_err(|e| anyhow::anyhow!(e))?;
    usw::commands::dispatch(cli.command)
}
