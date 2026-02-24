pub mod add;
pub mod daemon_cmd;
pub mod init;
pub mod list;
pub mod logs;
pub mod remove;
pub mod restart;
pub mod start;
pub mod status;
pub mod stop;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "syspulse", version, about = "Cross-platform daemon manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Custom data directory
    #[arg(long, global = true, env = "SYSPULSE_DATA_DIR")]
    pub data_dir: Option<std::path::PathBuf>,

    /// Custom socket path
    #[arg(long, global = true)]
    pub socket: Option<std::path::PathBuf>,

    /// Output format
    #[arg(long, global = true, default_value = "table")]
    pub format: OutputFormat,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run the daemon manager (foreground)
    Daemon,
    /// Start a daemon
    Start {
        /// Daemon name
        name: String,
        /// Wait for daemon to be running
        #[arg(long)]
        wait: bool,
        /// Timeout in seconds (with --wait)
        #[arg(long)]
        timeout: Option<u64>,
    },
    /// Stop a daemon
    Stop {
        /// Daemon name
        name: String,
        /// Force kill immediately
        #[arg(long)]
        force: bool,
        /// Timeout in seconds
        #[arg(long)]
        timeout: Option<u64>,
    },
    /// Restart a daemon
    Restart {
        /// Daemon name
        name: String,
        /// Force kill before restart
        #[arg(long)]
        force: bool,
        /// Wait for daemon to be running after restart
        #[arg(long)]
        wait: bool,
    },
    /// Show daemon status
    Status {
        /// Daemon name (omit for all)
        name: Option<String>,
    },
    /// List all daemons
    List,
    /// View daemon logs
    Logs {
        /// Daemon name
        name: String,
        /// Number of lines to show
        #[arg(short = 'n', long, default_value = "50")]
        lines: usize,
        /// Show stderr instead of stdout
        #[arg(long)]
        stderr: bool,
        /// Follow log output (not implemented in v0.1)
        #[arg(short, long)]
        follow: bool,
    },
    /// Add a new daemon
    Add {
        /// Load from config file (.sys)
        #[arg(long)]
        file: Option<std::path::PathBuf>,
        /// Daemon name (when not using --file)
        #[arg(long)]
        name: Option<String>,
        /// Command to run (when not using --file)
        #[arg(long, num_args = 1..)]
        command: Option<Vec<String>>,
    },
    /// Remove a daemon
    Remove {
        /// Daemon name
        name: String,
        /// Force remove even if running
        #[arg(long)]
        force: bool,
    },
    /// Generate a template config file
    Init {
        /// Output path
        #[arg(default_value = "syspulse.sys")]
        path: std::path::PathBuf,
    },
}
