# Quickstart Guide

## Prerequisites

- Rust toolchain with `cargo` installed. Get it from https://rustup.rs.
- A terminal that can run the commands below. The steps work on Windows, macOS, and Linux.

## Install the CLI

The easiest way to get the `syspulse` command line tool is to install it from crates.io.

```bash
cargo install syspulse-cli
```

After the install finishes, make sure the binary directory is in your `PATH`. On most systems this is `~/.cargo/bin` (Windows adds it to your user profile automatically).

## Create a basic daemon configuration

You can start with a ready‑made template:

```bash
syspulse init mydaemons.sys
```

Open `mydaemons.sys` and replace its content with a minimal example:

```toml
[[daemon]]
name = "example"
command = "echo Hello from Syspulse"
restart = "on-failure"
```

The `[[daemon]]` table defines a single daemon. **Do not use `[[daemons]]`; the singular form is required.**

## Start the daemon manager

Run the daemon manager in the background:

```bash
syspulse daemon &
```

The `&` makes the process run in the background on Unix‑like shells. On Windows PowerShell you can start it with the `Start-Process` cmdlet or simply add `Start-Job`.

## Register a daemon with the manager

If the manager is already running you can add a new daemon without restarting it:

```bash
syspulse add --file mydaemons.sys
```

The command reads the file and registers any `[[daemon]]` definitions it finds.

## Start and stop a daemon

To start a specific daemon:

```bash
syspulse start example
```

To stop it:

```bash
syspulse stop example
```

You can also restart a daemon with `syspulse restart <name>`.

## View status and logs

Check which daemons are running:

```bash
syspulse status
```

Show the log output of a daemon (replace `example` with your daemon name):

```bash
syspulse logs example
```

## Troubleshooting tips

- **`cargo` not found** – make sure the Rust toolchain is installed and its `bin` directory is on `PATH`.
- **Permission errors** – on Windows run the terminal as Administrator; on macOS/Linux ensure the command you launch is executable.
- **Configuration errors** – verify your `.sys` file is valid.
- **Daemon exits immediately** – verify the command you gave. If it finishes quickly the manager will restart it according to the `restart` policy.

You now have a working Syspulse setup. From here you can add more daemons, tweak restart policies, or explore health‑check features.
