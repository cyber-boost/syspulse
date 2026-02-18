use anyhow::Result;
use clap::Parser;
use commands::{Cli, Commands};

mod client;
mod commands;
mod output;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up tracing based on verbose/quiet flags
    let filter = if cli.verbose {
        "debug"
    } else if cli.quiet {
        "error"
    } else {
        "info"
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    // Disable color if requested via --no-color flag.
    // Set NO_COLOR env var so owo-colors and other tools respect it.
    if cli.no_color {
        std::env::set_var("NO_COLOR", "1");
    }

    // Resolve socket path: CLI flag > env > default
    let socket_path = cli
        .socket
        .clone()
        .unwrap_or_else(|| syspulse_core::paths::socket_path());

    match cli.command {
        Commands::Daemon => {
            commands::daemon_cmd::run(cli.data_dir).await?;
        }
        Commands::Start {
            name,
            wait,
            timeout,
        } => {
            commands::start::run(&socket_path, &name, wait, timeout, &cli.format).await?;
        }
        Commands::Stop {
            name,
            force,
            timeout,
        } => {
            commands::stop::run(&socket_path, &name, force, timeout, &cli.format).await?;
        }
        Commands::Restart { name, force, wait } => {
            commands::restart::run(&socket_path, &name, force, wait, &cli.format).await?;
        }
        Commands::Status { name } => {
            commands::status::run(&socket_path, name.as_deref(), &cli.format).await?;
        }
        Commands::List => {
            commands::list::run(&socket_path, &cli.format).await?;
        }
        Commands::Logs {
            name,
            lines,
            stderr,
            follow,
        } => {
            commands::logs::run(&socket_path, &name, lines, stderr, follow, &cli.format).await?;
        }
        Commands::Add {
            file,
            name,
            command,
        } => {
            commands::add::run(
                &socket_path,
                file.as_deref(),
                name.as_deref(),
                command.as_deref(),
                &cli.format,
            )
            .await?;
        }
        Commands::Remove { name, force } => {
            commands::remove::run(&socket_path, &name, force, &cli.format).await?;
        }
        Commands::Init { path } => {
            commands::init::run(&path)?;
        }
    }

    Ok(())
}
