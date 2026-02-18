# Syspulse

Syspulse is a lightweight, cross‑platform daemon manager that gives developers and DevOps engineers a unified, reliable way to control long‑running processes and services across Windows, macOS, and Linux environments.

It abstracts away platform‑specific quirks, offering a consistent CLI and API for process lifecycle management, health monitoring, and log handling. Define daemons in a single TOML file and manage them with uniform commands, cutting operational overhead across Windows, macOS, and Linux.

## Value Proposition

Managing services across different operating systems can be challenging. Syspulse simplifies this by offering a single, consistent interface for daemon management on Windows, macOS, and Linux.

Key benefits:
- **Cross-platform consistency** - Same configuration and commands work everywhere
- **Health monitoring** - Built-in HTTP, TCP, and command-based health checks
- **Smart restart policies** - Automatic recovery with exponential backoff
- **Integrated log rotation** - Automatic log management with configurable retention
- **Scheduled jobs** - Run periodic tasks without external cron dependencies
- **Resource limits** - Control memory and CPU usage to prevent resource exhaustion

## Quick Start

Install syspulse:
```bash
cargo install syspulse-cli
```

Create a daemon configuration:
```toml
# mydaemons.toml
[[daemon]]
name = "web-server"
command = ["python", "-m", "http.server", "8000"]
description = "Simple HTTP server"
```

Start managing your daemons:
```bash
# Start the daemon manager
syspulse daemon &

# Add your daemon
syspulse add --file mydaemons.toml

# Start the web server
syspulse start web-server

# Check status
syspulse status
```

## Features

- **Process Management**: Start, stop, restart daemons with graceful shutdown
- **Health Monitoring**: Continuous health checks with automatic recovery
- **Restart Policies**: Configurable restart strategies with backoff controls
- **Log Rotation**: Automatic log file rotation with compression options
- **Scheduling**: Cron-style scheduling for periodic tasks
- **Resource Limits**: Memory and CPU usage limits to prevent resource hogging
- **JSON API**: Both human-readable and machine-parseable output formats
- **Tagging**: Group and filter daemons by tags for easier management

## Documentation

See our full documentation in the [`docs/`](docs/) directory:
- [Quick Start Guide](docs/QUICKSTART.md) - Step-by-step tutorial
- [Configuration Reference](docs/CONFIG.md) - Complete TOML configuration options
- [Why Syspulse?](docs/WHY.md) - Comparison with alternatives and decision guide

## Examples

Check out the [`examples/`](examples/) directory for sample configurations:
- Simple daemon setups
- Full-featured multi-daemon configurations
- Platform-specific examples for Windows, macOS, and Linux
- Scheduled job configurations

## Development

Syspulse is built with Rust and organized as a Cargo workspace:

```
syspulse/
├── crates/
│   ├── syspulse-core/    # Core daemon management logic
│   ├── syspulse-cli/     # Command-line interface
│   └── syspulse-python/  # Python bindings
├── examples/             # Sample configurations
└── docs/                 # Documentation
```

Build from source:
```bash
cargo build --release
```

Run tests:
```bash
cargo test
```
